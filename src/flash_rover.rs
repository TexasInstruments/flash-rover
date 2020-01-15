// Copyright (c) 2020 , Texas Instruments.
// Licensed under the BSD-3-Clause license
// (see LICENSE or <https://opensource.org/licenses/BSD-3-Clause>) All files in the project
// notice may not be copied, modified, or distributed except according to those terms.

use std::io::{self, Read, Write};
use std::thread;
use std::time::Duration;

use snafu::{Backtrace, IntoError, ResultExt, Snafu};
use tempfile::TempPath;

use crate::args;
use crate::assets;
use crate::dss;
use crate::types::{Device, SpiPin};
use crate::xflash::XflashInfo;

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("An IO error has occured: {}", source))]
    IoError {
        source: io::Error,
        backtrace: Backtrace,
    },
    #[snafu(display("A DSS error occured: {}", source))]
    DssError {
        source: dss::Error,
        backtrace: Backtrace,
    },
    #[snafu(display("Unable to create CCXML file: {}", source))]
    CreateCcxmlError {
        source: io::Error,
        backtrace: Backtrace,
    },
    #[snafu(display("Unable to create firmware: {}", source))]
    CreateFirmwareError {
        source: io::Error,
        backtrace: Backtrace,
    },
    #[snafu(display("A firmware error has occured: {}", msg))]
    FwError { msg: String, backtrace: Backtrace },
}

pub type Result<T, E = Error> = std::result::Result<T, E>;

const LOG_FILENAME: &str = "dss_log.xml";
const LOG_STYLESHEET: &str = "DefaultStylesheet.xsl";
const SCRIPT_TIMEOUT: Duration = Duration::from_secs(15);
const SESSION_PATTERN: &str = "Texas Instruments XDS110 USB Debug Probe/Cortex_M(3|4)_0";

const SRAM_START: u32 = 0x2000_0000;
const STACK_ADDR: u32 = SRAM_START;
const RESET_ISR: u32 = SRAM_START + 0x04;

const CONF_START: u32 = 0x2000_3000;
const CONF_VALID: u32 = CONF_START;
const CONF_SPI_MISO: u32 = CONF_START + 0x04;
const CONF_SPI_MOSI: u32 = CONF_START + 0x08;
const CONF_SPI_CLK: u32 = CONF_START + 0x0C;
const CONF_SPI_CSN: u32 = CONF_START + 0x10;

const DOORBELL_START: u32 = 0x2000_3100;
const DOORBELL_CMD: u32 = DOORBELL_START;
const DOORBELL_RSP: u32 = DOORBELL_START + 0x10;

const XFLASH_BUF_START: u32 = 0x2000_4000;
const XFLASH_BUF_SIZE: u32 = 0x1000;

fn create_ccxml(xds: &str, device: &Device) -> Result<TempPath> {
    let asset = assets::get_ccxml_template(device)
        .ok_or_else(|| io::Error::from(io::ErrorKind::NotFound))
        .context(CreateCcxmlError {})?;

    let content = String::from_utf8_lossy(&asset[..]);
    const PATTERN: &str = "<<<SERIAL NUMBER>>>";
    let content = content.replace(PATTERN, &xds);

    let mut ccxml = tempfile::Builder::new()
        .prefix("flash-rover.ccxml.")
        .suffix(".ccxml")
        .tempfile()
        .context(CreateCcxmlError {})?;
    ccxml
        .write_all(content.as_bytes())
        .context(CreateCcxmlError {})?;

    let (file, path) = ccxml.into_parts();
    drop(file);

    Ok(path)
}

fn create_firmware(device: &Device) -> Result<TempPath> {
    let asset = assets::get_firmware(device)
        .ok_or_else(|| io::Error::from(io::ErrorKind::NotFound))
        .context(CreateFirmwareError {})?;

    let mut firmware = tempfile::Builder::new()
        .prefix("flash-rover.fw.")
        .suffix(".bin")
        .tempfile()
        .context(CreateFirmwareError {})?;
    firmware
        .write_all(&asset)
        .context(CreateFirmwareError {})?;
    let (file, path) = firmware.into_parts();
    drop(file);

    Ok(path)
}

#[derive(Copy, Clone, Debug)]
enum FwCmd {
    GetXflashInfo,
    SectorErase { offset: u32, length: u32 },
    MassErase,
    ReadBlock { offset: u32, length: u32 },
    WriteBlock { offset: u32, length: u32 },
}

impl FwCmd {
    fn to_bytes(&self) -> [u32; 4] {
        use FwCmd::*;

        match self {
            GetXflashInfo => [0xC0_u32.to_le(), 0, 0, 0],
            SectorErase { offset, length } => [0xC1_u32.to_le(), offset.to_le(), length.to_le(), 0],
            MassErase => [0xC2_u32.to_le(), 0, 0, 0],
            ReadBlock { offset, length } => [0xC3_u32.to_le(), offset.to_le(), length.to_le(), 0],
            WriteBlock { offset, length } => [0xC4_u32.to_le(), offset.to_le(), length.to_le(), 0],
        }
    }
}

#[derive(Copy, Clone, Debug)]
enum FwRsp {
    Ok,
    XflashInfo { mid: u32, did: u32 },
}

impl FwRsp {
    fn from_bytes(bytes: &[u32; 4]) -> Result<FwRsp> {
        const OK_VAL: u32 = 0xD0_u32.to_le();
        const XFLASHINFO_VAL: u32 = 0xD1_u32.to_le();

        let rsp = match bytes {
            [OK_VAL, 0, 0, 0] => FwRsp::Ok,
            [XFLASHINFO_VAL, mid, did, 0] => FwRsp::XflashInfo {
                mid: *mid,
                did: *did,
            },
            rsp => FwError {
                msg: format!("Received invalid FW response: {:?}", rsp),
            }
            .fail()?,
        };
        Ok(rsp)
    }
}

pub struct FlashRover {
    command: args::Command,
    ccxml: Option<TempPath>,
    script: dss::ScriptingEnvironment,
    debug_server: dss::DebugServer,
    debug_session: dss::DebugSession,
}

impl FlashRover {
    pub fn new(command: args::Command) -> Result<Self> {
        let ccxml = create_ccxml(&command.xds_id, &command.device_kind)?;
        let jvm = dss::build_jvm(command.ccs_path.as_path()).context(DssError {})?;

        let script = dss::ScriptingEnvironment::new(jvm).context(DssError {})?;
        script
            .trace_begin(LOG_FILENAME, LOG_STYLESHEET)
            .context(DssError {})?;
        script
            .trace_set_console_level(dss::TraceLevel::Off)
            .context(DssError {})?;
        script
            .trace_set_file_level(dss::TraceLevel::All)
            .context(DssError {})?;
        script
            .set_script_timeout(SCRIPT_TIMEOUT)
            .context(DssError {})?;

        let debug_server = script.get_server().context(DssError {})?;
        debug_server
            .set_config(&ccxml.to_string_lossy().to_owned())
            .context(DssError {})?;

        let debug_session = debug_server
            .open_session(SESSION_PATTERN)
            .context(DssError {})?;
        debug_session.target.connect().context(DssError {})?;
        debug_session.target.reset().context(DssError {})?;
        debug_session
            .expression
            .evaluate("GEL_AdvancedReset(\"Board Reset (automatic connect/disconnect)\")")
            .context(DssError {})?;
        
        let ccxml = Some(ccxml);

        Ok(Self {
            command,
            ccxml,
            script,
            debug_server,
            debug_session,
        })
    }

    pub fn run(self) -> Result<()> {
        use args::Subcommand::*;

        self.inject()?;

        match &self.command.subcommand {
            Info => self.info()?,
            SectorErase { offset, length } => self.sector_erase(*offset, *length)?,
            MassErase => self.mass_erase()?,
            Read {
                offset,
                length,
                output,
            } => self.read(*offset, *length, output.borrow_mut().as_mut())?,
            Write {
                erase,
                offset,
                length,
                input,
            } => self.write(*erase, *offset, *length, input.borrow_mut().as_mut())?,
        }

        Ok(())
    }

    fn inject(&self) -> Result<()> {
        let memory = &self.debug_session.memory;

        let fw = create_firmware(&self.command.device_kind)?;

        memory
            .load_raw(0, SRAM_START, fw.to_str().unwrap(), 32, false)
            .context(DssError {})?;

        fw.close().context(IoError{})?;

        if let Some(spi_pins) = self.command.spi_pins.as_ref() {
            memory
                .write_data(0, CONF_VALID, 1, 32)
                .context(DssError {})?;
            memory
                .write_data(0, CONF_SPI_MISO, spi_pins[SpiPin::Miso] as u32, 32)
                .context(DssError {})?;
            memory
                .write_data(0, CONF_SPI_MOSI, spi_pins[SpiPin::Mosi] as u32, 32)
                .context(DssError {})?;
            memory
                .write_data(0, CONF_SPI_CLK, spi_pins[SpiPin::Clk] as u32, 32)
                .context(DssError {})?;
            memory
                .write_data(0, CONF_SPI_CSN, spi_pins[SpiPin::Csn] as u32, 32)
                .context(DssError {})?;
        }

        let stack_addr = memory
            .read_data(0, STACK_ADDR, 32, false)
            .context(DssError {})?;
        let reset_isr = memory
            .read_data(0, RESET_ISR, 32, false)
            .context(DssError {})?;

        memory
            .write_register(dss::Register::MSP, stack_addr)
            .context(DssError {})?;
        memory
            .write_register(dss::Register::PC, reset_isr)
            .context(DssError {})?;
        memory
            .write_register(dss::Register::LR, 0xFFFF_FFFF)
            .context(DssError {})?;

        self.debug_session
            .target
            .run_asynch()
            .context(DssError {})?;

        Ok(())
    }

    fn send_fw_cmd(&self, fw_cmd: FwCmd) -> Result<FwRsp> {
        let memory = &self.debug_session.memory;

        let fw_cmd_bytes = fw_cmd.to_bytes();

        memory
            .write_data(0, DOORBELL_CMD + 0x0C, fw_cmd_bytes[3], 32)
            .context(DssError {})?;
        memory
            .write_data(0, DOORBELL_CMD + 0x08, fw_cmd_bytes[2], 32)
            .context(DssError {})?;
        memory
            .write_data(0, DOORBELL_CMD + 0x04, fw_cmd_bytes[1], 32)
            .context(DssError {})?;
        // Kind must be written last to trigger the command
        memory
            .write_data(0, DOORBELL_CMD, fw_cmd_bytes[0], 32)
            .context(DssError {})?;

        const SLEEP_TIME: Duration = Duration::from_millis(100);

        while memory
            .read_data(0, DOORBELL_CMD, 32, false)
            .context(DssError {})?
            != 0
        {
            thread::sleep(SLEEP_TIME);
        }

        while memory
            .read_data(0, DOORBELL_RSP, 32, false)
            .context(DssError {})?
            == 0
        {
            thread::sleep(SLEEP_TIME);
        }

        let fw_rsp_bytes: [u32; 4] = [
            memory
                .read_data(0, DOORBELL_RSP, 32, false)
                .context(DssError {})?,
            memory
                .read_data(0, DOORBELL_RSP + 0x04, 32, false)
                .context(DssError {})?,
            memory
                .read_data(0, DOORBELL_RSP + 0x08, 32, false)
                .context(DssError {})?,
            memory
                .read_data(0, DOORBELL_RSP + 0x0C, 32, false)
                .context(DssError {})?,
        ];

        memory
            .write_data(0, DOORBELL_RSP, 0, 32)
            .context(DssError {})?;

        FwRsp::from_bytes(&fw_rsp_bytes)
    }

    fn info(&self) -> Result<()> {
        match self.send_fw_cmd(FwCmd::GetXflashInfo)? {
            FwRsp::XflashInfo { mid, did } => {
                if let Some(xflash_info) = XflashInfo::find(mid, did) {
                    println!("{}", xflash_info);
                } else {
                    println!(
                        "Unknown and possibly unsupported external flash (MID: {}, DID: {})",
                        mid, did
                    );
                }
            }
            other_rsp => FwError {
                msg: format!(
                    "Received unexpected response from FW during info command: {:?}",
                    other_rsp
                ),
            }
            .fail()?,
        }

        Ok(())
    }

    fn sector_erase(&self, offset: u32, length: u32) -> Result<()> {
        match self.send_fw_cmd(FwCmd::SectorErase { offset, length })? {
            FwRsp::Ok => { /* success, do nothing */ }
            other_rsp => FwError {
                msg: format!(
                    "Received unexpected response from FW during sector-erase command: {:?}",
                    other_rsp
                ),
            }
            .fail()?,
        }

        Ok(())
    }

    fn mass_erase(&self) -> Result<()> {
        print!("Starting mass erase, this may take some time... ");
        io::stdout().flush().context(IoError {})?;

        match self.send_fw_cmd(FwCmd::MassErase)? {
            FwRsp::Ok => {
                println!("Done.");
            }
            other_rsp => FwError {
                msg: format!(
                    "Received unexpected response from FW during mass-erase command: {:?}",
                    other_rsp
                ),
            }
            .fail()?,
        }

        Ok(())
    }

    fn read(&self, offset: u32, length: u32, output: &mut dyn Write) -> Result<()> {
        let memory = &self.debug_session.memory;
        let mut length_rest = length;
        let mut offset_rest = offset;

        while length_rest > 0 {
            let ilength = std::cmp::min(length_rest, XFLASH_BUF_SIZE);

            let fw_cmd = FwCmd::ReadBlock {
                offset: offset_rest,
                length: ilength,
            };
            match self.send_fw_cmd(fw_cmd)? {
                FwRsp::Ok => { /* successful, do nothing */ }
                other_rsp => FwError {
                    msg: format!(
                        "Received unexpected response from FW during read command: {:?}",
                        other_rsp
                    ),
                }
                .fail()?,
            }

            let data = memory
                .read_datas(0, XFLASH_BUF_START, 8, ilength as usize, false)
                .context(DssError {})?;
            io::copy(&mut data.as_slice(), output).context(IoError {})?;

            length_rest -= ilength;
            offset_rest += ilength;
        }

        Ok(())
    }

    fn write(
        &self,
        erase: bool,
        offset: u32,
        length: Option<u32>,
        input: &mut dyn Read,
    ) -> Result<()> {
        let memory = &self.debug_session.memory;

        let vec = if let Some(length) = length {
            let length = length as usize;
            let mut vec = Vec::with_capacity(length);
            let read_bytes = input
                .take(length as u64)
                .read(&mut vec)
                .context(IoError {})?;
            if read_bytes != length {
                return Err(IoError {}.into_error(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    format!(
                        "Received too few bytes from input, expected {}, got {}",
                        length, read_bytes
                    ),
                )));
            }
            vec
        } else {
            let mut vec = Vec::new();
            input.read_to_end(&mut vec).context(IoError {})?;
            vec
        };

        let length = vec.len() as u32;

        let mut offset_rest = offset;

        if erase {
            self.sector_erase(offset, length)?;
        }

        for chunk in vec.chunks(XFLASH_BUF_SIZE as usize) {
            let ilength = chunk.len() as u32;

            memory
                .write_datas(0, XFLASH_BUF_START, chunk, 8)
                .context(DssError {})?;

            let fw_cmd = FwCmd::WriteBlock {
                offset: offset_rest,
                length: ilength,
            };
            match self.send_fw_cmd(fw_cmd)? {
                FwRsp::Ok => { /* successful, do nothing */ }
                other_rsp => {
                    return FwError {
                        msg: format!(
                            "Received unexpected response from FW during write command: {:?}",
                            other_rsp
                        ),
                    }
                    .fail();
                }
            }

            offset_rest += ilength;
        }

        Ok(())
    }
}

impl Drop for FlashRover {
    fn drop(&mut self) {
        let f = || -> Result<(), dss::Error> {
            self.debug_session.target.halt()?;
            self.debug_session.target.reset()?;
            self.debug_session.target.disconnect()?;

            self.debug_server.stop()?;

            self.script.trace_set_console_level(dss::TraceLevel::Info)?;
            self.script.trace_end()?;

            Ok(())
        };
        f().unwrap_or_default();
        if let Some(ccxml) = self.ccxml.take() {
            ccxml.close().unwrap_or_default();
        }
    }
}
