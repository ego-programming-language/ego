#[derive(Debug, Clone)]
pub enum Engine {
    Bytecode(Vec<u8>),
    //Native(fn(&mut VM, Vec<Value>) -> Result<Value, VMError>),
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
