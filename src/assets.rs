// Copyright (c) 2020 , Texas Instruments.
// Licensed under the BSD-3-Clause license
// (see LICENSE or <https://opensource.org/licenses/BSD-3-Clause>) All files in the project
// notice may not be copied, modified, or distributed except according to those terms.

use std::borrow::Cow;

use rust_embed::RustEmbed;

use crate::types::{Device, DeviceFamily};

#[derive(RustEmbed)]
#[folder = "./src/assets"]
struct Asset;

pub fn get_ccxml_template(device: Device) -> Option<Cow<'static, [u8]>> {
    use DeviceFamily::*;

    const PATH: &str = "ccxml/";

    let device_family: DeviceFamily = From::from(device);
    let file = match device_family {
        CC13x0 => "template_cc13x0.ccxml",
        CC26x0 => "template_cc26x0.ccxml",
        CC26x0R2 => "template_cc26x0r2.ccxml",
        CC13x2_CC26x2 | CC13x2x7_CC26x2x7 => "template_cc13x2_cc26x2.ccxml",
        CC13x4_CC26x4 => "template_cc13x4_cc26x4.ccxml",
    };
    Asset::get(format!("{}{}", PATH, file).as_str())
}

pub fn get_firmware(device: Device) -> Option<Cow<'static, [u8]>> {
    use DeviceFamily::*;

    const PATH: &str = "fw/";

    let device_family: DeviceFamily = From::from(device);
    let file = match device_family {
        CC13x0 => "cc13x0.bin",
        CC26x0 => "cc26x0.bin",
        CC26x0R2 => "cc26x0r2.bin",
        CC13x2_CC26x2 => "cc13x2_cc26x2.bin",
        CC13x2x7_CC26x2x7 => "cc13x2x7_cc26x2x7.bin",
        CC13x4_CC26x4 => "cc13x4_cc26x4.bin",
    };
    Asset::get(format!("{}{}", PATH, file).as_str())
}
