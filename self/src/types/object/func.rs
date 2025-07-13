#[derive(Debug, Clone)]
pub struct Function {
    pub identifier: String,
    pub parameters: Vec<String>,
    pub bytecode: Vec<u8>,
}

impl Function {
    pub fn new(identifier: String, parameters: Vec<String>, bytecode: Vec<u8>) -> Function {
        Function {
            identifier,
            parameters,
            bytecode,
        }
    }
    pub fn to_string(&self) -> String {
        self.identifier.clone()
    }
}
