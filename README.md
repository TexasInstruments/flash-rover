
# flash-rover

<p align="center">
    <img width="200" alt="flash-rover logo" src="icon.png">
</p>

*flash-rover* is a command line interface tool to read and write data on an
external flash connected to a TI CC13xx/CC26xx device. *flash-rover* accepts
reading and writing both streams of bytes or arbitrary files. The internal flash
on the TI device is also left untouched while *flash-rover* is accessing the
external flash. *flash-rover* supports Windows, macOS and Linux, with binary
downloads available for [every
release](https://github.com/ti-simplelink/flash-rover/releases).

Released under BSD-3-Clause license.


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

The following hardware requirements for both TI development boards and custom
boards are:
* A 2-pin JTAG connection via a XDS110 debugger to the TI device.
* The external flash is connected to the TI device through SPI.

Currently known supported external flash hardware are:
* Macronix MX25R1635F
* Macronix MX25R8035F
* WinBond W25X40CL
* WinBond W25X20CL

Note that other external flash hardware which are not listed above, but are
functionally compatible, will most likely work with *flash-rover*.


## Usage

It is assumed *flash-rover* is configured in your `PATH`.

The CCS path must point to the CCS installation folder. Note that this folder
should contain the `ccs_base/` subfolder.

Reading the external flash device information of a CC13x2/CC26x2 LaunchPad:

```bash
$ flash-rover --ccs /path/to/ccs/install/folder \
    --device cc13x2_cc26x2 \
    --xds L4100009 \
    info
Macronix MX25R8035F (MID: 0xC2, DID: 0x14, size: 1024.00 KiB)
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


## Building

The CLI portion of *flash-rover* is written in Rust, the JTAG interaction is
written in DSS, and the device firmware is written in C++. Building the CLI
requires in general the latest stable release of the Rust compiler. Building the
device firmware requires CCS version 9.0 or later.

To build *flash-rover*:

```bash
$ git clone https://github.com/ti-simplelink/flash-rover
$ cd flash-rover
$ CCS_ROOT=<path/to/ccs> ./ci/firmware.sh
$ ./ci/install.sh
$ ./output/flash-rover/flash-rover --version
flash-rover 0.1.1
```

The `CCS_ROOT` variable should point to your CCS installation folder, which
should contain the sub-folder `ccs_base/`.


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
