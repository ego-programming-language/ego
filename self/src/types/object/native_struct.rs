use crate::{
    std::{ai::types::Action, net::types::NetStream},
    types::Value,
};

#[derive(Debug)]
pub enum NativeStruct {
    NetStream(NetStream),
    Action(Action),
}

impl NativeStruct {
    pub fn to_string(&self) -> String {
        match self {
            NativeStruct::NetStream(x) => x.to_string(),
            NativeStruct::Action(x) => x.to_string(),
        }
    }

    pub fn property_access(&self, property: &str) -> Option<Value> {
        // here the property accesses values are owned. we're
        // bringing or the ref to the value or the value
        // itself
        match self {
            NativeStruct::NetStream(x) => x.shape.property_access(property),
            NativeStruct::Action(x) => x.property_access(property),
        }
    }
}
