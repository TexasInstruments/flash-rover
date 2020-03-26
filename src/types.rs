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
    CC1310,
    CC1312R,
    CC1350,
    CC1352P,
    CC1352R,
    CC2640,
    CC2640R2F,
    CC2642R,
    CC2650,
    CC2652P,
    CC2652R,
    CC2652RB,
}

impl Device {
    pub fn ccxml_desc(&self) -> &str {
        use Device::*;

        match self {
            CC1310 => "CC1310F128",
            CC1312R => "CC1312R1F3",
            CC1350 => "CC1350F128",
            CC1352P => "CC1352P1F3",
            CC1352R => "CC1352R1F3",
            CC2640 => "CC2640F128",
            CC2640R2F => "CC2640R2F",
            CC2642R => "CC2642R1F",
            CC2650 => "CC2650F128",
            CC2652P => "CC2652P1F",
            CC2652R => "CC2652R1F",
            CC2652RB => "CC2652RB1F",
        }
    }

    pub fn ccxml_id(&self) -> &str {
        // Currently all devices have the same "desc" and "id" values
        self.ccxml_desc()
    }

    pub fn ccxml_xml(&self) -> &str {
        use Device::*;

        match self {
            CC1310 => "cc1310f128.xml",
            CC1312R => "cc1312r1f3.xml",
            CC1350 => "cc1350f128.xml",
            CC1352P => "cc1352p1f3.xml",
            CC1352R => "cc1352r1f3.xml",
            CC2640 => "cc2640f128.xml",
            CC2640R2F => "cc2640r2f.xml",
            CC2642R => "cc2642r1f.xml",
            CC2650 => "cc2650f128.xml",
            CC2652P => "cc2652p1f.xml",
            CC2652R => "cc2652r1f.xml",
            CC2652RB => "cc2652rb1f.xml",
        }
    }
}

impl string::ToString for Device {
    fn to_string(&self) -> String {
        use Device::*;

        match self {
            CC1310 => "cc1310",
            CC1312R => "cc1312r",
            CC1350 => "cc1350",
            CC1352P => "cc1352p",
            CC1352R => "cc1352r",
            CC2640 => "cc2640",
            CC2640R2F => "cc2640r2f",
            CC2642R => "cc2642r",
            CC2650 => "cc2650",
            CC2652P => "cc2652p",
            CC2652R => "cc2652r",
            CC2652RB => "cc2652rb",
        }
        .to_string()
    }
}

impl str::FromStr for Device {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use Device::*;

        match s {
            "cc1310" => Ok(CC1310),
            "cc1312r" => Ok(CC1312R),
            "cc1350" => Ok(CC1350),
            "cc1352p" => Ok(CC1352P),
            "cc1352r" => Ok(CC1352R),
            "cc2640" => Ok(CC2640),
            "cc2640r2f" => Ok(CC2640R2F),
            "cc2642r" => Ok(CC2642R),
            "cc2650" => Ok(CC2650),
            "cc2652p" => Ok(CC2652P),
            "cc2652r" => Ok(CC2652R),
            "cc2652rb" => Ok(CC2652RB),
            input => InvalidDevice { input }.fail(),
        }
    }
}

#[allow(non_camel_case_types)]
#[derive(Copy, Clone, Debug)]
pub enum DeviceFamily {
    CC13x0,
    CC26x0,
    CC26x0R2,
    CC13x2_CC26x2,
}

impl From<Device> for DeviceFamily {
    fn from(device: Device) -> Self {
        use Device::*;
        use DeviceFamily::*;

        match device {
            CC1310 | CC1350 => CC13x0,
            CC2640 | CC2650 => CC26x0,
            CC2640R2F => CC26x0R2,
            CC1312R | CC1352P | CC1352R | CC2642R | CC2652P | CC2652R | CC2652RB => CC13x2_CC26x2,
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

#[derive(Copy, Clone, Debug)]
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
