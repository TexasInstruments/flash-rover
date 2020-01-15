// Copyright (c) 2020 , Texas Instruments.
// Licensed under the BSD-3-Clause license
// (see LICENSE or <https://opensource.org/licenses/BSD-3-Clause>) All files in the project
// notice may not be copied, modified, or distributed except according to those terms.

use std::cell::RefCell;
use std::fs::File;
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use std::str;

use snafu::{Backtrace, OptionExt, ResultExt, Snafu};

use crate::app;
use crate::types::{Device, SpiPins};

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("Unable to parse {} from argmatch {}", value, name))]
    ParseArgMatch {
        name: String,
        value: String,
        backtrace: Backtrace,
    },
    #[snafu(display("Missing expected argument {}", arg))]
    MissingArgument { arg: String, backtrace: Backtrace },
    #[snafu(display("Unable to parse argument {}: {}", arg, reason))]
    ParseArgument {
        arg: String,
        reason: String,
        backtrace: Backtrace,
    },
    #[snafu(display("Argument '{}' is invalid: {}", arg, reason))]
    InvalidArgument {
        arg: String,
        reason: String,
        backtrace: Backtrace,
    },
    #[snafu(display("Path {} does not exist", path.display()))]
    InvalidPath { path: PathBuf, backtrace: Backtrace },
    #[snafu(display("Unable to create IO stream: {}", source))]
    CreateStreamError {
        source: io::Error,
        backtrace: Backtrace,
    },
    #[snafu(display("Invalid subcommand: {}", subcmd))]
    InvalidSubcommand {
        subcmd: String,
        backtrace: Backtrace,
    },
}

pub type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Clone, Debug)]
struct ArgMatches(clap::ArgMatches<'static>);

impl ArgMatches {
    fn new(clap_matches: clap::ArgMatches<'static>) -> Self {
        Self(clap_matches)
    }

    fn subcommand(&self) -> (&str, Option<ArgMatches>) {
        let (name, matches) = self.0.subcommand();
        (name, matches.cloned().map(ArgMatches::new))
    }

    fn value_of_lossy(&self, name: &str) -> Option<String> {
        self.0.value_of_lossy(name).map(|s| s.into_owned())
    }

    fn is_present(&self, name: &str) -> bool {
        self.0.is_present(name)
    }

    fn parse_of_lossy<T>(&self, name: &str) -> Result<Option<T>>
    where
        T: str::FromStr,
    {
        match self.value_of_lossy(name) {
            None => Ok(None),
            Some(value) => value
                .parse::<T>()
                .ok()
                .map(Some)
                .context(ParseArgMatch { name, value }),
        }
    }
}

pub enum Subcommand {
    Info,
    SectorErase {
        offset: u32,
        length: u32,
    },
    MassErase,
    Read {
        offset: u32,
        length: u32,
        output: RefCell<Box<dyn Write>>,
    },
    Write {
        erase: bool,
        offset: u32,
        length: Option<u32>,
        input: RefCell<Box<dyn Read>>,
    },
}

pub struct Command {
    pub ccs_path: PathBuf,
    pub xds_id: String,
    pub device_kind: Device,
    pub spi_pins: Option<SpiPins>,
    pub subcommand: Subcommand,
}

pub struct Args {
    matches: ArgMatches,
}

impl Args {
    pub fn parse() -> Result<Self> {
        let clap_matches = app::app().get_matches();
        let matches = ArgMatches::new(clap_matches);

        Ok(Self { matches })
    }

    pub fn ccs_path(&self) -> Result<PathBuf> {
        const ARG: &str = "ccs";
        let ccs = self
            .matches
            .value_of_lossy(ARG)
            .context(MissingArgument { arg: ARG })?;
        let path = Path::new(&ccs).to_path_buf();
        ensure!(path.exists(), InvalidPath { path });
        ensure!(path.join("ccs_base").exists(), InvalidArgument{ arg: ARG, reason: "CCS path does not contain the 'ccs_base/' subfolder" });
        Ok(path)
    }

    pub fn xds_id(&self) -> Result<String> {
        const ARG: &str = "xds";
        let arg = self
            .matches
            .value_of_lossy(ARG)
            .context(MissingArgument { arg: ARG })?;
        Ok(arg)
    }

    pub fn device_kind(&self) -> Result<Device> {
        const ARG: &str = "device";
        let arg = self
            .matches
            .parse_of_lossy(ARG)?
            .context(MissingArgument { arg: ARG })?;
        Ok(arg)
    }

    pub fn spi_pins(&self) -> Result<Option<SpiPins>> {
        const ARG: &str = "spi-pins";
        let arg = self.matches.parse_of_lossy(ARG)?;
        Ok(arg)
    }

    pub fn subcommand(&self) -> Result<Subcommand> {
        Ok(match self.matches.subcommand() {
            ("info", _) => Subcommand::Info,
            ("erase", Some(matches)) => {
                if matches.is_present("mass-erase") {
                    Subcommand::MassErase
                } else {
                    Subcommand::SectorErase {
                        offset: matches
                            .parse_of_lossy("offset")?
                            .context(MissingArgument { arg: "offset" })?,
                        length: matches
                            .parse_of_lossy("length")?
                            .context(MissingArgument { arg: "length" })?,
                    }
                }
            }
            ("read", Some(matches)) => Subcommand::Read {
                offset: matches
                    .parse_of_lossy("offset")?
                    .context(MissingArgument { arg: "offset" })?,
                length: matches
                    .parse_of_lossy("length")?
                    .context(MissingArgument { arg: "length" })?,
                output: RefCell::new(
                    if let Some(output_path) = matches.value_of_lossy("output") {
                        Box::new(File::create(output_path).context(CreateStreamError {})?)
                    } else {
                        Box::new(io::stdout())
                    },
                ),
            },
            ("write", Some(matches)) => Subcommand::Write {
                erase: matches.is_present("erase"),
                offset: matches
                    .parse_of_lossy("offset")?
                    .expect("Missing required argument 'offset'"),
                length: matches.parse_of_lossy("length")?,
                input: RefCell::new(if let Some(input_path) = matches.value_of_lossy("input") {
                    Box::new(File::open(input_path).context(CreateStreamError {})?)
                } else {
                    Box::new(io::stdin())
                }),
            },
            (subcmd, _) => InvalidSubcommand { subcmd }.fail()?,
        })
    }

    pub fn command(&self) -> Result<Command, Error> {
        Ok(Command {
            ccs_path: self.ccs_path()?,
            xds_id: self.xds_id()?,
            device_kind: self.device_kind()?,
            spi_pins: self.spi_pins()?,
            subcommand: self.subcommand()?,
        })
    }
}
