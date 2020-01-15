// Copyright (c) 2019 , Texas Instruments.
// Licensed under the BSD-3-Clause license
// (see LICENSE or <https://opensource.org/licenses/BSD-3-Clause>) All files in the project
// notice may not be copied, modified, or distributed except according to those terms.

use std::fmt;

use byte_unit::Byte;

pub struct XflashInfo {
    manufacturer_id: u32,
    device_id: u32,
    size: u32,
    name: &'static str,
}

const SUPPORTED_XFLASH_HW: &[XflashInfo] = &[
    // Macronix
    XflashInfo {
        manufacturer_id: 0xC2,
        device_id: 0x17,
        size: 0x0400_0000,
        name: "Macronix MX25R6435F",
    },
    XflashInfo {
        manufacturer_id: 0xC2,
        device_id: 0x16,
        size: 0x0200_0000,
        name: "Macronix MX25R3235F",
    },
    XflashInfo {
        manufacturer_id: 0xC2,
        device_id: 0x15,
        size: 0x0100_0000,
        name: "Macronix MX25R1635F",
    },
    XflashInfo {
        manufacturer_id: 0xC2,
        device_id: 0x14,
        size: 0x0080_0000,
        name: "Macronix MX25R8035F",
    },
    XflashInfo {
        manufacturer_id: 0xC2,
        device_id: 0x13,
        size: 0x0040_0000,
        name: "Macronix MX25R4035F",
    },
    XflashInfo {
        manufacturer_id: 0xC2,
        device_id: 0x12,
        size: 0x0020_0000,
        name: "Macronix MX25R2035F",
    },
    XflashInfo {
        manufacturer_id: 0xC2,
        device_id: 0x11,
        size: 0x0010_0000,
        name: "Macronix MX25R1035F",
    },
    XflashInfo {
        manufacturer_id: 0xC2,
        device_id: 0x10,
        size: 0x0008_0000,
        name: "Macronix MX25R512F",
    },
    // WinBond
    XflashInfo {
        manufacturer_id: 0xEF,
        device_id: 0x12,
        size: 0x0040_0000,
        name: "WinBond W25X40CL",
    },
    XflashInfo {
        manufacturer_id: 0xEF,
        device_id: 0x11,
        size: 0x0020_0000,
        name: "WinBond W25X20CL",
    },
    XflashInfo {
        manufacturer_id: 0xEF,
        device_id: 0x10,
        size: 0x0010_0000,
        name: "WinBond W25X10CL",
    },
    XflashInfo {
        manufacturer_id: 0xEF,
        device_id: 0x05,
        size: 0x0008_0000,
        name: "WinBond W25X05CL",
    },
];

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

impl XflashInfo {
    pub fn find(mid: u32, did: u32) -> Option<&'static XflashInfo> {
        SUPPORTED_XFLASH_HW
            .iter()
            .find(|i| i.manufacturer_id == mid && i.device_id == did)
    }
}
