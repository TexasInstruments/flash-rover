// Copyright (c) 2020 , Texas Instruments.
// Licensed under the BSD-3-Clause license
// (see LICENSE or <https://opensource.org/licenses/BSD-3-Clause>) All files in the project
// notice may not be copied, modified, or distributed except according to those terms.

use std::io::{self, Read, Write};
use std::time::Duration;

use dss::com::ti::{
    ccstudio::scripting::environment::ScriptingEnvironment,
    debug::engine::scripting::{DebugServer, DebugSession, Register},
};
use snafu::{Backtrace, ResultExt, Snafu};
use tempfile::TempPath;

use crate::assets;
use crate::command::{Command, Subcommand};
use crate::firmware::{self, Firmware};
use crate::types::{Device, SpiPin};

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
    FirmwareError {
        source: firmware::Error,
        backtrace: Backtrace,
    },
    #[snafu(display("Received too few bytes from input"))]
    InvalidInputLength {
        backtrace: Backtrace,
    },
    #[snafu(display("Verification of written data failed"))]
    VerificationFailed {
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
}

pub type Result<T, E = Error> = std::result::Result<T, E>;

const DEBUG_SERVER_NAME: &str = "DebugServer.1";
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

fn create_ccxml(xds: &str, device: Device) -> Result<TempPath> {
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

fn create_firmware(device: Device) -> Result<TempPath> {
    let asset = assets::get_firmware(device)
        .ok_or_else(|| io::Error::from(io::ErrorKind::NotFound))
        .context(CreateFirmwareError {})?;

    let mut firmware = tempfile::Builder::new()
        .prefix("flash-rover.fw.")
        .suffix(".bin")
        .tempfile()
        .context(CreateFirmwareError {})?;
    firmware.write_all(&asset).context(CreateFirmwareError {})?;
    let (file, path) = firmware.into_parts();
    drop(file);

    Ok(path)
}

pub struct FlashRover<'a> {
    command: Command,
    ccxml: Option<TempPath>,
    debug_server: DebugServer<'a>,
    debug_session: DebugSession<'a>,
    firmware: Firmware<'a>,
}

impl<'a> FlashRover<'a> {
    pub fn new(script: &'a ScriptingEnvironment<'a>, command: Command) -> Result<Self> {
        let ccxml = create_ccxml(&command.xds_id, command.device_kind)?;

        script
            .set_script_timeout(SCRIPT_TIMEOUT)
            .context(DssError {})?;

        let debug_server = script.get_server(DEBUG_SERVER_NAME).context(DssError {})?;
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

        let firmware = Firmware::new(debug_session.memory.clone());

        let ccxml = Some(ccxml);

        Ok(Self {
            command,
            ccxml,
            debug_server,
            debug_session,
            firmware,
        })
    }

    pub fn run(self) -> Result<()> {
        use Subcommand::*;

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
                verify,
                in_place,
                offset,
                length,
                input,
            } => self.write(
                *verify,
                *in_place,
                *offset,
                *length,
                input.borrow_mut().as_mut(),
            )?,
        }

        Ok(())
    }

    fn inject(&self) -> Result<()> {
        let memory = &self.debug_session.memory;

        let fw = create_firmware(self.command.device_kind)?;

        memory
            .load_raw(0, SRAM_START as _, fw.to_str().unwrap(), 32, false as _)
            .context(DssError {})?;

        fw.close().context(IoError {})?;

        if let Some(spi_pins) = self.command.spi_pins.as_ref() {
            memory
                .write_data(0, CONF_VALID as _, 1, 32)
                .context(DssError {})?;
            memory
                .write_data(0, CONF_SPI_MISO as _, spi_pins[SpiPin::Miso] as _, 32)
                .context(DssError {})?;
            memory
                .write_data(0, CONF_SPI_MOSI as _, spi_pins[SpiPin::Mosi] as _, 32)
                .context(DssError {})?;
            memory
                .write_data(0, CONF_SPI_CLK as _, spi_pins[SpiPin::Clk] as _, 32)
                .context(DssError {})?;
            memory
                .write_data(0, CONF_SPI_CSN as _, spi_pins[SpiPin::Csn] as _, 32)
                .context(DssError {})?;
        }

        let stack_addr = memory
            .read_data(0, STACK_ADDR as _, 32, false as _)
            .context(DssError {})?;
        let reset_isr = memory
            .read_data(0, RESET_ISR as _, 32, false as _)
            .context(DssError {})?;

        memory
            .write_register(Register::MSP, stack_addr)
            .context(DssError {})?;
        memory
            .write_register(Register::PC, reset_isr)
            .context(DssError {})?;
        memory
            .write_register(Register::LR, 0xFFFF_FFFF)
            .context(DssError {})?;

        self.debug_session
            .target
            .run_asynch()
            .context(DssError {})?;

        Ok(())
    }

    fn info(&self) -> Result<()> {
        let xflash_info = self.firmware.get_xflash_info().context(FirmwareError {})?;

        println!("{}", xflash_info);

        Ok(())
    }

    fn sector_erase(&self, offset: u32, length: u32) -> Result<()> {
        self.firmware
            .sector_erase(offset, length)
            .context(FirmwareError {})?;

        Ok(())
    }

    fn mass_erase(&self) -> Result<()> {
        print!("Starting mass erase, this may take some time... ");
        io::stdout().flush().context(IoError {})?;

        self.firmware.mass_erase().context(FirmwareError {})?;

        println!("Done.");
        Ok(())
    }

    fn read(&self, offset: u32, length: u32, output: &mut dyn Write) -> Result<()> {
        let data = self.firmware.read_data(offset, length).context(FirmwareError {})?;
        io::copy(&mut data.as_slice(), output).context(IoError {})?;

        Ok(())
    }

    fn write(
        &self,
        verify: bool,
        in_place: bool,
        offset: u32,
        length: Option<u32>,
        input: &mut dyn Read,
    ) -> Result<()> {
        let input_buf: Vec<u8> = if let Some(length) = length {
            let mut vec = Vec::with_capacity(length as _);
            let read_bytes = input
                .take(length as _)
                .read(&mut vec)
                .context(IoError {})?;
            ensure!(read_bytes == length as _, InvalidInputLength {});
            vec
        } else {
            let mut vec = Vec::new();
            input.read_to_end(&mut vec).context(IoError {})?;
            vec
        };

        let length = input_buf.len() as u32;

        if in_place {
            self.firmware.write_data(offset, &input_buf).context(FirmwareError {})?;

            if verify {
                let read_back = self.firmware.read_data(offset, length).context(FirmwareError {})?;
                
                ensure!(input_buf.eq(&read_back), VerificationFailed {});
            }
        } else {
            let first_address = offset - offset % firmware::BUF_SIZE;
            let first_length = offset % firmware::BUF_SIZE;
            let last_address = offset + length;
            let last_length = (firmware::BUF_SIZE - last_address % firmware::BUF_SIZE) % firmware::BUF_SIZE;

            let first_sector_part: Vec<u8> = self.firmware
                .read_data(first_address, first_length)
                .context(FirmwareError{})?;
            let last_sector_part: Vec<u8> = self.firmware
                .read_data(last_address, last_length)
                .context(FirmwareError{})?;

            let total_input: Vec<u8> = first_sector_part
                .into_iter()
                .chain(input_buf.clone().into_iter())
                .chain(last_sector_part.into_iter())
                .collect();
            let total_length = total_input.len() as u32;

            self.firmware.sector_erase(first_address, total_length).context(FirmwareError{})?;
            self.firmware.write_data(first_address, &total_input).context(FirmwareError{})?;

            if verify {
                let read_back = self.firmware.read_data(first_address, total_length).context(FirmwareError {})?;

                ensure!(total_input.eq(&read_back), VerificationFailed {});
            }
        }

        Ok(())
    }
}

impl<'a> Drop for FlashRover<'a> {
    fn drop(&mut self) {
        let f = || -> Result<(), Box<dyn std::error::Error>> {
            self.debug_session.target.halt()?;
            self.debug_session.target.reset()?;
            self.debug_session.target.disconnect()?;

            self.debug_server.stop()?;

            Ok(())
        };
        f().unwrap_or_default();
        if let Some(ccxml) = self.ccxml.take() {
            ccxml.close().unwrap_or_default();
        }
    }
}
