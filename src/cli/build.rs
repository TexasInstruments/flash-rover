
extern crate walkdir;

use std::env;
use std::path::Path;
use std::fs;

use walkdir::WalkDir;

fn copy_dss_folder() {
    let cwd = env::current_dir().unwrap();
    let out_dir = env::var("OUT_DIR").unwrap();

    fs::remove_dir_all(Path::new(&out_dir).join("dss")).unwrap();

    for entry in WalkDir::new("dss") {
        let entry = entry.unwrap();
        let out_entry = Path::new(&out_dir).join(entry.path());
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