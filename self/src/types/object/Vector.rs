use std::collections::HashMap;

use crate::types::Value;

#[derive(Debug, Clone)]
pub struct Vector {
    pub elements: Vec<Value>,
}

impl Vector {
    pub fn new(elements: Vec<Value>) -> Vector {
        Vector { elements }
    }

    pub fn to_string(&self) -> String {
        format!("elements[{}]", self.elements.len())
    }
}
