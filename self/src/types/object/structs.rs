use crate::opcodes::DataType;

#[derive(Debug, Clone)]
pub struct StructDeclaration {
    pub identifier: String,
    pub fields: Vec<(String, DataType)>,
}

impl StructDeclaration {
    pub fn new(identifier: String, fields: Vec<(String, DataType)>) -> StructDeclaration {
        StructDeclaration { identifier, fields }
    }
    pub fn to_string(&self) -> String {
        self.identifier.clone()
    }
}
