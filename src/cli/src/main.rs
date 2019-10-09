#[macro_use]
extern crate clap;
extern crate byte_unit;
extern crate failure;
extern crate path_clean;

use std::{
    env, fmt,
    fs::File,
    io,
    io::{Read, Write},
    path::Path,
    process,
};

use byte_unit::Byte;
use clap::{App, AppSettings, Arg, ArgMatches, SubCommand};
use failure::{err_msg, Error};
use path_clean::PathClean;

struct XflashInfo {
    manufacturer_id: u32,
    device_id: u32,
    size: u32,
    name: &'static str,
}

impl fmt::Display for XflashInfo {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{} (MID: {:X}, DID: {:X}, size: {})",
            self.name,
            self.manufacturer_id,
            self.device_id,
            Byte::from_bytes(self.size as u128)
                .get_appropriate_unit(true)
                .to_string()
        )
    }
}

const SUPPORTED_XFLASH_HW: &'static [XflashInfo] = &[
    XflashInfo {
        manufacturer_id: 0xC2,
        device_id: 0x15,
        size: 0x200000,
        name: "Macronics MX25R1635F",
    },
    XflashInfo {
        manufacturer_id: 0xC2,
        device_id: 0x14,
        size: 0x100000,
        name: "Macronics MX25R8035F",
    },
    XflashInfo {
        manufacturer_id: 0xEF,
        device_id: 0x12,
        size: 0x80000,
        name: "WinBond W25X40CL",
    },
    XflashInfo {
        manufacturer_id: 0xEF,
        device_id: 0x11,
        size: 0x40000,
        name: "WinBond W25X20CL",
    },
];

fn find_xflash(mid: u32, did: u32) -> Option<&'static XflashInfo> {
    for xflash in SUPPORTED_XFLASH_HW {
        if xflash.manufacturer_id == mid && xflash.device_id == did {
            return Some(xflash);
        }
    }
    None
}

fn spi_pins_validate(dios: String) -> Result<(), String> {
    if dios.split(',').all(|dio| dio.parse::<u32>().is_ok()) {
        Ok(())
    } else {
        Err(String::from("DIO values must be positive integers"))
    }
}

enum Command {
    Info,
    SectorErase {
        offset: u32,
        length: u32,
    },
    MassErase,
    Read {
        offset: u32,
        length: u32,
        io: Box<dyn Write>,
    },
    Write {
        erase: Option<bool>,
        verify: Option<String>,
        offset: u32,
        length: Option<u32>,
        io: Box<dyn Read>,
    },
}

impl Command {
    fn from_matches<'a>(matches: &ArgMatches<'a>) -> Result<Self, Error> {
        match matches.subcommand() {
            ("info", _) => Ok(Command::Info),
            ("erase", Some(erase_matches)) => {
                if erase_matches.is_present("mass-erase") {
                    Ok(Command::MassErase)
                } else {
                    Ok(Command::SectorErase {
                        // unwrap is OK since offset and length are required and validated
                        offset: value_t!(erase_matches, "offset", u32).unwrap(),
                        length: value_t!(erase_matches, "length", u32).unwrap(),
                    })
                }
            }
            ("read", Some(read_matches)) => {
                Ok(Command::Read {
                    // unwrap is OK since offset and length are required and validated
                    offset: value_t!(read_matches, "offset", u32).unwrap(),
                    length: value_t!(read_matches, "length", u32).unwrap(),
                    io: if let Some(output_path) = read_matches.value_of("output") {
                        Box::new(File::create(output_path)?)
                    } else {
                        Box::new(io::stdout())
                    },
                })
            }
            ("write", Some(write_matches)) => {
                Ok(Command::Write {
                    erase: value_t!(write_matches, "erase", bool).ok(),
                    verify: value_t!(write_matches, "verify", String).ok(),
                    // Only unwrap on offset is OK, since length is not required
                    offset: value_t!(write_matches, "offset", u32).unwrap(),
                    length: if let Some(length) = write_matches.value_of("length") {
                        Some(length.parse::<u32>().unwrap())
                    } else {
                        None
                    },
                    io: if let Some(input_path) = write_matches.value_of("input") {
                        Box::new(File::open(input_path)?)
                    } else {
                        Box::new(io::stdin())
                    },
                })
            }
            // This is OK since subcommand is required
            (_, _) => unreachable!(),
        }
    }
}

struct Cli {
    ccs: String,
    xds: String,
    device: String,
    spi_pins: Vec<String>,
    command: Command,
}

impl Cli {
    fn from_matches<'a>(matches: &ArgMatches<'a>) -> Result<Self, Error> {
        Ok(Self {
            ccs: value_t!(matches, "ccs", String)?,
            xds: value_t!(matches, "xds", String)?,
            device: value_t!(matches, "device", String)?,
            spi_pins: values_t!(matches, "spi-pins", String)?,
            command: Command::from_matches(matches)?,
        })
    }

    fn run<'a>(matches: &ArgMatches<'a>) -> Result<(), Error> {
        let cli = Cli::from_matches(matches)?;
        let dss = Path::new(&cli.ccs)
            .join("ccs_base/scripting/bin/")
            .join(if cfg!(target_os = "windows") {
                "dss.bat"
            } else {
                "dss.sh"
            })
            .clean();

        let exe = env::current_exe()?;
        let pwd = exe.parent().unwrap();
        let dss_script = pwd.join("src/dss_inject_fw.js").clean();
        let ccxml = pwd.join("src").join(&cli.xds).clean();

        println!("{:?}", dss_script);
        println!("{:?}", ccxml);

        let mut command = process::Command::new(dss);
        command
            .arg(dss_script)
            .arg(&cli.device)
            .arg("conf")
            .args(&cli.spi_pins[..]);

        match &cli.command {
            Command::Info => {
                command.arg("info");
                let output = command.output()?;
                if output.status.success() {
                    let stdout = String::from_utf8_lossy(&output.stdout[..]);
                    let mid_did: Vec<_> = stdout.trim().split(' ').collect();
                    if mid_did.len() != 2 {
                        return Err(err_msg("Firmware misbehaved during info command"));
                    }
                    let mid = mid_did[0]
                        .parse::<u32>()
                        .map_err(|_| err_msg("Invalid MID recieved"))?;
                    let did = mid_did[1]
                        .parse::<u32>()
                        .map_err(|_| err_msg("Invalid DID recieved"))?;
                    if let Some(xflash) = find_xflash(mid, did) {
                        println!("{}", xflash);
                    } else {
                        println!("Unsupported external flash (MID: {}, DID: {})", mid, did);
                    }
                } else {
                    eprintln!("{}", String::from_utf8(output.stderr)?);
                }
            }
            Command::MassErase => {
                command.arg("mass-erase");
                print!("Starting mass erase, this may take some time... ");
                let output = command.output()?;
                if output.status.success() {
                    println!("Done.");
                } else {
                    eprintln!("{}", String::from_utf8(output.stderr)?);
                }
            }
            Command::SectorErase { offset, length } => {
                command.arg("sector-erase");

            }
            Command::Read { offset, length, io } => {}
            Command::Write {
                erase,
                verify,
                offset,
                length,
                io,
            } => {}
        }

        Ok(())
    }

    fn info(&self, cmd: &mut process::Command) -> Result<(), Error> {
        unimplemented!();
    }
}

fn is_zero_or_positive(val: String) -> Result<(), String> {
    if val.parse::<u32>().is_ok() {
        Ok(())
    } else {
        Err(String::from("Value must be a zero or positive integer"))
    }
}

fn main() {
    let matches = App::new(crate_name!())
        .version(crate_version!())
        .author(crate_authors!())
        .about("Read and write to the external flash")
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .arg(Arg::with_name("ccs")
            .help("Path to where CCS installed")
            .short("c")
            .long("ccs")
            .value_name("PATH")
            .env("CCS_ROOT")
            .takes_value(true))
        .arg(Arg::with_name("xds")
            .help("The serial number ID of the XDS110 debugger connected to the device, e.g. L4100847")
            .short("x")
            .long("xds")
            .value_name("ID")
            .required(true))
        .arg(Arg::with_name("device")
            .help("The kind of device connected to the XDS110 debugger")
            .short("d")
            .long("device")
            .value_name("KIND")
            .possible_values(&["cc13x0", "cc26x0", "cc26x0r2", "cc13x2_cc26x2"])
            .required(true))
        .arg(Arg::with_name("spi-pins")
            .help("Override default SPI DIOs for external flash access, defaults to DIOs used for external flash on LaunchPads")
            .short("s")
            .long("spi-pins")
            .value_names(&["MISO", "MOSI", "CLK", "CSN"])
            .value_delimiter(",")
            .require_delimiter(true)
            .default_value("8,9,10,20")
            .validator(spi_pins_validate))
        .subcommand(SubCommand::with_name("info")
            .about("Get external flash device info")    
        )
        .subcommand(SubCommand::with_name("erase")
            .about("Perform erase operation, either on sectors or mass erase")
            .arg(Arg::with_name("offset")
                .help("Offset of bytes into external flash device to start erase")
                .value_name("OFFSET")
                .index(1)
                .validator(is_zero_or_positive)
                .required_unless("mass-erase"))
            .arg(Arg::with_name("length")
                .help("Length of bytes to erase from offset")
                .value_name("LENGTH")
                .index(2)
                .validator(is_zero_or_positive)
                .required_unless("mass-erase"))
            .arg(Arg::with_name("mass-erase")
                .help("Perform mass erase of the entire external flash device")
                .short("m")
                .long("mass-erase")
                .conflicts_with_all(&["offset", "length"]))
        )
        .subcommand(SubCommand::with_name("read")
            .about("Read data from an address range on the external flash")
            .arg(Arg::with_name("offset")
                .help("Offset of bytes into external flash device to start read")
                .value_name("OFFSET")
                .index(1)
                .validator(is_zero_or_positive)
                .required(true))
            .arg(Arg::with_name("length")
                .help("Length of bytes to read from offset")
                .value_name("LENGTH")
                .index(2)
                .validator(is_zero_or_positive)
                .required(true))
            .arg(Arg::with_name("output")
                .short("o")
                .long("output")
                .value_name("FILE")
                .help("File to store read data. Will overwrite file. Writes to stdout if omitted.")
                .takes_value(true))
        )
        .subcommand(SubCommand::with_name("write")
            .about("Write data to an address range on the external flash")
            .arg(Arg::with_name("erase")
                .help("Erase sectors before writing to them")
                .short("e")
                .long("erase"))
            .arg(Arg::with_name("verify")
                .help("Verify written data")
                .short("v")
                .long("verify")
                .value_name("MODE")
                .possible_values(&["crc", "readback"]))
            .arg(Arg::with_name("offset")
                .help("Offset of bytes into external flash device to start write")
                .value_name("OFFSET")
                .index(1)
                .validator(is_zero_or_positive)
                .required(true))
            .arg(Arg::with_name("length")
                .help("Length of bytes to write from offset")
                .value_name("LENGTH")
                .index(2)
                .validator(is_zero_or_positive))
            .arg(Arg::with_name("input")
                .short("i")
                .long("input")
                .value_name("FILE")
                .help("File to read contents of data to write. Reads from stdin if omitted.")
                .takes_value(true))
        )
        .get_matches();

    Cli::run(&matches).unwrap_or_else(|err| {
        eprintln!("Error: {}", err);
        process::exit(1);
    });
}
