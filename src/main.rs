// Copyright (c) 2020 , Texas Instruments.
// Licensed under the BSD-3-Clause license
// (see LICENSE or <https://opensource.org/licenses/BSD-3-Clause>) All files in the project
// notice may not be copied, modified, or distributed except according to those terms.

#[macro_use]
extern crate clap;
extern crate path_clean;
extern crate path_slash;
extern crate snafu;
extern crate xflash;

use std::env;
use std::path::{Path, PathBuf};
use std::process;

use snafu::{Backtrace, OptionExt, ResultExt, Snafu};

use args::Args;

mod app;
mod args;

#[derive(Debug, Snafu)]
enum Error {
    ArgsError {
        source: args::Error,
        backtrace: Backtrace,
    },
    CurrentDirError {
        backtrace: Backtrace,
    },
    #[snafu(display("Unable to find CCS root"))]
    NoCCSDir {
        backtrace: Backtrace,
    },
    XflashError {
        source: xflash::Error,
        backtrace: Backtrace,
    },
}

type Result<T, E = Error> = std::result::Result<T, E>;

fn main() {
    if let Err(err) = run() {
        eprintln!("{:?}", err);
        process::exit(1);
    }
}

fn run() -> Result<()> {
    let args = Args::parse().context(ArgsError {})?;
    
    let current_dir = get_current_dir().context(CurrentDirError {})?;
    let ccs_root = get_ccs_root(&current_dir).context(NoCCSDir {})?;

    let command = args.command(&ccs_root).context(ArgsError {})?;

    xflash::run(command).context(XflashError {})?;

    Ok(())
}

fn get_current_dir() -> Option<PathBuf> {
    env::current_exe().ok()?.parent().map(Into::into)
}

fn get_ccs_root(current_dir: &Path) -> Option<PathBuf> {
    if cfg!(debug_assertions) {
        env::var_os("CCS_ROOT").map(Into::into)
    } else {
        // Find <SDK> in ancestors where <SDK>/ccs_base and <SDK>/eclipse exists
        current_dir
            .ancestors()
            .find(|p| p.join("ccs_base").exists() && p.join("eclipse").exists())
            .map(Into::into)
    }
}
