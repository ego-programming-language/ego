use crate::{std::net::types::NetStream, types::Value};

#[derive(Debug)]
pub enum NativeStruct {
    NetStream(NetStream),
}

impl NativeStruct {
    pub fn to_string(&self) -> String {
        match self {
            NativeStruct::NetStream(x) => x.to_string(),
        }
    }

    pub fn property_access(&self, property: &str) -> Option<&Value> {
        match self {
            NativeStruct::NetStream(x) => x.shape.property_access(property),
        }
    }
}
