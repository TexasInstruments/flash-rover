// Copyright (c) 2019 , Texas Instruments.
// Licensed under the BSD-3-Clause license
// (see LICENSE or <https://opensource.org/licenses/BSD-3-Clause>) All files in the project
// notice may not be copied, modified, or distributed except according to those terms.

use std::cell::RefCell;
use std::fs::File;
use std::io;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::str::FromStr;

use failure::Error;

use crate::app;

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

    fn parse_of_lossy<T>(&self, name: &str) -> Result<Option<T>, Error>
    where
        T: FromStr,
        <T as FromStr>::Err: failure::Fail,
    {
        match self.value_of_lossy(name) {
            None => Ok(None),
            Some(v) => v.parse::<T>().map(Some).map_err(From::from),
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
    pub device_kind: String,
    pub spi_pins: Option<[u8; 4]>,
    pub subcommand: Subcommand,
}

pub struct Args {
    matches: ArgMatches,
}

impl Args {
    pub fn parse() -> Result<Self, Error> {
        let clap_matches = app::app().get_matches();
        let matches = ArgMatches::new(clap_matches);

        Ok(Self { matches })
    }

    pub fn ccs_path(&self) -> Result<PathBuf, Error> {
        let ccs = self
            .matches
            .value_of_lossy("ccs")
            .expect("Missing required argument 'ccs'");
        let path = Path::new(&ccs).to_path_buf();
        ensure!(path.exists(), "CCS path {} does not exist", ccs);
        Ok(path)
    }

    pub fn xds_id(&self) -> Result<String, Error> {
        Ok(self
            .matches
            .value_of_lossy("xds")
            .expect("Missing required argument 'xds'"))
    }

    pub fn device_kind(&self) -> Result<String, Error> {
        Ok(self
            .matches
            .value_of_lossy("device")
            .expect("Missing required argument 'device'"))
    }

    pub fn spi_pins(&self) -> Result<Option<[u8; 4]>, Error> {
        match self.matches.value_of_lossy("spi-pins") {
            None => Ok(None),
            Some(pins) => Ok({
                let dios = pins
                    .split(',')
                    .map(str::parse)
                    .collect::<Result<Vec<_>, _>>()?;
                ensure!(
                    dios.len() == 4,
                    "Argument 'spi-pins' expects 4 values, got {}",
                    dios.len()
                );
                let mut ok_dios: [_; 4] = Default::default();
                ok_dios.copy_from_slice(&dios[0..4]);
                Some(ok_dios)
            }),
        }
    }

    pub fn subcommand(&self) -> Result<Subcommand, Error> {
        Ok(match self.matches.subcommand() {
            ("info", _) => Subcommand::Info,
            ("erase", Some(matches)) => {
                if matches.is_present("mass-erase") {
                    Subcommand::MassErase
                } else {
                    Subcommand::SectorErase {
                        offset: matches
                            .parse_of_lossy("offset")?
                            .expect("Missing required argument 'offset'"),
                        length: matches
                            .parse_of_lossy("length")?
                            .expect("Missing required argument 'length'"),
                    }
                }
            }
            ("read", Some(matches)) => Subcommand::Read {
                offset: matches
                    .parse_of_lossy("offset")?
                    .expect("Missing required argument 'offset"),
                length: matches
                    .parse_of_lossy("length")?
                    .expect("Missing required argument 'length'"),
                output: RefCell::new(
                    if let Some(output_path) = matches.value_of_lossy("output") {
                        Box::new(File::create(output_path)?)
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
                    Box::new(File::open(input_path)?)
                } else {
                    Box::new(io::stdin())
                }),
            },
            (subcmd, _) => bail!("Invalid subcommand {}", subcmd),
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
