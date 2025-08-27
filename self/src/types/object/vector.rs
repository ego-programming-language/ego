use std::collections::HashMap;

use crate::{types::Value, vm::Vm};

#[derive(Debug, Clone)]
pub struct Vector {
    pub elements: Vec<Value>,
    pub members: HashMap<String, Value>,
}

impl Vector {
    pub fn new(elements: Vec<Value>) -> Vector {
        Vector {
            elements,
            members: HashMap::new(),
        }
    }

    pub fn init_vector_members(&mut self, members: HashMap<String, Value>) {
        self.members = members
    }

    pub fn to_string(&self, vm: &Vm) -> String {
        let elements: Vec<String> = self.elements.iter().map(|ele| ele.to_string(vm)).collect();
        format!("{:#?}", elements)
    }

    pub fn property_access(&self, property: &str) -> Option<Value> {
        self.members.get(property).cloned()
    }
}
