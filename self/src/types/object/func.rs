use crate::{core::error::VMError, memory::Handle, types::Value, vm::Vm};

#[derive(Debug, Clone)]
pub enum Engine {
    Bytecode(Vec<u8>),
    Native(fn(&mut Vm, Option<Handle>, Vec<Value>, bool) -> Result<Value, VMError>),
}

#[derive(Debug, Clone)]
pub struct Function {
    pub identifier: String,
    pub parameters: Vec<String>,
    pub engine: Engine,
}

impl Function {
    pub fn new(identifier: String, parameters: Vec<String>, engine: Engine) -> Function {
        Function {
            identifier,
            parameters,
            engine,
        }
    }
    pub fn to_string(&self) -> String {
        self.identifier.clone()
    }
}
