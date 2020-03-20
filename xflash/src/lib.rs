// Copyright (c) 2020 , Texas Instruments.
// Licensed under the BSD-3-Clause license
// (see LICENSE or <https://opensource.org/licenses/BSD-3-Clause>) All files in the project
// notice may not be copied, modified, or distributed except according to those terms.

extern crate byte_unit;
extern crate dss;
extern crate jni;
extern crate path_clean;
extern crate path_slash;
extern crate rust_embed;
#[macro_use]
extern crate snafu;
extern crate tempfile;

use std::path::PathBuf;
use std::str::FromStr;

use dss::com::ti::ccstudio::scripting::environment::{ScriptingEnvironment, TraceLevel};
use snafu::{Backtrace, ResultExt, Snafu};
use tempfile::{Builder, NamedTempFile};

use crate::command::Command;
use crate::flash_rover::FlashRover;

mod assets;
pub mod command;
mod flash_rover;
pub mod types;
mod xflash;

#[derive(Debug, Snafu)]
pub enum Error {
    DssError {
        source: dss::Error,
        backtrace: Backtrace,
    },
    FlashRoverError {
        source: flash_rover::Error,
        backtrace: Backtrace,
    },
}

pub type Result<T, E = Error> = std::result::Result<T, E>;

struct DssLogger {
    trace_level: TraceLevel,
    file: Option<NamedTempFile>,
}

impl DssLogger {
    const STYLESHEET: &'static str = "DefaultStylesheet.xsl";

    fn new(trace_level: TraceLevel) -> Self {
        let file = match trace_level {
            TraceLevel::Off => None,
            _ => Builder::new()
                .prefix("flash-rover.dss-log.")
                .suffix(".xml")
                .tempfile()
                .ok(),
        };

        Self { trace_level, file }
    }

    fn start(&self, script: &ScriptingEnvironment) -> Result<()> {
        script
            .trace_set_console_level(TraceLevel::Off)
            .context(DssError {})?;

        if let Some(file_path) = self
            .file
            .as_ref()
            .map(|file| file.path().to_str())
            .flatten()
        {
            script
                .trace_begin(file_path, DssLogger::STYLESHEET)
                .context(DssError {})?;
            script
                .trace_set_file_level(self.trace_level)
                .context(DssError {})?;
        }

        Ok(())
    }

    fn stop(&self, script: &ScriptingEnvironment) -> Result<()> {
        if self.file.is_some() {
            script.trace_end().context(DssError {})?;
        }

        Ok(())
    }

    fn keep(&mut self) -> Option<PathBuf> {
        if let Some(file) = self.file.take() {
            let (_file, path) = file.keep().ok()?;
            Some(path)
        } else {
            None
        }
    }
}

pub fn run(command: Command) -> Result<()> {
    let trace_level = TraceLevel::from_str(&command.log_dss).unwrap_or(TraceLevel::Off);
    let mut dss_logger = DssLogger::new(trace_level);

    let dss_obj = dss::Dss::new(command.ccs_path.as_path()).context(DssError {})?;
    let script = dss_obj.scripting_environment().context(DssError {})?;

    dss_logger.start(&script)?;

    let status = FlashRover::new(&script, command)
        .and_then(|cli| cli.run())
        .context(FlashRoverError {});

    if let Err(err) = status {
        if let Some(dss_logger_path) = dss_logger.keep() {
            eprintln!(
                "A DSS error occured with logging enabled, check the log file here: {}",
                dss_logger_path.display()
            );
        }
        return Err(err);
    };

    dss_logger.stop(&script)?;

    Ok(())
}
