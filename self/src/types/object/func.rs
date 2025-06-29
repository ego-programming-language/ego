#[derive(Debug)]
pub struct Function {
    identifier: String,
    bytecode: Vec<u8>,
}

impl Function {
    pub fn new(identifier: String, bytecode: Vec<u8>) -> Function {
        Function {
            identifier,
            bytecode,
        }
    }
    pub fn to_string(&self) -> &String {
        &self.identifier
    }
}
