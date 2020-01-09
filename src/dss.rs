// Copyright (c) 2019 , Texas Instruments.
// Licensed under the BSD-3-Clause license
// (see LICENSE or <https://opensource.org/licenses/BSD-3-Clause>) All files in the project
// notice may not be copied, modified, or distributed except according to those terms.

use std::convert::TryFrom;
use std::env;
use std::iter;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::time::Duration;

use j4rs::{ClasspathEntry, Instance, InvocationArg, Jvm, JvmBuilder};
use path_clean::PathClean;
use path_slash::PathBufExt;
use snafu::{Backtrace, ResultExt, Snafu};

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("A J4Rs error has occured: {}", source))]
    J4RsError {
        backtrace: Backtrace,
        source: j4rs::errors::J4RsError,
    },
    #[snafu(display("An error occured modifying PATH env: {}", source))]
    PathEnvError {
        backtrace: Backtrace,
        source: std::env::JoinPathsError,
    },
}

pub type Result<T, E = Error> = std::result::Result<T, E>;

pub struct Target {
    jvm: Rc<Jvm>,
    instance: Instance,
}

impl Target {
    fn new(jvm: Rc<Jvm>, instance: Instance) -> Result<Self> {
        Ok(Self { jvm, instance })
    }

    pub fn connect(&self) -> Result<()> {
        self.jvm
            .invoke(&self.instance, "connect", &[])
            .context(J4RsError {})?;

        Ok(())
    }

    pub fn disconnect(&self) -> Result<()> {
        self.jvm
            .invoke(&self.instance, "disconnect", &[])
            .context(J4RsError {})?;

        Ok(())
    }

    pub fn reset(&self) -> Result<()> {
        self.jvm
            .invoke(&self.instance, "reset", &[])
            .context(J4RsError {})?;

        Ok(())
    }

    pub fn halt(&self) -> Result<()> {
        self.jvm
            .invoke(&self.instance, "halt", &[])
            .context(J4RsError {})?;

        Ok(())
    }

    pub fn run_asynch(&self) -> Result<()> {
        self.jvm
            .invoke(&self.instance, "runAsynch", &[])
            .context(J4RsError {})?;

        Ok(())
    }
}

#[derive(Copy, Clone)]
pub enum Register {
    R0,
    R1,
    R2,
    R3,
    R4,
    R5,
    R6,
    R7,
    R8,
    R9,
    R10,
    R11,
    R12,
    MSP,
    PSP,
    LR,
    PC,
    XPSR,
}

impl TryFrom<Register> for InvocationArg {
    type Error = Error;

    fn try_from(value: Register) -> Result<Self> {
        let res = match value {
            Register::R0 => "R0",
            Register::R1 => "R1",
            Register::R2 => "R2",
            Register::R3 => "R3",
            Register::R4 => "R4",
            Register::R5 => "R5",
            Register::R6 => "R6",
            Register::R7 => "R7",
            Register::R8 => "R8",
            Register::R9 => "R9",
            Register::R10 => "R10",
            Register::R11 => "R11",
            Register::R12 => "R12",
            Register::MSP => "MSP",
            Register::PSP => "PSP",
            Register::LR => "LR",
            Register::PC => "PC",
            Register::XPSR => "XPSR",
        };
        InvocationArg::try_from(res).context(J4RsError {})
    }
}

pub struct Memory {
    jvm: Rc<Jvm>,
    instance: Instance,
}

impl Memory {
    fn new(jvm: Rc<Jvm>, instance: Instance) -> Result<Self> {
        Ok(Self { jvm, instance })
    }

    pub fn load_raw(
        &self,
        page: usize,
        address: u32,
        filename: &str,
        type_size: usize,
        byte_swap: bool,
    ) -> Result<()> {
        self.jvm
            .invoke(
                &self.instance,
                "loadRaw",
                &[
                    InvocationArg::try_from(page as i32)
                        .context(J4RsError {})?
                        .into_primitive()
                        .context(J4RsError {})?,
                    InvocationArg::try_from(address as i64)
                        .context(J4RsError {})?
                        .into_primitive()
                        .context(J4RsError {})?,
                    InvocationArg::try_from(filename).context(J4RsError {})?,
                    InvocationArg::try_from(type_size as i32)
                        .context(J4RsError {})?
                        .into_primitive()
                        .context(J4RsError {})?,
                    InvocationArg::try_from(byte_swap)
                        .context(J4RsError {})?
                        .into_primitive()
                        .context(J4RsError {})?,
                ],
            )
            .context(J4RsError {})?;

        Ok(())
    }

    /// void writeData(int nPage, long nAddress, long nValue, int nTypeSize);
    pub fn write_data(
        &self,
        page: usize,
        address: u32,
        value: u32,
        type_size: usize,
    ) -> Result<()> {
        self.jvm
            .invoke(
                &self.instance,
                "writeData",
                &[
                    InvocationArg::try_from(page as i32)
                        .context(J4RsError {})?
                        .into_primitive()
                        .context(J4RsError {})?,
                    InvocationArg::try_from(address as i64)
                        .context(J4RsError {})?
                        .into_primitive()
                        .context(J4RsError {})?,
                    InvocationArg::try_from(value as i64)
                        .context(J4RsError {})?
                        .into_primitive()
                        .context(J4RsError {})?,
                    InvocationArg::try_from(type_size as i32)
                        .context(J4RsError {})?
                        .into_primitive()
                        .context(J4RsError {})?,
                ],
            )
            .context(J4RsError {})?;

        Ok(())
    }

    /// void writeData(int nPage, long nAddress, long[] nValues, int nTypeSize);
    pub fn write_datas(
        &self,
        page: usize,
        address: u32,
        values: &[u8],
        type_size: usize,
    ) -> Result<()> {
        let values: Result<Vec<_>, _> = values
            .iter()
            .map(|v| InvocationArg::try_from(*v as i64).and_then(|v| v.into_primitive()))
            .collect();
        let arr_values = self
            .jvm
            .create_java_array("long", &values.context(J4RsError {})?)
            .context(J4RsError {})?;
        self.jvm
            .invoke(
                &self.instance,
                "writeData",
                &[
                    InvocationArg::try_from(page as i32)
                        .context(J4RsError {})?
                        .into_primitive()
                        .context(J4RsError {})?,
                    InvocationArg::try_from(address as i64)
                        .context(J4RsError {})?
                        .into_primitive()
                        .context(J4RsError {})?,
                    InvocationArg::from(arr_values),
                    InvocationArg::try_from(type_size as i32)
                        .context(J4RsError {})?
                        .into_primitive()
                        .context(J4RsError {})?,
                ],
            )
            .context(J4RsError {})?;

        Ok(())
    }

    /// long readData(int nPage, long nAddress, int nTypeSize, boolean bSigned);
    pub fn read_data(
        &self,
        page: usize,
        address: u32,
        type_size: usize,
        signed: bool,
    ) -> Result<u32> {
        let data = self
            .jvm
            .invoke(
                &self.instance,
                "readData",
                &[
                    InvocationArg::try_from(page as i32)
                        .context(J4RsError {})?
                        .into_primitive()
                        .context(J4RsError {})?,
                    InvocationArg::try_from(address as i64)
                        .context(J4RsError {})?
                        .into_primitive()
                        .context(J4RsError {})?,
                    InvocationArg::try_from(type_size as i32)
                        .context(J4RsError {})?
                        .into_primitive()
                        .context(J4RsError {})?,
                    InvocationArg::try_from(signed)
                        .context(J4RsError {})?
                        .into_primitive()
                        .context(J4RsError {})?,
                ],
            )
            .context(J4RsError {})?;
        // We make sure to extract the value as i64, which is the equivalent type of primitive long
        let data: i64 = self.jvm.to_rust(data).context(J4RsError {})?;

        Ok(data as u32)
    }

    /// long[] readData(int nPage, long nAddress, int nTypeSize, int nNumValues, boolean bSigned);
    pub fn read_datas(
        &self,
        page: usize,
        address: u32,
        type_size: usize,
        num_values: usize,
        signed: bool,
    ) -> Result<Vec<u8>> {
        let data = self
            .jvm
            .invoke(
                &self.instance,
                "readData",
                &[
                    InvocationArg::try_from(page as i32)
                        .context(J4RsError {})?
                        .into_primitive()
                        .context(J4RsError {})?,
                    InvocationArg::try_from(address as i64)
                        .context(J4RsError {})?
                        .into_primitive()
                        .context(J4RsError {})?,
                    InvocationArg::try_from(type_size as i32)
                        .context(J4RsError {})?
                        .into_primitive()
                        .context(J4RsError {})?,
                    InvocationArg::try_from(num_values as i32)
                        .context(J4RsError {})?
                        .into_primitive()
                        .context(J4RsError {})?,
                    InvocationArg::try_from(signed)
                        .context(J4RsError {})?
                        .into_primitive()
                        .context(J4RsError {})?,
                ],
            )
            .context(J4RsError {})?;
        // We make sure to extract the value as i64, which is the equivalent type of primitive long
        let data: Vec<u8> = self.jvm.to_rust(data).context(J4RsError {})?;

        Ok(data)
    }

    /// void writeRegister(java.lang.String sRegister, long nValue);
    pub fn write_register(&self, register: Register, value: u32) -> Result<()> {
        self.jvm
            .invoke(
                &self.instance,
                "writeRegister",
                &[
                    InvocationArg::try_from(register)?,
                    InvocationArg::try_from(value as i64)
                        .context(J4RsError {})?
                        .into_primitive()
                        .context(J4RsError {})?,
                ],
            )
            .context(J4RsError {})?;

        Ok(())
    }
}

pub struct Expression {
    jvm: Rc<Jvm>,
    instance: Instance,
}

impl Expression {
    fn new(jvm: Rc<Jvm>, instance: Instance) -> Result<Self> {
        Ok(Self { jvm, instance })
    }

    pub fn evaluate(&self, expression: &str) -> Result<()> {
        self.jvm
            .invoke(
                &self.instance,
                "evaluate",
                &[InvocationArg::try_from(expression).context(J4RsError {})?],
            )
            .context(J4RsError {})?;

        Ok(())
    }
}

pub struct DebugSession {
    _jvm: Rc<Jvm>,
    _instance: Instance,
    pub target: Target,
    pub memory: Memory,
    pub expression: Expression,
}

impl DebugSession {
    fn new(jvm: Rc<Jvm>, instance: Instance) -> Result<Self> {
        let target = Target::new(
            jvm.clone(),
            jvm.field(&instance, "target").context(J4RsError {})?,
        )?;
        let memory = Memory::new(
            jvm.clone(),
            jvm.field(&instance, "memory").context(J4RsError {})?,
        )?;
        let expression = Expression::new(
            jvm.clone(),
            jvm.field(&instance, "expression").context(J4RsError {})?,
        )?;

        Ok(Self {
            _jvm: jvm,
            _instance: instance,
            target,
            memory,
            expression,
        })
    }
}

pub struct DebugServer {
    jvm: Rc<Jvm>,
    instance: Instance,
}

impl DebugServer {
    fn new(jvm: Rc<Jvm>, instance: Instance) -> Result<Self> {
        Ok(Self { jvm, instance })
    }

    /// void setConfig(java.lang.String sConfigurationFile);
    pub fn set_config(&self, config_file: &str) -> Result<()> {
        self.jvm
            .invoke(
                &self.instance,
                "setConfig",
                &[InvocationArg::try_from(config_file).context(J4RsError {})?],
            )
            .context(J4RsError {})?;

        Ok(())
    }

    pub fn open_session(&self, pattern: &str) -> Result<DebugSession> {
        let debug_session = self
            .jvm
            .invoke(
                &self.instance,
                "openSession",
                &[InvocationArg::try_from(pattern).context(J4RsError {})?],
            )
            .context(J4RsError {})?;
        let debug_session = self
            .jvm
            .cast(&debug_session, "com.ti.debug.engine.scripting.DebugSession")
            .context(J4RsError {})?;

        DebugSession::new(self.jvm.clone(), debug_session)
    }

    pub fn stop(&self) -> Result<()> {
        self.jvm
            .invoke(&self.instance, "stop", &[])
            .context(J4RsError {})?;

        Ok(())
    }
}

#[derive(Copy, Clone)]
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

impl TryFrom<TraceLevel> for InvocationArg {
    type Error = Error;

    fn try_from(value: TraceLevel) -> Result<Self, Self::Error> {
        let res = match value {
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
        InvocationArg::try_from(res).context(J4RsError {})
    }
}

pub struct ScriptingEnvironment {
    jvm: Rc<Jvm>,
    instance: Instance,
}

impl ScriptingEnvironment {
    pub fn new(jvm: Rc<Jvm>) -> Result<Self> {
        let instance = jvm
            .invoke_static(
                "com.ti.ccstudio.scripting.environment.ScriptingEnvironment",
                "instance",
                &[],
            )
            .context(J4RsError {})?;

        Ok(Self { jvm, instance })
    }

    pub fn get_server(&self) -> Result<DebugServer> {
        let debug_server = self
            .jvm
            .invoke(
                &self.instance,
                "getServer",
                &[InvocationArg::try_from("DebugServer.1").context(J4RsError {})?],
            )
            .context(J4RsError {})?;
        let debug_server = self
            .jvm
            .cast(&debug_server, "com.ti.debug.engine.scripting.DebugServer")
            .context(J4RsError {})?;

        DebugServer::new(self.jvm.clone(), debug_server)
    }

    pub fn trace_begin(&self, filename: &str, stylesheet: &str) -> Result<()> {
        self.jvm
            .invoke(
                &self.instance,
                "traceBegin",
                &[
                    InvocationArg::try_from(filename).context(J4RsError {})?,
                    InvocationArg::try_from(stylesheet).context(J4RsError {})?,
                ],
            )
            .context(J4RsError {})?;

        Ok(())
    }

    pub fn trace_end(&self) -> Result<()> {
        self.jvm
            .invoke(&self.instance, "traceEnd", &[])
            .context(J4RsError {})?;

        Ok(())
    }

    pub fn trace_set_console_level(&self, trace_level: TraceLevel) -> Result<()> {
        self.jvm
            .invoke(
                &self.instance,
                "traceSetConsoleLevel",
                &[InvocationArg::try_from(trace_level)?],
            )
            .context(J4RsError {})?;

        Ok(())
    }

    pub fn trace_set_file_level(&self, trace_level: TraceLevel) -> Result<()> {
        self.jvm
            .invoke(
                &self.instance,
                "traceSetFileLevel",
                &[InvocationArg::try_from(trace_level)?],
            )
            .context(J4RsError {})?;

        Ok(())
    }

    pub fn set_script_timeout(&self, timeout: Duration) -> Result<()> {
        self.jvm
            .invoke(
                &self.instance,
                "setScriptTimeout",
                &[InvocationArg::try_from(timeout.as_millis() as i32)
                    .context(J4RsError {})?
                    .into_primitive()
                    .context(J4RsError {})?],
            )
            .context(J4RsError {})?;

        Ok(())
    }
}

pub fn build_jvm(ccs_path: &Path) -> Result<Rc<Jvm>> {
    let ccs_java_path = ccs_path
        .join(PathBuf::from_slash("eclipse/jre/bin"))
        .clean();

    let dss_path = ccs_path.join("ccs_base/DebugServer/packages/ti/dss/java/dss.jar");

    let path = env::var_os("PATH").unwrap_or_default();
    let path_iter = env::split_paths(&path);
    let paths = iter::once(ccs_java_path).chain(path_iter);
    env::set_var("PATH", &env::join_paths(paths).context(PathEnvError {})?);

    Ok(Rc::new(
        JvmBuilder::new()
            .classpath_entry(ClasspathEntry::new(dss_path.to_str().unwrap()))
            .java_opts(vec![
                j4rs::JavaOpt::new("-Dfile.encoding=UTF8"),
                j4rs::JavaOpt::new("-Xms40m"),
                j4rs::JavaOpt::new("-Xmx384m"),
            ])
            .build()
            .context(J4RsError {})?,
    ))
}
