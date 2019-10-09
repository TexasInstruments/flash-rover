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
            std::fs::create_dir(out_entry).unwrap();
        } else if entry.path().is_file() {
            std::fs::copy(entry.path(), out_entry).unwrap();
        }
    }
}

fn main() {
    copy_dss_folder();
}
