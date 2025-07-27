use crate::{heap::HeapRef, types::Value};

pub mod func;
pub mod native_struct;
pub mod structs;

#[derive(Debug, Clone)]
pub struct BoundAccess {
    pub object: HeapRef,
    pub property: Box<Value>,
}

impl BoundAccess {
    pub fn new(object: HeapRef, property: Box<Value>) -> Self {
        BoundAccess { object, property }
    }

    pub fn to_string(&self) -> String {
        format!("property access of struct({})", self.object.get_address())
    }
}
