use std::cell::RefCell;
use std::io::{Read, Write};
use std::path::PathBuf;

use crate::types::{Device, SpiPins};

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
    pub log_dss: String,
    pub xds_id: String,
    pub device_kind: Device,
    pub spi_pins: Option<SpiPins>,
    pub subcommand: Subcommand,
}
