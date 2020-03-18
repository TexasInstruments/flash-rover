use std::io;
use std::os;
use std::path::{Path, PathBuf};

use path_clean::PathClean;
use path_slash::PathBufExt;
use snafu::{Backtrace, OptionExt, ResultExt, Snafu};

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("Unable to find java home in CCS root: {}", ccs_root.display()))]
    NoJavaHome {
        ccs_root: PathBuf,
        backtrace: Backtrace,
    },
    #[snafu(display("Unable to find JVM lib in Java home: {}", java_home.display()))]
    NoLibJvm {
        java_home: PathBuf,
        backtrace: Backtrace,
    },
    LibJvmCopyError {
        source: io::Error,
        backtrace: Backtrace,
    }
}

pub type Result<T, E = Error> = std::result::Result<T, E>;

#[cfg(target_arch = "x86")]
const JAVA_ARCH: &str = "i386";

#[cfg(target_arch = "x86_64")]
const JAVA_ARCH: &str = "amd64";

const JAVA_HOME_LOCATIONS: &[&str] = &[
    "eclipse/jre",
    "eclipse/Ccstudio.app/jre/Contents/Home",
    "ccs_base/jre",
    "ccs_base/eclipse/jre",
];

const JVM_LOCATIONS: &[&str] = &[
    "bin/server",
    "bin/{JAVA_ARCH}/server",
    "lib/server",
    "lib/{JAVA_ARCH}/server",
];

pub fn copy_to_workdir(workdir: &Path, ccs_root: &Path) -> Result<()> {
    if workdir.join(libjvm_filename()).exists() {
        // No need to copy, already exists in workdir
        println!("libjvm link exists");
        return Ok(());
    }
    println!("libjvm link does not exists, create one");

    let java_home = find_java_home(ccs_root).context(NoJavaHome{ ccs_root })?;
    let libjvm = find_libjvm(&java_home).context(NoLibJvm{java_home})?;
    let destination = workdir.join(libjvm_filename());

    create_symlink(libjvm, destination).context(LibJvmCopyError{})?;

    Ok(())
}

#[cfg(windows)]
fn create_symlink<P: AsRef<Path>, Q: AsRef<Path>>(src: P, dst: Q) -> io::Result<()> {
    os::windows::fs::symlink_file(src, dst)
}

#[cfg(unix)]
fn create_symlink<P: AsRef<Path>, Q: AsRef<Path>>(src: P, dst: Q) -> io::Result<()> {
    os::unix::fs::symlink(src, dst)
}

fn libjvm_filename() -> &'static str {
    match () {
        #[cfg(target_os = "windows")]
        () => "jvm.dll",
        #[cfg(target_os = "linux")]
        () => "libjvm.so",
        #[cfg(target_os = "macos")]
        () => "libjvm.dylib",
    }
}

fn find_java_home(ccs_root: &Path) -> Option<PathBuf> {
    JAVA_HOME_LOCATIONS
        .iter()
        .map(PathBuf::from_slash)
        .map(|p| ccs_root.join(p).clean())
        .find(|p| p.exists())
}

fn find_libjvm(java_home: &Path) -> Option<PathBuf> {
    JVM_LOCATIONS
        .iter()
        .map(|p| p.replace("{JAVA_ARCH}", JAVA_ARCH))
        .map(PathBuf::from_slash)
        .map(|p| java_home.join(p).join(libjvm_filename()).clean())
        .find(|p| p.exists())
}
