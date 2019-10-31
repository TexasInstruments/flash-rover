// Copyright (c) 2019 , Texas Instruments.
// Licensed under the BSD-3-Clause license
// (see LICENSE or <https://opensource.org/licenses/BSD-3-Clause>) All files in the project
// notice may not be copied, modified, or distributed except according to those terms.

use std::convert::TryFrom;
use std::env;
use std::fs::File;
use std::io;
use std::io::{Read, Write};
use std::iter;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::thread;
use std::time::Duration;

use j4rs::errors::J4RsError;
use j4rs::ClasspathEntry;
use j4rs::Instance;
use j4rs::InvocationArg;
use j4rs::Jvm;
use j4rs::JvmBuilder;

use failure::{err_msg, Fail};
use path_clean::PathClean;
use path_slash::PathBufExt;
use tempfile::TempPath;

use crate::args;
use crate::flash_rover::XflashInfo;

#[allow(dead_code)]
#[derive(Fail, Debug)]
pub enum Error {
    #[fail(display = "A FW error has occured: {}", inner)]
    Fw {
        #[fail(cause)]
        inner: failure::Error,
    },
    #[fail(display = "A DSS error has occured: {}", inner)]
    J4Rs {
        #[fail(cause)]
        inner: J4RsError,
    },
    #[fail(display = "An IO error has occured: {}", inner)]
    Io {
        #[fail(cause)]
        inner: io::Error,
    },
    #[fail(display = "An error occured modifying PATH env: {}", inner)]
    PathEnv {
        #[fail(cause)]
        inner: env::JoinPathsError,
    },
}

impl From<J4RsError> for Error {
    fn from(err: J4RsError) -> Self {
        Self::J4Rs { inner: err }
    }
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Self {
        Self::Io { inner: err }
    }
}

impl From<env::JoinPathsError> for Error {
    fn from(err: env::JoinPathsError) -> Self {
        Self::PathEnv { inner: err }
    }
}

type Result<T, E = Error> = std::result::Result<T, E>;

pub struct Target {
    jvm: Rc<Jvm>,
    instance: Instance,
}

impl Target {
    fn new(jvm: Rc<Jvm>, instance: Instance) -> Result<Self> {
        Ok(Self { jvm, instance })
    }

    pub fn connect(&self) -> Result<()> {
        self.jvm.invoke(&self.instance, "connect", &[])?;

        Ok(())
    }

    pub fn disconnect(&self) -> Result<()> {
        self.jvm.invoke(&self.instance, "disconnect", &[])?;

        Ok(())
    }

    pub fn reset(&self) -> Result<()> {
        self.jvm.invoke(&self.instance, "reset", &[])?;

        Ok(())
    }

    pub fn halt(&self) -> Result<()> {
        self.jvm.invoke(&self.instance, "halt", &[])?;

        Ok(())
    }

    pub fn run_asynch(&self) -> Result<()> {
        self.jvm.invoke(&self.instance, "runAsynch", &[])?;

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
    type Error = J4RsError;

    fn try_from(value: Register) -> std::result::Result<Self, Self::Error> {
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
        InvocationArg::try_from(res)
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
        self.jvm.invoke(
            &self.instance,
            "loadRaw",
            &[
                InvocationArg::try_from(page as i32)?.into_primitive()?,
                InvocationArg::try_from(address as i64)?.into_primitive()?,
                InvocationArg::try_from(filename)?,
                InvocationArg::try_from(type_size as i32)?.into_primitive()?,
                InvocationArg::try_from(byte_swap)?.into_primitive()?,
            ],
        )?;

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
        self.jvm.invoke(
            &self.instance,
            "writeData",
            &[
                InvocationArg::try_from(page as i32)?.into_primitive()?,
                InvocationArg::try_from(address as i64)?.into_primitive()?,
                InvocationArg::try_from(value as i64)?.into_primitive()?,
                InvocationArg::try_from(type_size as i32)?.into_primitive()?,
            ],
        )?;

        Ok(())
    }

    /// void writeData(int nPage, long nAddress, long[] nValues, int nTypeSize);
    pub fn write_datas(
        &self,
        page: usize,
        address: u32,
        values: Vec<i64>,
        type_size: usize,
    ) -> Result<()> {
        let values: Result<Vec<_>, _> = values
            .into_iter()
            .map(|v| InvocationArg::try_from(v).and_then(|v| v.into_primitive()))
            .collect();
        let arr_values = self.jvm.create_java_array("long", &values?)?;
        self.jvm.invoke(
            &self.instance,
            "writeData",
            &[
                InvocationArg::try_from(page as i32)?.into_primitive()?,
                InvocationArg::try_from(address as i64)?.into_primitive()?,
                InvocationArg::from(arr_values),
                InvocationArg::try_from(type_size as i32)?.into_primitive()?,
            ],
        )?;

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
        let data = self.jvm.invoke(
            &self.instance,
            "readData",
            &[
                InvocationArg::try_from(page as i32)?.into_primitive()?,
                InvocationArg::try_from(address as i64)?.into_primitive()?,
                InvocationArg::try_from(type_size as i32)?.into_primitive()?,
                InvocationArg::try_from(signed)?.into_primitive()?,
            ],
        )?;
        // We make sure to extract the value as i64, which is the equivalent type of primitive long
        let data: i64 = self.jvm.to_rust(data)?;

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
        let data = self.jvm.invoke(
            &self.instance,
            "readData",
            &[
                InvocationArg::try_from(page as i32)?.into_primitive()?,
                InvocationArg::try_from(address as i64)?.into_primitive()?,
                InvocationArg::try_from(type_size as i32)?.into_primitive()?,
                InvocationArg::try_from(num_values as i32)?.into_primitive()?,
                InvocationArg::try_from(signed)?.into_primitive()?,
            ],
        )?;
        // We make sure to extract the value as i64, which is the equivalent type of primitive long
        let data: Vec<u8> = self.jvm.to_rust(data)?;

        Ok(data)
    }

    /// void writeRegister(java.lang.String sRegister, long nValue);
    pub fn write_register(&self, register: Register, value: u32) -> Result<()> {
        self.jvm.invoke(
            &self.instance,
            "writeRegister",
            &[
                InvocationArg::try_from(register)?,
                InvocationArg::try_from(value as i64)?.into_primitive()?,
            ],
        )?;

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
        self.jvm.invoke(
            &self.instance,
            "evaluate",
            &[InvocationArg::try_from(expression)?],
        )?;

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
        let target = Target::new(jvm.clone(), jvm.field(&instance, "target")?)?;
        let memory = Memory::new(jvm.clone(), jvm.field(&instance, "memory")?)?;
        let expression = Expression::new(jvm.clone(), jvm.field(&instance, "expression")?)?;

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
        self.jvm.invoke(
            &self.instance,
            "setConfig",
            &[InvocationArg::try_from(config_file)?],
        )?;

        Ok(())
    }

    pub fn open_session(&self, pattern: &str) -> Result<DebugSession> {
        let debug_session = self.jvm.invoke(
            &self.instance,
            "openSession",
            &[InvocationArg::try_from(pattern)?],
        )?;
        let debug_session = self
            .jvm
            .cast(&debug_session, "com.ti.debug.engine.scripting.DebugSession")?;

        DebugSession::new(self.jvm.clone(), debug_session)
    }

    pub fn stop(&self) -> Result<()> {
        self.jvm.invoke(&self.instance, "stop", &[])?;

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
    type Error = J4RsError;

    fn try_from(value: TraceLevel) -> std::result::Result<Self, Self::Error> {
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
        InvocationArg::try_from(res)
    }
}

pub struct ScriptingEnvironment {
    jvm: Rc<Jvm>,
    instance: Instance,
}

impl ScriptingEnvironment {
    pub fn new(jvm: Rc<Jvm>) -> Result<Self> {
        let instance = jvm.invoke_static(
            "com.ti.ccstudio.scripting.environment.ScriptingEnvironment",
            "instance",
            &[],
        )?;

        Ok(Self { jvm, instance })
    }

    pub fn get_server(&self) -> Result<DebugServer> {
        let debug_server = self.jvm.invoke(
            &self.instance,
            "getServer",
            &[InvocationArg::try_from("DebugServer.1")?],
        )?;
        let debug_server = self
            .jvm
            .cast(&debug_server, "com.ti.debug.engine.scripting.DebugServer")?;

        DebugServer::new(self.jvm.clone(), debug_server)
    }

    pub fn trace_begin(&self, filename: &str, stylesheet: &str) -> Result<()> {
        self.jvm.invoke(
            &self.instance,
            "traceBegin",
            &[
                InvocationArg::try_from(filename)?,
                InvocationArg::try_from(stylesheet)?,
            ],
        )?;

        Ok(())
    }

    pub fn trace_end(&self) -> Result<()> {
        self.jvm.invoke(&self.instance, "traceEnd", &[])?;

        Ok(())
    }

    pub fn trace_set_console_level(&self, trace_level: TraceLevel) -> Result<()> {
        self.jvm.invoke(
            &self.instance,
            "traceSetConsoleLevel",
            &[InvocationArg::try_from(trace_level)?],
        )?;

        Ok(())
    }

    pub fn trace_set_file_level(&self, trace_level: TraceLevel) -> Result<()> {
        self.jvm.invoke(
            &self.instance,
            "traceSetFileLevel",
            &[InvocationArg::try_from(trace_level)?],
        )?;

        Ok(())
    }

    pub fn set_script_timeout(&self, timeout: Duration) -> Result<()> {
        self.jvm.invoke(
            &self.instance,
            "setScriptTimeout",
            &[InvocationArg::try_from(timeout.as_millis() as i32)?.into_primitive()?],
        )?;

        Ok(())
    }
}

fn build_jvm(ccs_path: &Path) -> Result<Rc<Jvm>> {
    let ccs_java_path = ccs_path
        .join(PathBuf::from_slash("eclipse/jre/bin"))
        .clean();

    let dss_path = ccs_path.join("ccs_base/DebugServer/packages/ti/dss/java/dss.jar");

    let path = env::var_os("PATH").unwrap_or_default();
    let path_iter = env::split_paths(&path);
    let paths = iter::once(ccs_java_path).chain(path_iter);
    env::set_var("PATH", &env::join_paths(paths)?);

    Ok(Rc::new(
        JvmBuilder::new()
            .classpath_entry(ClasspathEntry::new(dss_path.to_str().unwrap()))
            .java_opts(vec![
                j4rs::JavaOpt::new("-Dfile.encoding=UTF8"),
                j4rs::JavaOpt::new("-Xms40m"),
                j4rs::JavaOpt::new("-Xmx384m"),
            ])
            .build()?,
    ))
}

fn exe_dir() -> io::Result<PathBuf> {
    Ok(env::current_exe()?
        .parent()
        .ok_or_else(|| io::Error::from(io::ErrorKind::NotFound))?
        .to_owned())
}

fn create_ccxml(xds: &str, device: &str) -> io::Result<TempPath> {
    let cwd = exe_dir()?;
    let template = cwd
        .join("dss/ccxml")
        .join(format!("template_{}.ccxml", device))
        .clean();

    let mut content = Vec::new();
    File::open(template)?.read_to_end(&mut content)?;

    let content = String::from_utf8_lossy(&content[..]);
    const PATTERN: &str = "<<<SERIAL NUMBER>>>";
    let content = content.replace(PATTERN, &xds);

    let mut ccxml = tempfile::Builder::new()
        .prefix("flash-rover.")
        .suffix(".ccxml")
        .tempfile()?;
    ccxml.write_all(content.as_bytes())?;
    let (file, path) = ccxml.into_parts();
    drop(file);

    Ok(path)
}

const LOG_FILENAME: &str = "dss_log.xml";
const LOG_STYLESHEET: &str = "DefaultStylesheet.xsl";
const SCRIPT_TIMEOUT: Duration = Duration::from_secs(15);
const SESSION_PATTERN: &str = "Texas Instruments XDS110 USB Debug Probe/Cortex_M(3|4)_0";

const SRAM_START: u32 = 0x2000_0000;
const STACK_ADDR: u32 = SRAM_START + 0x00;
const RESET_ISR: u32 = SRAM_START + 0x04;

const CONF_START: u32 = 0x2000_3000;
const CONF_VALID: u32 = CONF_START + 0x00;
const CONF_SPI_MISO: u32 = CONF_START + 0x04;
const CONF_SPI_MOSI: u32 = CONF_START + 0x08;
const CONF_SPI_CLK: u32 = CONF_START + 0x0C;
const CONF_SPI_CSN: u32 = CONF_START + 0x10;

const DOORBELL_START: u32 = 0x2000_3100;
const DOORBELL_CMD: u32 = DOORBELL_START + 0x00;
const DOORBELL_RSP: u32 = DOORBELL_START + 0x10;

const XFLASH_BUF_START: u32 = 0x2000_4000;
const XFLASH_BUF_SIZE: u32 = 0x1000;

#[derive(Copy, Clone, Debug)]
enum FwCmd {
    GetXflashInfo,
    SectorErase { offset: u32, length: u32 },
    MassErase,
    ReadBlock { offset: u32, length: u32 },
    WriteBlock { offset: u32, length: u32 },
}

impl FwCmd {
    fn to_bytes(&self) -> [u32; 4] {
        use FwCmd::*;

        match self {
            GetXflashInfo => [0xC0_u32.to_le(), 0, 0, 0],
            SectorErase { offset, length } => [0xC1_u32.to_le(), offset.to_le(), length.to_le(), 0],
            MassErase => [0xC2_u32.to_le(), 0, 0, 0],
            ReadBlock { offset, length } => [0xC3_u32.to_le(), offset.to_le(), length.to_le(), 0],
            WriteBlock { offset, length } => [0xC4_u32.to_le(), offset.to_le(), length.to_le(), 0],
        }
    }
}

#[derive(Copy, Clone, Debug)]
enum FwRsp {
    Ok,
    XflashInfo { mid: u32, did: u32 },
}

impl FwRsp {
    fn from_bytes(bytes: &[u32; 4]) -> Result<FwRsp> {
        const OK_VAL: u32 = 0xD0_u32.to_le();
        const XFLASHINFO_VAL: u32 = 0xD1_u32.to_le();

        Ok(match bytes {
            [OK_VAL, 0, 0, 0] => FwRsp::Ok,
            [XFLASHINFO_VAL, mid, did, 0] => FwRsp::XflashInfo {
                mid: *mid,
                did: *did,
            },
            _ => {
                return Err(Error::Fw {
                    inner: err_msg("Received invalid FW response"),
                })
            }
        })
    }
}

pub struct FlashRover {
    command: args::Command,
    _ccxml: TempPath,
    script: ScriptingEnvironment,
    debug_server: DebugServer,
    debug_session: DebugSession,
}

impl FlashRover {
    pub fn new(command: args::Command) -> Result<Self> {
        let ccxml = create_ccxml(&command.xds_id, &command.device_kind)?;
        let jvm = build_jvm(command.ccs_path.as_path())?;

        let script = ScriptingEnvironment::new(jvm.clone())?;
        script.trace_begin(LOG_FILENAME, LOG_STYLESHEET)?;
        script.trace_set_console_level(TraceLevel::Off)?;
        script.trace_set_file_level(TraceLevel::All)?;
        script.set_script_timeout(SCRIPT_TIMEOUT)?;

        let debug_server = script.get_server()?;
        debug_server.set_config(&ccxml.to_string_lossy().to_owned())?;

        let debug_session = debug_server.open_session(SESSION_PATTERN)?;
        debug_session.target.connect()?;
        debug_session.target.reset()?;
        debug_session
            .expression
            .evaluate("GEL_AdvancedReset(\"Board Reset (automatic connect/disconnect)\")")?;

        Ok(Self {
            command,
            _ccxml: ccxml,
            script,
            debug_server,
            debug_session,
        })
    }

    pub fn run(self) -> Result<()> {
        use args::Subcommand::*;

        self.inject()?;

        Ok(match &self.command.subcommand {
            Info => self.info()?,
            SectorErase { offset, length } => self.sector_erase(*offset, *length)?,
            MassErase => self.mass_erase()?,
            Read {
                offset,
                length,
                output,
            } => self.read(*offset, *length, output.borrow_mut().as_mut())?,
            Write {
                erase,
                offset,
                length,
                input,
            } => self.write(*erase, *offset, *length, input.borrow_mut().as_mut())?,
        })
    }

    fn inject(&self) -> Result<()> {
        let memory = &self.debug_session.memory;

        let fw = exe_dir()?.join(PathBuf::from_slash(format!(
            "dss/fw/{}.bin",
            &self.command.device_kind
        )));
        memory.load_raw(0, SRAM_START, fw.to_str().unwrap(), 32, false)?;

        if let Some(spi_pins) = self.command.spi_pins.as_ref() {
            memory.write_data(0, CONF_VALID, 1, 32)?;
            memory.write_data(0, CONF_SPI_MISO, spi_pins[0] as u32, 32)?;
            memory.write_data(0, CONF_SPI_MOSI, spi_pins[1] as u32, 32)?;
            memory.write_data(0, CONF_SPI_CLK, spi_pins[2] as u32, 32)?;
            memory.write_data(0, CONF_SPI_CSN, spi_pins[3] as u32, 32)?;
        }

        let stack_addr = memory.read_data(0, STACK_ADDR, 32, false)?;
        let reset_isr = memory.read_data(0, RESET_ISR, 32, false)?;

        memory.write_register(Register::MSP, stack_addr)?;
        memory.write_register(Register::PC, reset_isr)?;
        memory.write_register(Register::LR, 0xFFFF_FFFF)?;

        self.debug_session.target.run_asynch()?;

        Ok(())
    }

    fn send_fw_cmd(&self, fw_cmd: FwCmd) -> Result<FwRsp> {
        let memory = &self.debug_session.memory;

        let fw_cmd_bytes = fw_cmd.to_bytes();

        memory.write_data(0, DOORBELL_CMD + 0x0C, fw_cmd_bytes[3], 32)?;
        memory.write_data(0, DOORBELL_CMD + 0x08, fw_cmd_bytes[2], 32)?;
        memory.write_data(0, DOORBELL_CMD + 0x04, fw_cmd_bytes[1], 32)?;
        // Kind must be written last to trigger the command
        memory.write_data(0, DOORBELL_CMD + 0x00, fw_cmd_bytes[0], 32)?;

        const SLEEP_TIME: Duration = Duration::from_millis(100);

        while memory.read_data(0, DOORBELL_CMD, 32, false)? != 0 {
            thread::sleep(SLEEP_TIME);
        }

        while memory.read_data(0, DOORBELL_RSP, 32, false)? == 0 {
            thread::sleep(SLEEP_TIME);
        }

        let fw_rsp_bytes: [u32; 4] = [
            memory.read_data(0, DOORBELL_RSP + 0x00, 32, false)?,
            memory.read_data(0, DOORBELL_RSP + 0x04, 32, false)?,
            memory.read_data(0, DOORBELL_RSP + 0x08, 32, false)?,
            memory.read_data(0, DOORBELL_RSP + 0x0C, 32, false)?,
        ];

        memory.write_data(0, DOORBELL_RSP, 0, 32)?;

        FwRsp::from_bytes(&fw_rsp_bytes)
    }

    fn info(&self) -> Result<()> {
        match self.send_fw_cmd(FwCmd::GetXflashInfo)? {
            FwRsp::XflashInfo { mid, did } => {
                if let Some(xflash_info) = XflashInfo::find(mid, did) {
                    println!("{}", xflash_info);
                } else {
                    println!(
                        "Unknown and possibly unsupported external flash (MID: {}, DID: {})",
                        mid, did
                    );
                }
            }
            other_rsp => {
                return Err(Error::Fw {
                    inner: format_err!(
                        "Received unexpected response from FW during info command: {:?}",
                        other_rsp
                    ),
                })
            }
        }

        Ok(())
    }

    fn sector_erase(&self, offset: u32, length: u32) -> Result<()> {
        match self.send_fw_cmd(FwCmd::SectorErase { offset, length })? {
            FwRsp::Ok => { /* success, do nothing */ }
            other_rsp => {
                return Err(Error::Fw {
                    inner: format_err!(
                        "Received unexpected response from FW during info command: {:?}",
                        other_rsp
                    ),
                })
            }
        }

        Ok(())
    }

    fn mass_erase(&self) -> Result<()> {
        print!("Starting mass erase, this may take some time... ");
        io::stdout().flush()?;

        match self.send_fw_cmd(FwCmd::MassErase)? {
            FwRsp::Ok => {
                println!("Done.");
            }
            other_rsp => {
                println!("Error.");
                return Err(Error::Fw {
                    inner: format_err!(
                        "Received unexpected response from FW during info command: {:?}",
                        other_rsp
                    ),
                });
            }
        }

        Ok(())
    }

    fn read(&self, offset: u32, length: u32, output: &mut dyn Write) -> Result<()> {
        let memory = &self.debug_session.memory;
        let mut length_rest = length;
        let mut offset_rest = offset;

        while length_rest > 0 {
            let ilength = std::cmp::min(length_rest, XFLASH_BUF_SIZE);

            let fw_cmd = FwCmd::ReadBlock {
                offset: offset_rest,
                length: length_rest,
            };
            match self.send_fw_cmd(fw_cmd)? {
                FwRsp::Ok => { /* successful, do nothing */ }
                other_rsp => {
                    return Err(Error::Fw {
                        inner: format_err!(
                            "Received unexpected response from FW during info command: {:?}",
                            other_rsp
                        ),
                    })
                }
            }

            let data = memory.read_datas(0, XFLASH_BUF_START, 8, ilength as usize, false)?;
            io::copy(&mut data.as_slice(), output)?;

            length_rest -= ilength;
            offset_rest += ilength;
        }

        Ok(())
    }

    fn write(
        &self,
        erase: bool,
        offset: u32,
        length: Option<u32>,
        input: &mut dyn Read,
    ) -> Result<()> {
        let memory = &self.debug_session.memory;

        let vec = if let Some(length) = length {
            let mut vec = Vec::with_capacity(length as usize);
            input.take(length as u64).read(&mut vec)?;
            vec
        } else {
            let mut vec = Vec::new();
            input.read_to_end(&mut vec)?;
            vec
        };

        let length = vec.len() as u32;

        let mut offset_rest = offset;

        if erase {
            self.sector_erase(offset, length)?;
        }

        for chunk in vec.chunks(XFLASH_BUF_SIZE as usize) {
            let buf: Vec<i64> = chunk.iter().map(|i| *i as _).collect();
            let ilength = buf.len() as u32;

            memory.write_datas(0, XFLASH_BUF_START, buf, 8)?;

            let fw_cmd = FwCmd::WriteBlock {
                offset: offset_rest,
                length: ilength,
            };
            match self.send_fw_cmd(fw_cmd)? {
                FwRsp::Ok => { /* successful, do nothing */ }
                other_rsp => {
                    return Err(Error::Fw {
                        inner: format_err!(
                            "Received unexpected response from FW during info command: {:?}",
                            other_rsp
                        ),
                    })
                }
            }

            offset_rest += ilength;
        }

        Ok(())
    }
}

impl Drop for FlashRover {
    fn drop(&mut self) {
        let f = || -> Result<()> {
            self.debug_session.target.halt()?;
            self.debug_session.target.reset()?;
            self.debug_session.target.disconnect()?;

            self.debug_server.stop()?;

            self.script.trace_set_console_level(TraceLevel::Info)?;
            self.script.trace_end()?;

            Ok(())
        };
        f().unwrap_or_default();
    }
}
