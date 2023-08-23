// Copyright (c) 2020 , Texas Instruments.
// Licensed under the BSD-3-Clause license
// (see LICENSE or <https://opensource.org/licenses/BSD-3-Clause>) All files in the project
// notice may not be copied, modified, or distributed except according to those terms.

use clap::{App, AppSettings, Arg, SubCommand};

pub fn app() -> App<'static, 'static> {
    App::new(crate_name!())
        .author(crate_authors!())
        .version(crate_version!())
        .about("Read and write to the external flash on a CC13xx/CC26xx device")
        .max_term_width(100)
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .arg(Arg::with_name("log-dss")
            .help("Log DSS script with a specified log level")
            .long_help(
"Log DSS script with a specified log level to a temporary log file. If a DSS error occurs the log \
file will be saved and the path will be displayed. If OFF is specified then no logging will happen.")
            .long("log-dss")
            .value_name("LEVEL")
            .default_value("OFF")
            .possible_values(&[
                "OFF", 
                "SEVERE", 
                "WARNING", 
                "INFO", 
                "CONFIG", 
                "FINE", 
                "FINER", 
                "FINEST", 
                "ALL",
            ]))
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
            .possible_values(&[
                "cc1310", 
                "cc1312r", 
                "cc1350", 
                "cc1352p", 
                "cc1352r", 
                "cc2640",
                "cc2640r2f",
                "cc2642r",
                "cc2650",
                "cc2652p",
                "cc2652r",
                "cc2652rb",
                "cc1312r7",
                "cc1352p7",
                "cc2652p7",
                "cc2652r7",
                "cc1354p10",
                "cc1314r10"
            ])
            .required(true))
        .arg(Arg::with_name("spi-pins")
            .help("Override default SPI DIOs for external flash access, defaults to DIOs used for external flash on LaunchPads [8,9,10,20]")
            .short("s")
            .long("spi-pins")
            .value_names(&[
                "MISO",
                "MOSI",
                "CLK",
                "CSN"
            ])
            .value_delimiter(",")
            .require_delimiter(true)
            .validator(spi_pins_validate))
        .subcommand(subcommand_info())
        .subcommand(subcommand_erase())
        .subcommand(subcommand_read())
        .subcommand(subcommand_write())
}

fn subcommand_info() -> App<'static, 'static> {
    SubCommand::with_name("info").about("Get external flash device info")
}

fn subcommand_erase() -> App<'static, 'static> {
    SubCommand::with_name("erase")
        .about("Perform erase operation, either on sectors or mass erase")
        .arg(
            Arg::with_name("offset")
                .help("Offset of bytes into external flash device to start erase")
                .value_name("OFFSET")
                .index(1)
                .validator(is_zero_or_positive)
                .required_unless("mass-erase"),
        )
        .arg(
            Arg::with_name("length")
                .help("Length of bytes to erase from offset")
                .value_name("LENGTH")
                .index(2)
                .validator(is_zero_or_positive)
                .required_unless("mass-erase"),
        )
        .arg(
            Arg::with_name("mass-erase")
                .help("Perform mass erase of the entire external flash device")
                .short("m")
                .long("mass-erase")
                .conflicts_with_all(&["offset", "length"]),
        )
}

fn subcommand_read() -> App<'static, 'static> {
    SubCommand::with_name("read")
        .about("Read data from an address range on the external flash")
        .arg(
            Arg::with_name("offset")
                .help("Offset of bytes into external flash device to start read")
                .value_name("OFFSET")
                .index(1)
                .validator(is_zero_or_positive)
                .required(true),
        )
        .arg(
            Arg::with_name("length")
                .help("Length of bytes to read from offset")
                .value_name("LENGTH")
                .index(2)
                .validator(is_zero_or_positive)
                .required(true),
        )
        .arg(
            Arg::with_name("output")
                .help("File to store read data. Will overwrite file. Writes to stdout if omitted.")
                .short("o")
                .long("output")
                .value_name("FILE")
                .takes_value(true),
        )
}

fn subcommand_write() -> App<'static, 'static> {
    SubCommand::with_name("write")
        .about("Write data to an address range on the external flash")
        .arg(
            Arg::with_name("verify")
                .help("Verify the integrity of the written data")
                .long_help(
"Verify that the written data was successfully written by reading back the data from external flash \
and compare. Verification is done per sector basis.")
                .long("verify")
        )
        .arg(
            Arg::with_name("in-place")
                .help("Write in-place in external flash, without ensuring coherency in touched sectors")
                .long_help(
"By default all data in the write address range will be overwritten, and all touched sectors are \
ensured to retain the data outside the write address range. By writing in-place, data is written to \
the external flash without first erasing touched sectors. Note that when writing in-place, reading \
back the same address range may yield different data than initially written.")
                .short("p")
                .long("in-place")
        )
        .arg(
            Arg::with_name("offset")
                .help("Offset of bytes into external flash device to start write")
                .value_name("OFFSET")
                .index(1)
                .validator(is_zero_or_positive)
                .required(true),
        )
        .arg(
            Arg::with_name("length")
                .help("Length of bytes to write from offset")
                .value_name("LENGTH")
                .index(2)
                .validator(is_zero_or_positive),
        )
        .arg(
            Arg::with_name("input")
                .help("File to read contents of data to write. Reads from stdin if omitted.")
                .short("i")
                .long("input")
                .value_name("FILE")
                .takes_value(true),
        )
}

fn spi_pins_validate(dio: String) -> Result<(), String> {
    type ParsedSpiPin = u8;

    if dio.parse::<ParsedSpiPin>().is_err() {
        return Err(format!(
            "Invalid DIO value {}, must be an unsigned 8-bit integer",
            dio
        ));
    }

    Ok(())
}

fn is_zero_or_positive(val: String) -> Result<(), String> {
    if val.parse::<u32>().is_err() {
        return Err(String::from("Value must be a zero or positive integer"));
    }

    Ok(())
}
