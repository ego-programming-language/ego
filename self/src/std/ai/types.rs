use crate::types::{
    raw::{utf8::Utf8, RawValue},
    Value,
};

#[derive(Debug)]
pub struct Action {
    pub module: String,
    pub member: String,
    pub params: Vec<Value>,
}

impl Action {
    pub fn new(module: String, member: String, params: Vec<Value>) -> Action {
        Action {
            module,
            member,
            params,
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
            //"params" => self.params,
            _ => None,
        }
    }
}
