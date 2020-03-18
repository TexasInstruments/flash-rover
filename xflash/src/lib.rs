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

use snafu::{Backtrace, ResultExt, Snafu};

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

pub fn run(command: Command) -> Result<()> {
    let dss_obj = dss::Dss::new(command.ccs_path.as_path()).context(DssError {})?;
    let flash_rover = FlashRover::new(&dss_obj, command).context(FlashRoverError {})?;
    flash_rover.run().context(FlashRoverError {})?;
    Ok(())
}
