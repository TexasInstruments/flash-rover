
use std::env;
use std::io;

use libloading::{Library, Symbol};
use snafu::{Backtrace, ResultExt, Snafu};
use xflash_args::Command;

#[derive(Debug, Snafu)]
pub enum Error {
    LibloadingError {
        source: io::Error,
        backtrace: Backtrace,
    },
    XflashRunError {
        source: Box<dyn std::error::Error>,
        backtrace: Backtrace,
    }
}

pub type Result<T, E = Error> = std::result::Result<T, E>;

pub struct LibXflash {
    lib: Library,
}

impl LibXflash {
    pub fn new() -> Result<Self> {
        let lib_path = env::current_exe()
            .context(LibloadingError {})?
            .parent()
            .unwrap()
            .join(LibXflash::filename());

        let lib = Library::new(lib_path.as_path()).context(LibloadingError {})?;

        Ok(Self {
            lib
        })
    }

    pub fn run(self, command: Command) -> Result<()> {
        let run: Symbol<fn(Command) -> Result<(), Box<dyn std::error::Error>>> = unsafe {
            self.lib.get(b"run")
                .expect("Unable to find libxflash entry point symbol")
        };

        run(command).context(XflashRunError {})
    }

    fn filename() -> &'static str {
        match () {
            #[cfg(target_os = "windows")]
            () => "xflash.dll",
            #[cfg(target_os = "linux")]
            () => "libxflash.so",
            #[cfg(target_os = "macos")]
            () => "libxflash.dylib",
        }
    }
}
