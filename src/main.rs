// Copyright (c) 2019 , Texas Instruments.
// Licensed under the BSD-3-Clause license
// (see LICENSE or <https://opensource.org/licenses/BSD-3-Clause>) All files in the project
// notice may not be copied, modified, or distributed except according to those terms.

extern crate byte_unit;
#[macro_use]
extern crate clap;
//#[macro_use]
extern crate j4rs;
extern crate path_clean;
extern crate path_slash;
extern crate rust_embed;
#[macro_use]
extern crate snafu;
extern crate tempfile;

use std::process;

use snafu::{Backtrace, ErrorCompat, ResultExt, Snafu};

mod app;
mod args;
mod assets;
mod dss;
mod flash_rover;
mod types;
mod xflash;

use args::Args;
use flash_rover::FlashRover;

#[derive(Debug, Snafu)]
enum Error {
    ArgsError {
        source: args::Error,
        backtrace: Backtrace,
    },
    FlashRoverError {
        source: flash_rover::Error,
        backtrace: Backtrace,
    },
}

type Result<T, E = Error> = std::result::Result<T, E>;

fn main() {
    if let Err(e) = run() {
        eprintln!("error: {}", e);
        if let Some(backtrace) = ErrorCompat::backtrace(&e) {
            eprintln!("{}", backtrace);
        }
        process::exit(1);
    }
}

fn run() -> Result<()> {
    let args = Args::parse().context(ArgsError)?;
    let command = args.command().context(ArgsError)?;
    let flash_rover = FlashRover::new(command).context(FlashRoverError {})?;
    flash_rover.run().context(FlashRoverError {})?;

    Ok(())
}
