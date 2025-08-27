use crate::{memory::Handle, types::Value};

pub mod func;
pub mod native_struct;
pub mod structs;
pub mod vector;

#[derive(Debug, Clone)]
pub struct BoundAccess {
    pub object: Handle,
    pub property: Box<Value>,
}

impl BoundAccess {
    pub fn new(object: Handle, property: Box<Value>) -> Self {
        BoundAccess { object, property }
    }

    pub fn to_string(&self) -> String {
        format!("property access of struct({})", self.object.pointer)
    }
}
