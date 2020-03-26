// Copyright (c) 2020 , Texas Instruments.
// Licensed under the BSD-3-Clause license
// (see LICENSE or <https://opensource.org/licenses/BSD-3-Clause>) All files in the project
// notice may not be copied, modified, or distributed except according to those terms.

use std::string;

use jni::{
    objects::JObject,
    sys::{jboolean, jint, jlong, jlongArray, jsize},
    JNIEnv,
};

pub type Result<T, E = jni::errors::Error> = std::result::Result<T, E>;

pub struct DebugServer<'a> {
    env: JNIEnv<'a>,
    instance: JObject<'a>,
}

impl<'a> DebugServer<'a> {
    pub(crate) const CLASS: &'static str = "Lcom/ti/debug/engine/scripting/DebugServer;";

    pub(crate) fn new(env: JNIEnv<'a>, instance: JObject<'a>) -> Result<Self> {
        Ok(Self { env, instance })
    }

    // void setConfig(java.lang.String sConfigurationFile);
    pub fn set_config(&self, config_file: &str) -> Result<()> {
        const METHOD: &str = "setConfig";
        const SIGNATURE: &str = "(Ljava/lang/String;)V";

        let config_file = JObject::from(self.env.new_string(config_file)?);

        self.env
            .call_method(self.instance, METHOD, SIGNATURE, &[From::from(config_file)])?
            .v()?;

        Ok(())
    }

    pub fn open_session(&self, pattern: &str) -> Result<DebugSession<'a>> {
        const METHOD: &str = "openSession";
        const SIGNATURE: &str = "(Ljava/lang/String;)Lcom/ti/debug/engine/scripting/DebugSession;";

        let pattern = JObject::from(self.env.new_string(pattern)?);

        let debug_session = self
            .env
            .call_method(self.instance, METHOD, SIGNATURE, &[From::from(pattern)])?
            .l()?;

        Ok(DebugSession::new(self.env.clone(), debug_session)?)
    }

    pub fn stop(&self) -> Result<()> {
        const METHOD: &str = "stop";
        const SIGNATURE: &str = "()V";

        self.env
            .call_method(self.instance, METHOD, SIGNATURE, &[])?
            .v()?;

        Ok(())
    }
}

pub struct DebugSession<'a> {
    _env: JNIEnv<'a>,
    _instance: JObject<'a>,
    pub target: Target<'a>,
    pub memory: Memory<'a>,
    pub expression: Expression<'a>,
}

impl<'a> DebugSession<'a> {
    pub(crate) fn new(env: JNIEnv<'a>, instance: JObject<'a>) -> Result<DebugSession<'a>> {
        let target = env.get_field(instance, "target", Target::CLASS)?.l()?;
        let memory = env.get_field(instance, "memory", Memory::CLASS)?.l()?;
        let expression = env
            .get_field(instance, "expression", Expression::CLASS)?
            .l()?;

        let target = Target::new(env.clone(), target)?;
        let memory = Memory::new(env.clone(), memory)?;
        let expression = Expression::new(env.clone(), expression)?;

        Ok(Self {
            _env: env,
            _instance: instance,
            target,
            memory,
            expression,
        })
    }
}

pub struct Target<'a> {
    env: JNIEnv<'a>,
    instance: JObject<'a>,
}

impl<'a> Target<'a> {
    pub(crate) const CLASS: &'static str = "Lcom/ti/debug/engine/scripting/Target;";

    pub(crate) fn new(env: JNIEnv<'a>, instance: JObject<'a>) -> Result<Self> {
        Ok(Self { env, instance })
    }

    pub fn connect(&self) -> Result<()> {
        const METHOD: &str = "connect";
        const SIGNATURE: &str = "()V";

        self.env
            .call_method(self.instance, METHOD, SIGNATURE, &[])?
            .v()?;

        Ok(())
    }

    pub fn disconnect(&self) -> Result<()> {
        const METHOD: &str = "disconnect";
        const SIGNATURE: &str = "()V";

        self.env
            .call_method(self.instance, METHOD, SIGNATURE, &[])?
            .v()?;

        Ok(())
    }

    pub fn reset(&self) -> Result<()> {
        const METHOD: &str = "reset";
        const SIGNATURE: &str = "()V";

        self.env
            .call_method(self.instance, METHOD, SIGNATURE, &[])?
            .v()?;

        Ok(())
    }

    pub fn halt(&self) -> Result<()> {
        const METHOD: &str = "halt";
        const SIGNATURE: &str = "()V";

        self.env
            .call_method(self.instance, METHOD, SIGNATURE, &[])?
            .v()?;

        Ok(())
    }

    pub fn is_halted(&self) -> Result<bool> {
        const METHOD: &str = "isHalted";
        const SIGNATURE: &str = "()Z";

        let ret = self
            .env
            .call_method(self.instance, METHOD, SIGNATURE, &[])?
            .z()?;

        Ok(ret)
    }

    pub fn run_asynch(&self) -> Result<()> {
        const METHOD: &str = "runAsynch";
        const SIGNATURE: &str = "()V";

        self.env
            .call_method(self.instance, METHOD, SIGNATURE, &[])?
            .v()?;

        Ok(())
    }
}

#[derive(Copy, Clone, Debug)]
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

impl string::ToString for Register {
    fn to_string(&self) -> String {
        let res = match self {
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
        res.to_owned()
    }
}

#[derive(Clone)]
pub struct Memory<'a> {
    env: JNIEnv<'a>,
    instance: JObject<'a>,
}

impl<'a> Memory<'a> {
    pub(crate) const CLASS: &'static str = "Lcom/ti/debug/engine/scripting/Memory;";

    pub(crate) fn new(env: JNIEnv<'a>, instance: JObject<'a>) -> Result<Self> {
        Ok(Self { env, instance })
    }

    pub fn load_raw(
        &self,
        page: jint,
        address: jlong,
        filename: &str,
        type_size: jint,
        byte_swap: jboolean,
    ) -> Result<()> {
        const METHOD: &str = "loadRaw";
        const SIGNATURE: &str = "(IJLjava/lang/String;IZ)V";

        let filename = JObject::from(self.env.new_string(filename)?);

        self.env
            .call_method(
                self.instance,
                METHOD,
                SIGNATURE,
                &[
                    From::from(page),
                    From::from(address),
                    From::from(filename),
                    From::from(type_size),
                    From::from(byte_swap),
                ],
            )?
            .v()?;

        Ok(())
    }

    pub fn write_data(
        &self,
        page: jint,
        address: jlong,
        value: jlong,
        type_size: jint,
    ) -> Result<()> {
        const METHOD: &str = "writeData";
        const SIGNATURE: &str = "(IJJI)V";

        self.env
            .call_method(
                self.instance,
                METHOD,
                SIGNATURE,
                &[
                    From::from(page),
                    From::from(address),
                    From::from(value),
                    From::from(type_size),
                ],
            )?
            .v()?;

        Ok(())
    }

    pub fn write_datas(
        &self,
        page: jint,
        address: jlong,
        values: &[jlong],
        type_size: jint,
    ) -> Result<()> {
        const METHOD: &str = "writeData";
        const SIGNATURE: &str = "(IJ[JI)V";

        let array = self.env.new_long_array(values.len() as jsize)?;
        self.env.set_long_array_region(array, 0, values)?;
        let array_obj = JObject::from(array);

        self.env
            .call_method(
                self.instance,
                METHOD,
                SIGNATURE,
                &[
                    From::from(page),
                    From::from(address),
                    From::from(array_obj),
                    From::from(type_size),
                ],
            )?
            .v()?;

        Ok(())
    }

    pub fn read_data(
        &self,
        page: jint,
        address: jlong,
        type_size: jint,
        signed: jboolean,
    ) -> Result<jlong> {
        const METHOD: &str = "readData";
        const SIGNATURE: &str = "(IJIZ)J";

        let res = self
            .env
            .call_method(
                self.instance,
                METHOD,
                SIGNATURE,
                &[
                    From::from(page),
                    From::from(address),
                    From::from(type_size),
                    From::from(signed),
                ],
            )?
            .j()?;

        Ok(res)
    }

    pub fn read_datas(
        &self,
        page: jint,
        address: jlong,
        type_size: jint,
        num_values: jint,
        signed: jboolean,
    ) -> Result<Vec<jlong>> {
        const METHOD: &str = "readData";
        const SIGNATURE: &str = "(IJIIZ)[J";

        let array_obj = self
            .env
            .call_method(
                self.instance,
                METHOD,
                SIGNATURE,
                &[
                    From::from(page),
                    From::from(address),
                    From::from(type_size),
                    From::from(num_values),
                    From::from(signed),
                ],
            )?
            .l()?;

        let array: jlongArray = array_obj.into_inner();
        let mut res = vec![0; num_values as usize];
        self.env
            .get_long_array_region(array, 0, res.as_mut_slice())?;

        Ok(res)
    }

    pub fn write_register(&self, register: Register, value: jlong) -> Result<()> {
        const METHOD: &str = "writeRegister";
        const SIGNATURE: &str = "(Ljava/lang/String;J)V";

        let register = JObject::from(self.env.new_string(register.to_string())?);

        self.env
            .call_method(
                self.instance,
                METHOD,
                SIGNATURE,
                &[From::from(register), From::from(value)],
            )?
            .v()?;

        Ok(())
    }
}

pub struct Expression<'a> {
    env: JNIEnv<'a>,
    instance: JObject<'a>,
}

impl<'a> Expression<'a> {
    pub(crate) const CLASS: &'static str = "Lcom/ti/debug/engine/scripting/Expression;";

    pub(crate) fn new(env: JNIEnv<'a>, instance: JObject<'a>) -> Result<Self> {
        Ok(Self { env, instance })
    }

    pub fn evaluate(&self, expression: &str) -> Result<jlong> {
        const METHOD: &str = "evaluate";
        const SIGNATURE: &str = "(Ljava/lang/String;)J";

        let expression = JObject::from(self.env.new_string(expression)?);

        let res = self
            .env
            .call_method(self.instance, METHOD, SIGNATURE, &[From::from(expression)])?
            .j()?;

        Ok(res)
    }
}
