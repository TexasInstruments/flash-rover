// Copyright (c) 2019 , Texas Instruments.
// Licensed under the BSD-3-Clause license
// (see LICENSE or <https://opensource.org/licenses/BSD-3-Clause>) All files in the project
// notice may not be copied, modified, or distributed except according to those terms.

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
    path::{Path, PathBuf},
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
            "{} (MID: 0x{:X}, DID: 0x{:X}, size: {})",
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
        name: "Macronix MX25R1635F",
    },
    XflashInfo {
        manufacturer_id: 0xC2,
        device_id: 0x14,
        size: 0x100000,
        name: "Macronix MX25R8035F",
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

enum CommandKind {
    Info,
    SectorErase,
    MassErase,
    Read { io: Box<dyn Write> },
    Write { io: Box<dyn Read> },
}

struct Command {
    kind: CommandKind,
    cmd: process::Command,
}

impl Command {
    fn from_matches<'a>(matches: &ArgMatches<'a>) -> Result<Self, Error> {
        let ccs = value_t!(matches, "ccs", String)?;
        let device = value_t!(matches, "device", String)?;
        let spi_pins = values_t!(matches, "spi-pins", String)?;

        let dss = Path::new(&ccs)
            .join("ccs_base/scripting/bin/")
            .join(if cfg!(target_os = "windows") {
                "dss.bat"
            } else {
                "dss.sh"
            })
            .clean();

        let cwd = exe_dir()?;
        let dss_script = cwd.join("dss/dss-flash-rover.js").clean();

        let mut cmd = process::Command::new(dss);
        cmd.arg(dss_script)
            .arg(&device)
            .arg("conf")
            .args(&spi_pins[..]);

        match matches.subcommand() {
            ("info", _) => {
                cmd.arg("info");
                Ok(Command {
                    kind: CommandKind::Info,
                    cmd: cmd,
                })
            }
            ("erase", Some(erase_matches)) => {
                if erase_matches.is_present("mass-erase") {
                    cmd.arg("mass-erase");
                    Ok(Command {
                        kind: CommandKind::MassErase,
                        cmd: cmd,
                    })
                } else {
                    cmd.arg("sector-erase")
                        .arg(value_t!(erase_matches, "offset", String).unwrap())
                        .arg(value_t!(erase_matches, "length", String).unwrap());
                    Ok(Command {
                        kind: CommandKind::SectorErase,
                        cmd: cmd,
                    })
                }
            }
            ("read", Some(read_matches)) => {
                cmd.arg("read")
                    .arg(value_t!(read_matches, "offset", String).unwrap())
                    .arg(value_t!(read_matches, "length", String).unwrap());
                Ok(Command {
                    kind: CommandKind::Read {
                        io: if let Some(output_path) = read_matches.value_of("output") {
                            Box::new(File::create(output_path)?)
                        } else {
                            Box::new(io::stdout())
                        },
                    },
                    cmd: cmd,
                })
            }
            ("write", Some(write_matches)) => {
                cmd.arg("write")
                    .arg(value_t!(write_matches, "offset", String).unwrap())
                    .arg(if write_matches.is_present("length") {
                        value_t!(write_matches, "length", String).unwrap()
                    } else {
                        // unbounded write is indicated by -1
                        String::from("-1")
                    })
                    .arg(if write_matches.is_present("erase") {
                        "1"
                    } else {
                        "0"
                    });
                Ok(Command {
                    kind: CommandKind::Write {
                        io: if let Some(input_path) = write_matches.value_of("input") {
                            Box::new(File::open(input_path)?)
                        } else {
                            Box::new(io::stdin())
                        },
                    },
                    cmd: cmd,
                })
            }
            // This is OK since subcommand is required
            (_, _) => unreachable!(),
        }
    }
}

fn exe_dir() -> io::Result<PathBuf> {
    Ok(env::current_exe()?
        .parent()
        .ok_or(io::Error::from(io::ErrorKind::NotFound))?
        .to_owned())
}

fn create_ccxml(xds: &str, device: &str) -> io::Result<()> {
    let cwd = exe_dir()?;
    let template = cwd
        .join("dss/ccxml")
        .join(format!("template_{}.ccxml", device))
        .clean();
    let ccxml = cwd
        .join("dss/ccxml")
        .join(format!("{}.ccxml", device))
        .clean();

    let mut content = Vec::new();
    File::open(template)?.read_to_end(&mut content)?;

    let content = String::from_utf8_lossy(&content[..]);
    let pattern = "<<<SERIAL NUMBER>>>";
    let content = content.replace(pattern, &xds);

    File::create(ccxml)?.write_all(content.as_bytes())?;

    Ok(())
}

struct Cli {
    xds: String,
    device: String,
    command: Command,
}

impl Cli {
    fn from_matches<'a>(matches: &ArgMatches<'a>) -> Result<Self, Error> {
        Ok(Self {
            xds: value_t!(matches, "xds", String)?,
            device: value_t!(matches, "device", String)?,
            command: Command::from_matches(matches)?,
        })
    }

    fn run(&mut self) -> Result<(), Error> {
        create_ccxml(&self.xds, &self.device)?;

        env::set_current_dir(exe_dir()?)?;

        match &mut self.command {
            Command {
                kind: CommandKind::Info,
                cmd,
            } => Cli::info(cmd)?,
            Command {
                kind: CommandKind::MassErase,
                cmd,
            } => Cli::mass_erase(cmd)?,
            Command {
                kind: CommandKind::SectorErase,
                cmd,
            } => Cli::sector_erase(cmd)?,
            Command {
                kind: CommandKind::Read { io },
                cmd,
            } => Cli::read(cmd, &mut *io)?,
            Command {
                kind: CommandKind::Write { io },
                cmd,
            } => Cli::write(cmd, &mut *io)?,
        }

        Ok(())
    }

    fn info(cmd: &mut process::Command) -> Result<(), Error> {
        let output = cmd.output()?;
        if !output.status.success() {
            return Err(err_msg(
                String::from_utf8_lossy(&output.stderr[..]).into_owned(),
            ));
        }

        let stdout = String::from_utf8_lossy(&output.stdout[..]);
        let mid_did: Vec<_> = stdout.trim().split(' ').collect();
        if mid_did.len() != 2 {
            return Err(err_msg(format!(
                "Got unexpected output during info command: {}",
                stdout.trim()
            )));
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
            println!(
                "Unknown and possible unsupported external flash (MID: {}, DID: {})",
                mid, did
            );
        }

        Ok(())
    }

    fn mass_erase(cmd: &mut process::Command) -> Result<(), Error> {
        print!("Starting mass erase, this may take some time... ");
        io::stdout().flush().unwrap();
        let output = cmd.output()?;
        if !output.status.success() {
            println!("");
            return Err(err_msg(
                String::from_utf8_lossy(&output.stderr[..]).into_owned(),
            ));
        }

        println!("Done.");
        Ok(())
    }

    fn sector_erase(cmd: &mut process::Command) -> Result<(), Error> {
        let output = cmd.output()?;
        if !output.status.success() {
            return Err(err_msg(
                String::from_utf8_lossy(&output.stderr[..]).into_owned(),
            ));
        }

        Ok(())
    }

    fn read(cmd: &mut process::Command, io_output: &mut dyn Write) -> Result<(), Error> {
        let mut child = cmd
            .stdout(process::Stdio::piped())
            .stderr(process::Stdio::piped())
            .spawn()?;

        // unwrap is OK since child is created with piped stdout, see above
        io::copy(child.stdout.as_mut().unwrap(), io_output)?;

        let output = child.wait_with_output()?;

        if !output.status.success() {
            return Err(err_msg(
                String::from_utf8_lossy(&output.stderr[..]).into_owned(),
            ));
        }

        Ok(())
    }

    fn write(cmd: &mut process::Command, io_input: &mut dyn Read) -> Result<(), Error> {
        let mut child = cmd
            .stdin(process::Stdio::piped())
            .stderr(process::Stdio::piped())
            .spawn()?;

        // unwrap is OK since child is created with piped stdin, see above
        io::copy(io_input, child.stdin.as_mut().unwrap())?;

        let output = child.wait_with_output()?;

        if !output.status.success() {
            return Err(err_msg(
                String::from_utf8_lossy(&output.stderr[..]).into_owned(),
            ));
        }

        Ok(())
    }
}

fn cli<'a>(matches: &ArgMatches<'a>) -> Result<(), Error> {
    Cli::from_matches(matches)?.run()
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

    cli(&matches).unwrap_or_else(|err| {
        eprintln!("Error: {}", err);
        process::exit(1);
    });
}
