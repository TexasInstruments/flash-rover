// Copyright (c) 2020 , Texas Instruments.
// Licensed under the BSD-3-Clause license
// (see LICENSE or <https://opensource.org/licenses/BSD-3-Clause>) All files in the project
// notice may not be copied, modified, or distributed except according to those terms.

use std::str;
use std::string;
use std::time::Duration;

use jni::{objects::JObject, JNIEnv};

use crate::com::ti::debug::engine::scripting::DebugServer;

type Result<T, E = jni::errors::Error> = std::result::Result<T, E>;

#[derive(Copy, Clone, Debug)]
pub enum TraceLevel {
    Off,
    Severe,
    Warning,
    Info,
    Config,
    Fine,
    Finer,
    Finest,
    All,
}

impl string::ToString for TraceLevel {
    fn to_string(&self) -> String {
        let res = match self {
            TraceLevel::Off => "OFF",
            TraceLevel::Severe => "SEVERE",
            TraceLevel::Warning => "WARNING",
            TraceLevel::Info => "INFO",
            TraceLevel::Config => "CONFIG",
            TraceLevel::Fine => "FINE",
            TraceLevel::Finer => "FINER",
            TraceLevel::Finest => "FINEST",
            TraceLevel::All => "ALL",
        };
        res.to_owned()
    }
}

impl str::FromStr for TraceLevel {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "OFF" => Ok(TraceLevel::Off),
            "SEVERE" => Ok(TraceLevel::Severe),
            "WARNING" => Ok(TraceLevel::Warning),
            "INFO" => Ok(TraceLevel::Info),
            "CONFIG" => Ok(TraceLevel::Config),
            "FINE" => Ok(TraceLevel::Fine),
            "FINER" => Ok(TraceLevel::Finer),
            "FINEST" => Ok(TraceLevel::Finest),
            "ALL" => Ok(TraceLevel::All),
            _ => Err(format!("Invalid TraceLevel string {}", s).into()),
        }
    }
}

pub struct ScriptingEnvironment<'a> {
    env: JNIEnv<'a>,
    instance: JObject<'a>,
}

impl<'a> ScriptingEnvironment<'a> {
    const CLASS: &'static str = "Lcom/ti/ccstudio/scripting/environment/ScriptingEnvironment;";

    pub(crate) fn new(env: JNIEnv<'a>) -> Result<ScriptingEnvironment<'a>> {
        const METHOD: &str = "instance";
        const SIGNATURE: &str = "()Lcom/ti/ccstudio/scripting/environment/ScriptingEnvironment;";

        let class = env.find_class(ScriptingEnvironment::CLASS)?;

        let instance = env.call_static_method(class, METHOD, SIGNATURE, &[])?.l()?;

        Ok(Self { env, instance })
    }

    pub fn get_server(&self, server_name: &str) -> Result<DebugServer<'a>> {
        const METHOD: &str = "getServer";
        const SIGNATURE: &str = "(Ljava/lang/String;)Lcom/ti/ccstudio/scripting/IScriptServer;";

        let file_name = JObject::from(self.env.new_string(server_name)?);

        let debug_server = self
            .env
            .call_method(self.instance, METHOD, SIGNATURE, &[From::from(file_name)])?
            .l()?;

        assert!(self
            .env
            .is_instance_of(debug_server, DebugServer::CLASS)
            .expect("Invalid instance of DebugServer"));

        Ok(DebugServer::new(self.env.clone(), debug_server)?)
    }

    pub fn trace_begin(&self, filename: &str, stylesheet: &str) -> Result<()> {
        const METHOD: &str = "traceBegin";
        const SIGNATURE: &str = "(Ljava/lang/String;Ljava/lang/String;)V";

        let file_name = JObject::from(self.env.new_string(filename)?);
        let stylesheet_name = JObject::from(self.env.new_string(stylesheet)?);

        self.env
            .call_method(
                self.instance,
                METHOD,
                SIGNATURE,
                &[From::from(file_name), From::from(stylesheet_name)],
            )?
            .v()?;

        Ok(())
    }

    pub fn trace_end(&self) -> Result<()> {
        const METHOD: &str = "traceEnd";
        const SIGNATURE: &str = "()V";

        self.env
            .call_method(self.instance, METHOD, SIGNATURE, &[])?
            .v()?;

        Ok(())
    }

    pub fn trace_set_console_level(&self, trace_level: TraceLevel) -> Result<()> {
        const METHOD: &str = "traceSetConsoleLevel";
        const SIGNATURE: &str = "(Ljava/lang/String;)V";

        let level = JObject::from(self.env.new_string(trace_level.to_string())?);

        self.env
            .call_method(self.instance, METHOD, SIGNATURE, &[From::from(level)])?
            .v()?;

        Ok(())
    }

    pub fn trace_set_file_level(&self, trace_level: TraceLevel) -> Result<()> {
        const METHOD: &str = "traceSetFileLevel";
        const SIGNATURE: &str = "(Ljava/lang/String;)V";

        let level = JObject::from(self.env.new_string(trace_level.to_string())?);

        self.env
            .call_method(self.instance, METHOD, SIGNATURE, &[From::from(level)])?
            .v()?;

        Ok(())
    }

    pub fn set_script_timeout(&self, timeout: Duration) -> Result<()> {
        const METHOD: &str = "setScriptTimeout";
        const SIGNATURE: &str = "(I)V";

        let timeout = timeout.as_millis() as jni::sys::jint;

        self.env
            .call_method(self.instance, METHOD, SIGNATURE, &[From::from(timeout)])?
            .v()?;

        Ok(())
    }
}
