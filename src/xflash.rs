// Copyright (c) 2020 , Texas Instruments.
// Licensed under the BSD-3-Clause license
// (see LICENSE or <https://opensource.org/licenses/BSD-3-Clause>) All files in the project
// notice may not be copied, modified, or distributed except according to those terms.

use std::fmt;

use byte_unit::Byte;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct XflashId {
    mid: u32,
    did: u32,
}

#[derive(Clone, Copy, Debug)]
pub struct XflashInfo {
    name: &'static str,
    size: u32,
}

#[derive(Clone, Copy, Debug)]
pub enum Xflash {
    Known(XflashId, XflashInfo),
    Unknown(XflashId),
}

const SUPPORTED_HW: &[Xflash] = &[
    // Macronix
    Xflash::Known(
        XflashId {
            mid: 0xC2,
            did: 0x17,
        },
        XflashInfo {
            name: "Macronix MX25R6435F",
            size: 0x0400_0000,
        },
    ),
    Xflash::Known(
        XflashId {
            mid: 0xC2,
            did: 0x16,
        },
        XflashInfo {
            name: "Macronix MX25R3235F",
            size: 0x0200_0000,
        },
    ),
    Xflash::Known(
        XflashId {
            mid: 0xC2,
            did: 0x15,
        },
        XflashInfo {
            name: "Macronix MX25R1635F",
            size: 0x0100_0000,
        },
    ),
    Xflash::Known(
        XflashId {
            mid: 0xC2,
            did: 0x14,
        },
        XflashInfo {
            name: "Macronix MX25R8035F",
            size: 0x0080_0000,
        },
    ),
    Xflash::Known(
        XflashId {
            mid: 0xC2,
            did: 0x13,
        },
        XflashInfo {
            name: "Macronix MX25R4035F",
            size: 0x0040_0000,
        },
    ),
    Xflash::Known(
        XflashId {
            mid: 0xC2,
            did: 0x12,
        },
        XflashInfo {
            name: "Macronix MX25R2035F",
            size: 0x0020_0000,
        },
    ),
    Xflash::Known(
        XflashId {
            mid: 0xC2,
            did: 0x11,
        },
        XflashInfo {
            name: "Macronix MX25R1035F",
            size: 0x0010_0000,
        },
    ),
    Xflash::Known(
        XflashId {
            mid: 0xC2,
            did: 0x10,
        },
        XflashInfo {
            name: "Macronix MX25R512F",
            size: 0x0008_0000,
        },
    ),
    // WinBond
    Xflash::Known(
        XflashId {
            mid: 0xEF,
            did: 0x12,
        },
        XflashInfo {
            name: "WinBond W25X40CL",
            size: 0x0040_0000,
        },
    ),
    Xflash::Known(
        XflashId {
            mid: 0xEF,
            did: 0x11,
        },
        XflashInfo {
            name: "WinBond W25X20CL",
            size: 0x0020_0000,
        },
    ),
    Xflash::Known(
        XflashId {
            mid: 0xEF,
            did: 0x10,
        },
        XflashInfo {
            name: "WinBond W25X10CL",
            size: 0x0010_0000,
        },
    ),
    Xflash::Known(
        XflashId {
            mid: 0xEF,
            did: 0x05,
        },
        XflashInfo {
            name: "WinBond W25X05CL",
            size: 0x0008_0000,
        },
    ),
];

impl fmt::Display for Xflash {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Xflash::Known(id, info) => write!(
                f,
                "{}, {} (MID: 0x{:X}, DID: 0x{:X})",
                info.name,
                Byte::from_bytes(info.size as u128)
                    .get_appropriate_unit(true)
                    .to_string(),
                id.mid,
                id.did,
            ),
            Xflash::Unknown(id) => write!(
                f,
                "Unknown external flash (MID: 0x{:X}, DID: 0x{:X})",
                id.mid, id.did,
            ),
        }
    }
}

impl Xflash {
    pub fn from_id(mid: u32, did: u32) -> Self {
        let id = XflashId { mid, did };
        SUPPORTED_HW
            .iter()
            .find(|xflash| match xflash {
                Xflash::Known(maybe_id, _) => &id == maybe_id,
                _ => false,
            })
            .map(|x| *x)
            .unwrap_or(Xflash::Unknown(id))
    }
}
