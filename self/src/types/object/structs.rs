use std::collections::HashMap;

use crate::{opcodes::DataType, types::Value};

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

#[derive(Debug, Clone)]
pub struct StructLiteral {
    pub identifier: String,
    pub fields: HashMap<String, Value>,
}

impl StructLiteral {
    pub fn new(identifier: String, fields: HashMap<String, Value>) -> StructLiteral {
        StructLiteral { identifier, fields }
    }

    pub fn property_access(&self, property: &str) -> Option<Value> {
        self.fields.get(property).cloned()
    }

    pub fn to_string(&self) -> String {
        format!("[instance] {}", self.identifier)
    }
}
