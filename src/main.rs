// Copyright (c) 2020 , Texas Instruments.
// Licensed under the BSD-3-Clause license
// (see LICENSE or <https://opensource.org/licenses/BSD-3-Clause>) All files in the project
// notice may not be copied, modified, or distributed except according to those terms.

extern crate byte_unit;
#[macro_use]
extern crate clap;
extern crate dss;
extern crate path_clean;
extern crate path_slash;
extern crate rust_embed;
#[macro_use]
extern crate snafu;
extern crate tempfile;

use std::env;
use std::path::PathBuf;
use std::process;
use std::str::FromStr;

use dss::{
    Dss,
    com::ti::ccstudio::scripting::environment::TraceLevel,
};

use snafu::{Backtrace, ErrorCompat, OptionExt, ResultExt, Snafu};

use args::Args;
use dss_logger::DssLogger;
use flash_rover::FlashRover;

mod app;
mod args;
mod assets;
mod command;
mod dss_logger;
mod firmware;
mod flash_rover;
mod types;
mod xflash;

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
    DssError {
        source: dss::Error,
        backtrace: Backtrace,
    },
    DssLoggerError {
        source: dss_logger::Error,
        backtrace: Backtrace,
    },
    FlashRoverError {
        source: flash_rover::Error,
        backtrace: Backtrace,
    },
}

type Result<T, E = Error> = std::result::Result<T, E>;

fn main() {
    if let Err(err) = run() {
        eprintln!("Error: {}", err);
        if let Some(backtrace) = ErrorCompat::backtrace(&err) {
            eprintln!("{}", backtrace);
        }
        process::exit(1);
    }
}

fn run() -> Result<()> {
    let args = Args::parse().context(ArgsError {})?;

    let ccs_root = get_ccs_root().context(NoCCSDir {})?;
    let command = args.command(&ccs_root).context(ArgsError {})?;

    let trace_level = TraceLevel::from_str(&command.log_dss).unwrap_or(TraceLevel::Off);
    let mut dss_log = DssLogger::new(trace_level);

    let dss_obj = Dss::new(command.ccs_path.as_path()).context(DssError {})?;
    let script = dss_obj.scripting_environment().context(DssError {})?;

    dss_log.start(&script).context(DssLoggerError {})?;

    let status = FlashRover::new(&script, command)
        .and_then(|cli| cli.run())
        .context(FlashRoverError {});

    if let Err(err) = status {
        if let Some(dss_log_path) = dss_log.keep() {
            eprintln!(
                "A DSS error occured with DSS logging enabled, check the log file here: {}",
                dss_log_path.display()
            );
        }
        return Err(err);
    };

    dss_log.stop(&script).context(DssLoggerError {})?;

    Ok(())
}

fn get_ccs_root() -> Option<PathBuf> {
    if cfg!(debug_assertions) {
        env::var_os("CCS_ROOT").map(Into::into)
    } else {
        // Find <SDK> in ancestors where <SDK>/ccs_base and <SDK>/eclipse exists
        let current_dir: PathBuf = env::current_exe().ok()?.parent()?.into();
        current_dir
            .ancestors()
            .find(|p| p.join("ccs_base").exists() && p.join("eclipse").exists())
            .map(Into::into)
    }
}
