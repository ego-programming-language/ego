use crate::{heap::HeapRef, types::raw::RawValue};

pub mod object;
pub mod raw;

#[derive(Debug, Clone)]
pub enum Value {
    RawValue(RawValue),
    HeapRef(HeapRef),
}

impl Value {
    pub fn to_string(&self) -> String {
        match self {
            Value::RawValue(x) => x.to_string(),
            Value::HeapRef(x) => x.get_address().to_string(),
        }
    }

    pub fn get_type(&self) -> String {
        match self {
            Value::RawValue(x) => x.get_type_string(),
            Value::HeapRef(_) => "HEAP_REF".to_string(),
        }
    }
}
