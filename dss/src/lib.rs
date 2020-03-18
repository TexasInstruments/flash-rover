// Copyright (c) 2020 , Texas Instruments.
// Licensed under the BSD-3-Clause license
// (see LICENSE or <https://opensource.org/licenses/BSD-3-Clause>) All files in the project
// notice may not be copied, modified, or distributed except according to those terms.

extern crate jni;
extern crate path_clean;
extern crate path_slash;

pub mod com;

use std::path::Path;

use jni::JNIVersion;
use path_clean::PathClean;

use com::ti::ccstudio::scripting::environment::ScriptingEnvironment;

pub type Error = jni::errors::Error;

pub type Result<T, E = Error> = std::result::Result<T, E>;

pub struct Dss {
    jvm: jni::JavaVM,
}

impl Dss {
    pub fn new(ccs_path: &Path) -> Result<Self> {
        let dss_classpath = ccs_path
            .join("ccs_base/DebugServer/packages/ti/dss/java/dss.jar")
            .clean();

        let jvm_args = jni::InitArgsBuilder::new()
            .version(JNIVersion::V8)
            .option(&format!("-Djava.class.path={}", dss_classpath.display()))
            .option("-Dfile.encoding=UTF8")
            .option("-Xms40m")
            .option("-Xmx384m")
            .build()
            .unwrap();

        let jvm = jni::JavaVM::new(jvm_args)?;
        jvm.attach_current_thread_permanently()?;

        Ok(Self { jvm })
    }

    pub fn scripting_environment(&self) -> Result<ScriptingEnvironment> {
        let env = self.jvm.get_env()?;
        Ok(ScriptingEnvironment::new(env)?)
    }
}
