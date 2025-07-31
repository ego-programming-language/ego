use crate::{
    heap::HeapRef,
    types::{
        raw::{utf8::Utf8, RawValue},
        Value,
    },
};

#[derive(Debug)]
pub struct Action {
    pub module: String,
    pub member: String,
    pub exec: HeapRef, // heap_ref to the executor function
    pub args: Vec<Value>,
}

impl Action {
    pub fn new(module: String, exec: HeapRef, member: String, args: Vec<Value>) -> Action {
        Action {
            module,
            exec,
            member,
            args,
        }
    }

    pub fn to_string(&self) -> String {
        format!("Action({}, {})", self.module, self.member)
    }

    pub fn property_access(&self, property: &str) -> Option<Value> {
        match property {
            "module" => Some(Value::RawValue(RawValue::Utf8(Utf8::new(
                self.module.clone(),
            )))),
            "member" => Some(Value::RawValue(RawValue::Utf8(Utf8::new(
                self.member.clone(),
            )))),
            "exec" => Some(Value::HeapRef(self.exec.clone())),
            //"params" => self.params,
            _ => None,
        }
    }
}
