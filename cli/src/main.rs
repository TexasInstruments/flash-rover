// Copyright (c) 2019 , Texas Instruments.
// Licensed under the BSD-3-Clause license
// (see LICENSE or <https://opensource.org/licenses/BSD-3-Clause>) All files in the project
// notice may not be copied, modified, or distributed except according to those terms.

extern crate byte_unit;
#[macro_use]
extern crate clap;
#[macro_use]
extern crate failure;
extern crate j4rs;
extern crate path_clean;
extern crate path_slash;
extern crate tempfile;

use std::process;

use failure::Error;

mod app;
mod args;
mod dss;
mod flash_rover;

fn main() {
    if let Err(err) = args::Args::parse().and_then(try_main) {
        eprintln!("{}", err);
        process::exit(1);
    }
}

fn try_main(args: args::Args) -> Result<(), Error> {
    let command = args.command()?;
    dss::FlashRover::new(command)?.run()?;

    Ok(())
}
