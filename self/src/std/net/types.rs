use std::{collections::HashMap, net::TcpStream};

use crate::types::{object::structs::StructLiteral, Value};

#[derive(Debug)]
pub struct NetStream {
    pub host: String,
    pub stream: TcpStream,
    pub shape: StructLiteral,
}

impl NetStream {
    pub fn new(host: String, stream: TcpStream, shape: HashMap<String, Value>) -> NetStream {
        NetStream {
            host,
            stream,
            shape: StructLiteral::new("NetStream".to_string(), shape),
        }
    }

    pub fn to_string(&self) -> String {
        "NetStream".to_string()
    }
}
