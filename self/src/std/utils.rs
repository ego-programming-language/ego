use serde_json::Value as JsonValue;

use crate::types::{
    raw::{bool::Bool, utf8::Utf8, RawValue},
    Value,
};

pub fn cast_json_value(json: &JsonValue) -> Option<Value> {
    match json {
        JsonValue::String(x) => Some(Value::RawValue(RawValue::Utf8(Utf8::new(x.clone())))),
        JsonValue::Bool(x) => Some(Value::RawValue(RawValue::Bool(Bool::new(x.clone())))),
        _ => None,
    }
}
