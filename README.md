
# flash-rover

<p align="center">
    <img width="200" alt="flash-rover logo" src="icon.png">
</p>

*flash-rover* is a command line interface tool to read and write data on an
external flash connected to a TI CC13xx/CC26xx device. *flash-rover* accepts
reading and writing both streams of bytes or arbitrary files. The internal flash
on the TI device is also left untouched while *flash-rover* is accessing the
external flash, meaning no need to manually flash the TI device with some
firmware. *flash-rover* supports Windows, Linux and macOS, with binary downloads
available for [every
release](https://github.com/ti-simplelink/flash-rover/releases).

Released under BSD-3-Clause license.

**Disclaimer**: *flash-rover* does not generate the necessary OAD metadata
needed to write OAD images to the external flash, even though a common use of
external flash on SimpleLink devices is OAD. OAD requires specific metadata and
image sectors to be placed in external flash. However, since *flash-rover* is a
generic tool, it does not handle creation of OAD image metadata. This must be
done by the user and some steps may be manual. See the OAD chapter of your Stack
User's Guide for more information.


## Prerequisites

*flash-rover* itself only requires [CCS] installed on your system.

The following TI devices are supported:
* **CC13x0**:
    * [CC1310]
    * [CC1350]
* **CC26x0**:
    * [CC2640]
    * [CC2650]
* **CC26x0R2**:
    * [CC2640R2F]
* **CC13x2/CC26x2**:
    * [CC1312R]
    * [CC1352R]
    * [CC1352P]
    * [CC2642R]
    * [CC2652R]
    * [CC2652RB]

The following hardware requirements for both TI development boards and custom
boards are:
* A 2-pin JTAG connection via a XDS110 debugger to the TI device.
* The external flash is connected to the TI device via SPI.

Currently known supported external flash hardware are:
* Macronix MX25R
* WinBond W25X 

Note that other external flash hardware which are not listed above, but are
functionally compatible, will most likely work with *flash-rover*.


## Usage

Download the correct zip folder for your operating system from the [Releases
page](https://github.com/ti-simplelink/flash-rover/releases) and extract it. Add
the path to the executable to the environment `PATH` variable, or `cd` into the
directory of the executable.

Refer to the help menu of the executable for documentation on the CLI and the
different subcommands:

```bash
$ flash-rover help
$ flash-rover help write
$ flash-rover write --help
```

The CCS path must point to the CCS installation folder. Note that this folder
should contain the `ccs_base/` subfolder.


### Examples

Reading the external flash device information of a CC13x2/CC26x2 LaunchPad:

```bash
$ flash-rover --ccs /path/to/ccs/install/folder \
    --device cc13x2_cc26x2 \
    --xds L4100009 \
    info
Macronix MX25R8035F (MID: 0xC2, DID: 0x14, size: 8.00 MiB)
```

Read the first 10 bytes (offset 0, length 10) of the external flash on a
CC2640R2 LaunchPad and store it in a new file called `output.bin`:

```bash
# You can either stream the output into a file
$ flash-rover --ccs /path/to/ccs/install/folder \
    --device cc26x0r2 \
    --xds L50012SB \
    read 0 10 > output.bin 
# or explicitly specify the output file 
$ flash-rover --ccs /path/to/ccs/install/folder \
    --device cc26x0r2 \
    --xds L50012SB \
    read 0 10 --output output.bin
```

Write an entire input file called `input.bin` to offset 100 of the external
flash on a CC1310 LaunchPad, and erase the sectors before writing. Read the
memory range before and after (printout to stdout) to verify the contents have
changed:

```bash
$ echo "Powered by flash-rover!" > input.bin
$ flash-rover --ccs /path/to/ccs/install/folder \
    --device cc13x0 \
    --xds L200005Z \
    read 100 $(wc -c < input.bin)

$ flash-rover --ccs /path/to/ccs/install/folder \
    --device cc13x0 \
    --xds L200005Z \
    write 100 --erase < input.bin 
$ flash-rover --ccs /path/to/ccs/install/folder \
    --device cc13x0 \
    --xds L200005Z \
    read 100 $(wc -c < input.bin)
Powered by flash-rover!
```


## How it works

*flash-rover* connects to the TI device through the [Debug Server Script
(DSS)][DSS] environment, available through CCS. When connected to the TI device,
*flash-rover* hijacks the CPU by copying over the entire firmware into RAM,
halts the CPU, and resets the execution context of the CPU into the firmware in
RAM. Now, *flash-rover* communicates with the firmware through JTAG via some
dedicated memory address in RAM, being able to send various commands and read
the corresponding response. The firmware is responsible for communicating with
the external flash via SPI.


## Building

It is recommended for customers to download the pre-compiled executable from the
[Releases page](https://github.com/ti-simplelink/flash-rover/releases) rather
than building from source.

The CLI is written in Rust and the device firmware is written in C++. Building
the CLI requires in general the latest stable release of the Rust compiler. See
[rustup] on how to install Rust. There already exists pre-compiled binaries of
the device firmware under `src/assets/fw`, however, building the device firmware
requires CCS version 9.0 or later.

To build *flash-rover* from source:

```bash
$ git clone https://github.com/ti-simplelink/flash-rover
$ cd flash-rover
$ cargo build --release
$ ./target/release/flash-rover --version
flash-rover 0.2.0
```


[rustup]:    https://rustup.rs/
[DSS]:       http://dev.ti.com/tirex/explore/node?node=AO6UKsAhivhxn6EDOzuszQ__FUz-xrs__LATEST
[CCS]:       http://www.ti.com/tool/CCSTUDIO
[CC1310]:    http://www.ti.com/product/CC1310
[CC1350]:    http://www.ti.com/product/CC1350
[CC2640]:    http://www.ti.com/product/CC2640
[CC2650]:    http://www.ti.com/product/CC2650
[CC2640R2F]: http://www.ti.com/product/CC2640R2F
[CC1312R]:   http://www.ti.com/product/CC1312R
[CC1352R]:   http://www.ti.com/product/CC1352R
[CC1352P]:   http://www.ti.com/product/CC1352P
[CC2642R]:   http://www.ti.com/product/CC2642R
[CC2652R]:   http://www.ti.com/product/CC2652R
[CC2652RB]:  http://www.ti.com/product/CC2652RB
