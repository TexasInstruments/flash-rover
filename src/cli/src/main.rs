#[macro_use]
extern crate clap;
extern crate serial;

#[macro_use]
extern crate failure;

use std::cmp;
use std::fs::File;
use std::io;
use std::io::{Read, Write};
use std::process;
use std::time::Duration;

use clap::{App, Arg, ArgGroup, ArgMatches, SubCommand};
use serial::prelude::*;

use failure::{err_msg, Error};

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
        io: Box<Write>,
    },
    Write {
        offset: u32,
        length: Option<u32>,
        io: Box<Read>,
    },
}

enum SerialCmd {
    Synchronize,
    FlashInfo,
    SectorErase {
        offset: u32,
        length: u32,
    },
    MassErase,
    Read {
        offset: u32,
        length: u32,
    },
    StartWrite,
    DataWrite {
        offset: u32,
        length: u32,
        data: Vec<u8>,
    },
}

#[derive(Debug)]
enum SerialRsp {
    Ack,
    AckPend,
    FlashInfo {
        manf_id: u8,
        dev_id: u8,
        dev_size: u32,
    },
    WriteSize {
        length: u32,
    },
    DataRead {
        offset: u32,
        length: u32,
        data: Vec<u8>,
    },
}

#[derive(Debug)]
enum SerialError {
    Generic,
    Serial,
    ExtFlash,
    Unsupported,
    AddressRange,
    BufferOverflow,
}

const SERIAL_SETTINGS: serial::PortSettings = serial::PortSettings {
    baud_rate: serial::Baud115200,
    char_size: serial::Bits8,
    parity: serial::ParityNone,
    stop_bits: serial::Stop1,
    flow_control: serial::FlowNone,
};

impl SerialCmd {
    const START_OP: u8 = 0xEF;

    fn type_int(&self) -> u8 {
        match self {
            SerialCmd::Synchronize => 0xC0,
            SerialCmd::FlashInfo => 0xC1,
            SerialCmd::SectorErase { .. } => 0xC2,
            SerialCmd::MassErase => 0xC3,
            SerialCmd::Read { .. } => 0xC4,
            SerialCmd::StartWrite => 0xC5,
            SerialCmd::DataWrite { .. } => 0xC6,
        }
    }

    fn into_bytes(mut self) -> Vec<u8> {
        let mut bytes = vec![Self::START_OP, self.type_int()];
        match self {
            SerialCmd::Synchronize => {}
            SerialCmd::FlashInfo => {}
            SerialCmd::SectorErase {
                offset: o,
                length: l,
            } => {
                bytes.extend_from_slice(&o.to_le_bytes());
                bytes.extend_from_slice(&l.to_le_bytes());
            }
            SerialCmd::MassErase => {}
            SerialCmd::Read {
                offset: o,
                length: l,
            } => {
                bytes.extend_from_slice(&o.to_le_bytes());
                bytes.extend_from_slice(&l.to_le_bytes());
            }
            SerialCmd::StartWrite => {}
            SerialCmd::DataWrite {
                offset: o,
                length: l,
                data: ref mut d,
            } => {
                bytes.extend_from_slice(&o.to_le_bytes());
                bytes.extend_from_slice(&l.to_le_bytes());
                bytes.append(d)
            }
        }
        bytes
    }
}

fn send_cmd(port: &mut SerialPort, cmd: SerialCmd) -> io::Result<()> {
    let cmd_bytes = cmd.into_bytes();
    port.write(&cmd_bytes[..])?;
    Ok(())
}

fn recv_rsp(port: &mut SerialPort) -> io::Result<Result<SerialRsp, SerialError>> {
    let mut read_buf = [0; 2];
    port.read_exact(&mut read_buf)?;

    if read_buf[0] != 0xEF {
        return Ok(Err(SerialError::Serial));
    }

    match read_buf[1] {
        0x01 => Ok(Ok(SerialRsp::Ack)),
        0x02 => Ok(Ok(SerialRsp::AckPend)),
        0x03 => {
            let mut manf_id_buf = [0; 1];
            let mut dev_id_buf = [0; 1];
            let mut dev_size_buf = [0; 4];
            port.read_exact(&mut manf_id_buf)?;
            port.read_exact(&mut dev_id_buf)?;
            port.read_exact(&mut dev_size_buf)?;
            Ok(Ok(SerialRsp::FlashInfo {
                manf_id: manf_id_buf[0],
                dev_id: dev_id_buf[0],
                dev_size: u32::from_le_bytes(dev_size_buf),
            }))
        }
        0x04 => {
            let mut length_buf = [0; 4];
            port.read_exact(&mut length_buf)?;
            Ok(Ok(SerialRsp::WriteSize {
                length: u32::from_le_bytes(length_buf),
            }))
        }
        0x05 => {
            let mut offset_buf = [0; 4];
            let mut length_buf = [0; 4];
            port.read_exact(&mut offset_buf)?;
            port.read_exact(&mut length_buf)?;
            let offset = u32::from_le_bytes(offset_buf);
            let length = u32::from_le_bytes(length_buf);
            let mut data = vec![0; length as usize];
            port.read_exact(&mut data[..])?;
            Ok(Ok(SerialRsp::DataRead {
                offset: offset,
                length: length,
                data: data,
            }))
        }
        0x80 => Ok(Err(SerialError::Generic)),
        0x81 => Ok(Err(SerialError::Serial)),
        0x82 => Ok(Err(SerialError::ExtFlash)),
        0x83 => Ok(Err(SerialError::Unsupported)),
        0x84 => Ok(Err(SerialError::AddressRange)),
        0x85 => Ok(Err(SerialError::BufferOverflow)),
        _ => Ok(Err(SerialError::Generic)),
    }
}

fn run(port_name: &str, cmd: Command) -> Result<(), Error> {
    let mut port = serial::open(port_name).unwrap();
    port.configure(&SERIAL_SETTINGS).unwrap();
    port.set_timeout(Duration::from_secs(1)).unwrap();

    send_cmd(&mut port, SerialCmd::Synchronize)?;
    let rsp = recv_rsp(&mut port)?;
    match rsp {
        Ok(SerialRsp::Ack) => {}
        _ => {}
    };

    match cmd {
        Command::Info => {
            send_cmd(&mut port, SerialCmd::FlashInfo)?;
            match recv_rsp(&mut port)? {
                Ok(SerialRsp::FlashInfo {
                    manf_id,
                    dev_id,
                    dev_size,
                }) => {
                    println!("External flash info - Manufacturer ID: 0x{:X}, Device ID: 0x{:X}, Flash Size: {} bytes", manf_id, dev_id, dev_size);
                    return Ok(());
                }
                _ => unimplemented!(),
            }
        }
        Command::MassErase => {
            send_cmd(&mut port, SerialCmd::MassErase)?;
            match recv_rsp(&mut port)? {
                Ok(SerialRsp::AckPend) => {}
                _ => return Err(err_msg("mass erase, ack pend misbehave")),
            };
            loop {
                match recv_rsp(&mut port) {
                    Ok(rsp) => match rsp {
                        Ok(SerialRsp::Ack) => return Ok(()),
                        _ => return Err(err_msg("mass erase, ack misbehave")),
                    },
                    Err(err) => match err.kind() {
                        io::ErrorKind::TimedOut => {}
                        _ => return Err(err)?,
                    },
                };
            }
        }
        Command::SectorErase { offset, length } => {
            send_cmd(&mut port, SerialCmd::SectorErase { offset, length })?;
            match recv_rsp(&mut port)? {
                Ok(SerialRsp::AckPend) => {}
                _ => return Err(err_msg("sector erase, ack pend misbehave")),
            };
            loop {
                match recv_rsp(&mut port) {
                    Ok(rsp) => match rsp {
                        Ok(SerialRsp::Ack) => return Ok(()),
                        _ => return Err(err_msg("sector erase, ack misbehave")),
                    },
                    Err(err) => match err.kind() {
                        io::ErrorKind::TimedOut => {}
                        _ => return Err(err)?,
                    },
                };
            }
        }
        Command::Read {
            offset,
            length,
            mut io,
        } => {
            send_cmd(&mut port, SerialCmd::Read { offset, length })?;
            match recv_rsp(&mut port)? {
                Ok(SerialRsp::AckPend) => {}
                _ => return Err(err_msg("read, ack pend misbehave")),
            };

            loop {
                match recv_rsp(&mut port) {
                    Ok(rsp) => match rsp {
                        Ok(SerialRsp::Ack) => return Ok(()),
                        Ok(SerialRsp::DataRead {
                            offset: _,
                            length: _,
                            data,
                        }) => {
                            io.write(&data)?;
                        }
                        _ => return Err(err_msg("read, ack misbehave")),
                    },
                    Err(err) => match err.kind() {
                        io::ErrorKind::TimedOut => {}
                        _ => return Err(err)?,
                    },
                };
            }
        }
        Command::Write {
            mut offset,
            length,
            mut io,
        } => {
            let mut buf = Vec::new();
            if let Some(length_cap) = length {
                io.take(length_cap as u64).read_to_end(&mut buf)?;
            } else {
                io.read_to_end(&mut buf)?;
            };

            let mut buf_length = buf.len();
            let mut buf_offset = 0;
            if buf_length == 0 {
                return Ok(());
            }

            send_cmd(&mut port, SerialCmd::StartWrite)?;
            let write_size = match recv_rsp(&mut port)? {
                Ok(SerialRsp::WriteSize { length }) => length as usize,
                _ => return Err(err_msg("write, write size misbehave")),
            };

            loop {
                let ilength = cmp::min(write_size, buf_length);

                send_cmd(
                    &mut port,
                    SerialCmd::DataWrite {
                        offset: offset,
                        length: ilength as u32,
                        data: (&buf[buf_offset..ilength]).to_vec(),
                    },
                )?;
                match recv_rsp(&mut port)? {
                    Ok(SerialRsp::AckPend) => {}
                    _ => return Err(err_msg("write, ack pend misbehave")),
                };

                loop {
                    match recv_rsp(&mut port) {
                        Ok(rsp) => match rsp {
                            Ok(SerialRsp::Ack) => break,
                            _ => return Err(err_msg("write, ack misbehave")),
                        },
                        Err(err) => match err.kind() {
                            io::ErrorKind::TimedOut => {}
                            _ => return Err(err)?,
                        },
                    };
                }

                buf_length -= ilength;
                offset += ilength as u32;
                buf_offset += ilength;
                if buf_length == 0 {
                    return Ok(());
                }
            }
        }
    };
}

fn create_cmd<'a>(matches: &ArgMatches<'a>) -> Result<Command, Error> {
    match matches.subcommand() {
        ("info", _) => Ok(Command::Info),
        ("erase", Some(erase_matches)) => {
            if erase_matches.is_present("mass-erase") {
                Ok(Command::MassErase)
            } else {
                Ok(Command::SectorErase {
                    // unwrap is OK since offset and length are required
                    offset: erase_matches
                        .value_of("offset")
                        .unwrap()
                        .parse::<u32>()
                        .map_err(|_| err_msg("invalid digit for argument 'offset'"))?,
                    length: erase_matches
                        .value_of("length")
                        .unwrap()
                        .parse::<u32>()
                        .map_err(|_| err_msg("invalid digit for argument 'length'"))?,
                })
            }
        }
        ("read", Some(read_matches)) => {
            Ok(Command::Read {
                // unwrap is OK since offset and length are required
                offset: read_matches
                    .value_of("offset")
                    .unwrap()
                    .parse::<u32>()
                    .map_err(|_| err_msg("invalid digit for argument 'offset'"))?,
                length: read_matches
                    .value_of("length")
                    .unwrap()
                    .parse::<u32>()
                    .map_err(|_| err_msg("invalid digit for argument 'length'"))?,
                io: if let Some(output_path) = read_matches.value_of("output") {
                    Box::new(File::create(output_path).unwrap())
                } else {
                    Box::new(io::stdout())
                },
            })
        }
        ("write", Some(write_matches)) => {
            Ok(Command::Write {
                // Only unwrap on offset is OK since length is not required
                offset: write_matches
                    .value_of("offset")
                    .unwrap()
                    .parse::<u32>()
                    .map_err(|_| err_msg("invalid digit for argument 'offset'"))?,
                length: if let Some(length) = write_matches.value_of("length") {
                    Some(length.parse::<u32>()?)
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
        (scmd, _) => Err(err_msg(format!("unsupported subcommand {}", scmd))),
    }
}

fn main() {
    let matches = App::new(crate_name!())
        .version(crate_version!())
        .author(crate_authors!())
        .about("Read and write to external flash")
        .arg(Arg::with_name("xds-id")
            .short("i")
            .long("xds-id")
            .value_name("ID")
            //.required(true)
            .help("The XDS ID of the debugger connected to the device, e.g. L4100847"))
        .arg(Arg::with_name("serial-port")
            .short("s")
            .long("serial-port")
            .value_name("PORT")
            .required(true)
            .help("Data serial port of the device"))
        .arg(Arg::with_name("ccs-root")
            .short("c")
            .long("ccs-root")
            .value_name("PATH")
            .env("CCS_ROOT")
            .help("Path to where CCS installed"))
        .subcommand(SubCommand::with_name("info")
            .about("Get external flash device info")    
        )
        .subcommand(SubCommand::with_name("erase")
            .about("Perform erase operation, either on sectors or mass erase")
            .arg(Arg::with_name("offset")
                .help("Offset of bytes into external flash device to start erase")
                .index(1))
            .arg(Arg::with_name("length")
                .help("Length of bytes to erase from offset")
                .index(2))
            .group(ArgGroup::with_name("erase-sector")
                .args(&["offset", "length"])
                .requires_all(&["offset", "length"])
                .multiple(true))
            .arg(Arg::with_name("mass-erase")
                .short("m")
                .long("mass-erase")
                .help("Perform mass erase of the entire external flash device")
                .conflicts_with("erase-sector"))
        )
        .subcommand(SubCommand::with_name("read")
            .about("Read data from an address range on the external flash")
            .arg(Arg::with_name("offset")
                .help("Offset of bytes into external flash device to start read")
                .index(1)
                .required(true))
            .arg(Arg::with_name("length")
                .help("Length of bytes to read from offset")
                .index(2)
                .required(true))
            .arg(Arg::with_name("output")
                .short("o")
                .long("output")
                .value_name("FILE")
                .help("File to store contents of read data. Prints to stdout if not specified.")
                .takes_value(true))
        )
        .subcommand(SubCommand::with_name("write")
            .about("Write data to an address range on the external flash")
            .arg(Arg::with_name("offset")
                .help("Offset of bytes into external flash device to start write")
                .index(1)
                .required(true))
            .arg(Arg::with_name("length")
                .help("Length of bytes to write from offset")
                .index(2))
            .arg(Arg::with_name("input")
                .short("i")
                .long("input")
                .value_name("FILE")
                .help("File to read contents of write data. Reads from stdin if not specified.")
                .takes_value(true))
        )
        .get_matches();

    if matches.subcommand_name().is_none() {
        eprintln!("{}", matches.usage());
        process::exit(1);
    }

    // let ccs_root = Path::new(matches.value_of("ccs-root").unwrap_or_else(||
    //     clap::Error::with_description("Unable to locate CCS installation path. Please specify argument --ccs-root or set environment variable $CCS_ROOT.",
    //                                   clap::ErrorKind::EmptyValue)
    //         .exit()
    // ));
    // if !ccs_root.is_dir() {
    //     clap::Error::with_description("Provided CCS root is not a valid path", clap::ErrorKind::InvalidValue)
    //         .exit();
    // }

    // This is OK since xds-id is required
    //let xds_id = matches.value_of("xds-id").unwrap();

    // println!("CCS root: {:?}", ccs_root);
    // println!("XDS ID: {}", xds_id);

    let serial_port = matches.value_of("serial-port").unwrap();

    create_cmd(&matches)
        .map(|cmd| run(serial_port, cmd))
        .unwrap_or_else(|err| {
            eprintln!("error: {}", err);
            process::exit(1);
        })
        .unwrap();
}
