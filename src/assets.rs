// Copyright (c) 2020 , Texas Instruments.
// Licensed under the BSD-3-Clause license
// (see LICENSE or <https://opensource.org/licenses/BSD-3-Clause>) All files in the project
// notice may not be copied, modified, or distributed except according to those terms.

use std::borrow::Cow;

use rust_embed::RustEmbed;

use crate::types::Device;

#[derive(RustEmbed)]
#[folder = "src/assets"]
struct Asset;

pub fn get_ccxml_template(device: &Device) -> Option<Cow<'static, [u8]>> {
    const PATH: &str = "ccxml/";
    let file = match device {
        Device::CC13x0 => "template_cc13x0.ccxml",
        Device::CC26x0 => "template_cc26x0.ccxml",
        Device::CC26x0R2 => "template_cc26x0r2.ccxml",
        Device::CC13x2_CC26x2 => "template_cc13x2_cc26x2.ccxml",
    };
    Asset::get(format!("{}{}", PATH, file).as_str())
}

pub fn get_firmware(device: &Device) -> Option<Cow<'static, [u8]>> {
    const PATH: &str = "fw/";
    let file = match device {
        Device::CC13x0 => "cc13x0.bin",
        Device::CC26x0 => "cc26x0.bin",
        Device::CC26x0R2 => "cc26x0r2.bin",
        Device::CC13x2_CC26x2 => "cc13x2_cc26x2.bin",
    };
    Asset::get(format!("{}{}", PATH, file).as_str())
}
