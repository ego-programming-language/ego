mod members;
pub mod types;

use crate::{heap::HeapObject, std::net::members::connect_ref};

pub fn generate_struct() -> (String, Vec<(String, HeapObject)>) {
    let mut fields = vec![];

    fields.push(("connect".to_string(), connect_ref()));

    ("net".to_string(), fields)
}
