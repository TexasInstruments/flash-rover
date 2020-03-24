// Copyright (c) 2020 , Texas Instruments.
// Licensed under the BSD-3-Clause license
// (see LICENSE or <https://opensource.org/licenses/BSD-3-Clause>) All files in the project
// notice may not be copied, modified, or distributed except according to those terms.

use std::path::PathBuf;

use dss::com::ti::ccstudio::scripting::environment::{ScriptingEnvironment, TraceLevel};
use snafu::{Backtrace, ResultExt, Snafu};
use tempfile::{Builder, NamedTempFile};

#[derive(Debug, Snafu)]
pub enum Error {
    DssError {
        source: dss::Error,
        backtrace: Backtrace,
    },
}

pub type Result<T, E = Error> = std::result::Result<T, E>;

pub struct DssLogger {
    trace_level: TraceLevel,
    file: Option<NamedTempFile>,
}

impl DssLogger {
    const STYLESHEET: &'static str = "DefaultStylesheet.xsl";

    pub fn new(trace_level: TraceLevel) -> Self {
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

    pub fn start(&self, script: &ScriptingEnvironment) -> Result<()> {
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

    pub fn stop(&self, script: &ScriptingEnvironment) -> Result<()> {
        if self.file.is_some() {
            script.trace_end().context(DssError {})?;
        }

        Ok(())
    }

    pub fn keep(&mut self) -> Option<PathBuf> {
        if let Some(file) = self.file.take() {
            let (_file, path) = file.keep().ok()?;
            Some(path)
        } else {
            None
        }
    }
}
