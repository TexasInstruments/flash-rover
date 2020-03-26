// Copyright (c) 2020 , Texas Instruments.
// Licensed under the BSD-3-Clause license
// (see LICENSE or <https://opensource.org/licenses/BSD-3-Clause>) All files in the project
// notice may not be copied, modified, or distributed except according to those terms.

use std::io::{self, Read, Write};
use std::time::Duration;

use dss::com::ti::{
    ccstudio::scripting::environment::ScriptingEnvironment,
    debug::engine::scripting::{DebugServer, DebugSession},
};
use snafu::{Backtrace, ResultExt, Snafu};
use tempfile::TempPath;

use crate::assets;
use crate::command::{Command, Subcommand};
use crate::firmware::{self, Firmware};
use crate::types::Device;

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("An IO error occured: {}", source))]
    IoError {
        source: io::Error,
        backtrace: Backtrace,
    },
    #[snafu(display("A DSS error occured: {}", source))]
    DssError {
        source: dss::Error,
        backtrace: Backtrace,
    },
    #[snafu(display("A Firmware error occured: {}", source))]
    FirmwareError {
        source: firmware::Error,
        backtrace: Backtrace,
    },
    #[snafu(display("Received too few bytes from input"))]
    InvalidInputLength { backtrace: Backtrace },
    #[snafu(display("Verification of written data failed"))]
    VerificationFailed { backtrace: Backtrace },
    #[snafu(display("Unable to create CCXML file: {}", source))]
    CreateCcxmlError {
        source: io::Error,
        backtrace: Backtrace,
    },
    #[snafu(display("Unable to create firmware file: {}", source))]
    CreateFirmwareError {
        source: io::Error,
        backtrace: Backtrace,
    },
}

pub type Result<T, E = Error> = std::result::Result<T, E>;

const DEBUG_SERVER_NAME: &str = "DebugServer.1";
const SCRIPT_TIMEOUT: Duration = Duration::from_secs(15);
const SESSION_PATTERN: &str = "Texas Instruments XDS110 USB Debug Probe/Cortex_M(3|4)_0";

fn create_ccxml(xds: &str, device: Device) -> Result<TempPath> {
    let asset = assets::get_ccxml_template(device)
        .ok_or_else(|| io::Error::from(io::ErrorKind::NotFound))
        .context(CreateCcxmlError {})?;

    let patterns = &[
        ("<<<SERIAL NUMBER>>>", xds),
        ("<<<DEVICE DESC>>>", device.ccxml_desc()),
        ("<<<DEVICE XML>>>", device.ccxml_xml()),
        ("<<<DEVICE ID>>>", device.ccxml_id()),
    ];

    let content = String::from_utf8_lossy(&asset[..]).to_string();
    let content = patterns.iter().fold(content, |state, pattern| {
        state.replace(pattern.0, pattern.1)
    });

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

pub struct FlashRover<'a> {
    command: Command,
    debug_server: DebugServer<'a>,
    debug_session: DebugSession<'a>,
    firmware: Firmware<'a>,
}

impl<'a> FlashRover<'a> {
    pub fn new(script: &'a ScriptingEnvironment<'a>, command: Command) -> Result<Self> {
        let ccxml = create_ccxml(&command.xds_id, command.device)?;

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

        let firmware = Firmware::new(debug_session.memory.clone(), command.device)
            .context(FirmwareError {})?;

        Ok(Self {
            command,
            debug_server,
            debug_session,
            firmware,
        })
    }

    fn reset_into_firmware(&self) -> Result<()> {
        const EXPRESSION_BOARD_RESET: &str =
            "GEL_AdvancedReset(\"Board Reset (automatic connect/disconnect)\")";

        if !self.debug_session.target.is_halted().context(DssError {})? {
            self.debug_session.target.halt().context(DssError {})?;
        }

        self.debug_session.target.reset().context(DssError {})?;
        self.debug_session
            .expression
            .evaluate(EXPRESSION_BOARD_RESET)
            .context(DssError {})?;

        self.firmware
            .inject(self.command.spi_pins)
            .context(FirmwareError {})?;

        self.debug_session
            .target
            .run_asynch()
            .context(DssError {})?;

        Ok(())
    }

    pub fn run(self) -> Result<()> {
        use Subcommand::*;

        self.reset_into_firmware()?;

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
        let data = self
            .firmware
            .read_data(offset, length)
            .context(FirmwareError {})?;
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
            let read_bytes = input.take(length as _).read(&mut vec).context(IoError {})?;
            ensure!(read_bytes == length as _, InvalidInputLength {});
            vec
        } else {
            let mut vec = Vec::new();
            input.read_to_end(&mut vec).context(IoError {})?;
            vec
        };

        let length = input_buf.len() as u32;

        if in_place {
            self.firmware
                .write_data(offset, &input_buf)
                .context(FirmwareError {})?;

            if verify {
                self.reset_into_firmware()?;

                let read_back = self
                    .firmware
                    .read_data(offset, length)
                    .context(FirmwareError {})?;

                ensure!(input_buf.eq(&read_back), VerificationFailed {});
            }
        } else {
            let first_address = offset - offset % firmware::BUF_SIZE;
            let first_length = offset % firmware::BUF_SIZE;
            let last_address = offset + length;
            let last_length =
                (firmware::BUF_SIZE - last_address % firmware::BUF_SIZE) % firmware::BUF_SIZE;

            let first_sector_part: Vec<u8> = self
                .firmware
                .read_data(first_address, first_length)
                .context(FirmwareError {})?;
            let last_sector_part: Vec<u8> = self
                .firmware
                .read_data(last_address, last_length)
                .context(FirmwareError {})?;

            let total_input: Vec<u8> = first_sector_part
                .into_iter()
                .chain(input_buf.into_iter())
                .chain(last_sector_part.into_iter())
                .collect();
            let total_length = total_input.len() as u32;

            self.firmware
                .sector_erase(first_address, total_length)
                .context(FirmwareError {})?;
            self.firmware
                .write_data(first_address, &total_input)
                .context(FirmwareError {})?;

            if verify {
                self.reset_into_firmware()?;

                let read_back = self
                    .firmware
                    .read_data(first_address, total_length)
                    .context(FirmwareError {})?;

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
    }
}
