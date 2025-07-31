use std::collections::HashMap;

use crate::{
    core::error::VMError,
    heap::{HeapObject, HeapRef},
    types::{
        raw::{u32::U32, RawValue},
        Value,
    },
    vm::Vm,
};

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

    pub fn to_string(&self) -> String {
        format!("elements[{}]", self.elements.len())
    }

    pub fn property_access(&self, property: &str) -> Option<Value> {
        self.members.get(property).cloned()
    }
}
