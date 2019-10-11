// Copyright (c) 2019 , Texas Instruments.
// Licensed under the BSD-3-Clause license
// (see LICENSE or <https://opensource.org/licenses/BSD-3-Clause>) All files in the project
// notice may not be copied, modified, or distributed except according to those terms.

extern crate walkdir;

use std::env;
use std::fs;
use std::path::Path;

use walkdir::WalkDir;

fn copy_dss_folder() {
    let out_dir = env::var("OUT_DIR").unwrap();
    let target_dir = Path::new(&out_dir).join("../../..");

    fs::remove_dir_all(target_dir.join("dss")).unwrap_or_default();

    for entry in WalkDir::new("dss") {
        let entry = entry.unwrap();
        let out_entry = target_dir.join(entry.path());
        if entry.path().is_dir() {
            fs::create_dir(out_entry).unwrap();
        } else if entry.path().is_file() {
            fs::copy(entry.path(), out_entry).unwrap();
        }
    }

    let fw_dir = Path::new(&out_dir).join("../../../dss/fw");
    let ccs_dir = Path::new("../fw/workspace");
    let fws = &[
        ccs_dir.join("flash_rover_fw_cc13x0_gcc/Firmware/cc13x0.bin"),
        ccs_dir.join("flash_rover_fw_cc26x0_gcc/Firmware/cc26x0.bin"),
        ccs_dir.join("flash_rover_fw_cc26x0r2_gcc/Firmware/cc26x0r2.bin"),
        ccs_dir.join("flash_rover_fw_cc13x2_cc26x2_gcc/Firmware/cc13x2_cc26x2.bin"),
    ];

    fs::create_dir(&fw_dir).unwrap();
    for fw in fws {
        fs::copy(fw, fw_dir.join(fw.file_name().unwrap())).unwrap();
    }
}

fn main() {
    copy_dss_folder();
}
