// Copyright (c) 2020 , Texas Instruments.
// Licensed under the BSD-3-Clause license
// (see LICENSE or <https://opensource.org/licenses/BSD-3-Clause>) All files in the project
// notice may not be copied, modified, or distributed except according to those terms.

use std::ops;
use std::str;
use std::string;

use snafu::{Backtrace, OptionExt, Snafu};

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("Invalid string when parsing Device: {}", input))]
    InvalidDevice { input: String, backtrace: Backtrace },
    #[snafu(display("Unable to parse SPI pins fom string {}: {}", input, msg))]
    InvalidSpiPins {
        input: String,
        msg: String,
        backtrace: Backtrace,
    },
}

pub type Result<T, E = Error> = std::result::Result<T, E>;

#[allow(non_camel_case_types)]
#[derive(Copy, Clone, Debug)]
pub enum Device {
    CC13x0,
    CC26x0,
    CC26x0R2,
    CC13x2_CC26x2,
}

impl string::ToString for Device {
    fn to_string(&self) -> String {
        match self {
            Device::CC13x0 => "cc13x0",
            Device::CC26x0 => "cc26x0",
            Device::CC26x0R2 => "cc26x0r2",
            Device::CC13x2_CC26x2 => "cc13x2_cc26x2",
        }
        .to_string()
    }
}

impl str::FromStr for Device {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "cc13x0" => Ok(Device::CC13x0),
            "cc26x0" => Ok(Device::CC26x0),
            "cc26x0r2" => Ok(Device::CC26x0R2),
            "cc13x2_cc26x2" => Ok(Device::CC13x2_CC26x2),
            input => InvalidDevice { input }.fail(),
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub enum SpiPin {
    Miso,
    Mosi,
    Clk,
    Csn,
}

pub struct SpiPins(pub [u8; 4]);

impl ops::Index<SpiPin> for SpiPins {
    type Output = u8;

    fn index(&self, pin: SpiPin) -> &Self::Output {
        match pin {
            SpiPin::Miso => &self.0[0],
            SpiPin::Mosi => &self.0[1],
            SpiPin::Clk => &self.0[2],
            SpiPin::Csn => &self.0[3],
        }
    }
}

impl str::FromStr for SpiPins {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut dio_iter = s.split(',').map(|d| {
            str::parse(d).ok().context(InvalidSpiPins {
                input: s,
                msg: "SPI pin value is not an acceptable digit",
            })
        });
        let too_many_pins_context = || InvalidSpiPins {
            input: s,
            msg: "expects 4 values",
        };
        let dios: [_; 4] = [
            dio_iter.next().with_context(too_many_pins_context)??,
            dio_iter.next().with_context(too_many_pins_context)??,
            dio_iter.next().with_context(too_many_pins_context)??,
            dio_iter.next().with_context(too_many_pins_context)??,
        ];
        ensure!(dio_iter.next().is_none(), too_many_pins_context());
        Ok(Self(dios))
    }
}
