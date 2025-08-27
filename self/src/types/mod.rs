use crate::{
    heap::HeapRef,
    memory::Handle,
    types::{object::BoundAccess, raw::RawValue},
};

pub mod object;
pub mod raw;

#[derive(Debug, Clone)]
pub enum Value {
    RawValue(RawValue),
    HeapRef(HeapRef),
    Handle(Handle),
    BoundAccess(BoundAccess),
}

impl Value {
    pub fn to_string(&self) -> String {
        match self {
            Value::RawValue(x) => x.to_string(),
            Value::HeapRef(x) => x.get_address().to_string(),
            Value::BoundAccess(x) => x.to_string(),
            Value::Handle(x) => x.to_string(),
        }
    }

    pub fn get_type(&self) -> String {
        match self {
            Value::RawValue(x) => x.get_type_string(),
            Value::HeapRef(_) => "HEAP_REF".to_string(),
            Value::BoundAccess(_) => "BOUND_ACCESS".to_string(),
            Value::Handle(_) => "HANDLE".to_string(),
        }
    }
}
