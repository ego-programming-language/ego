mod members;

use crate::{heap::HeapObject, std::fs::members::read_file_ref};

pub fn generate_struct() -> (String, Vec<(String, HeapObject)>) {
    let mut fields = vec![];

    fields.push(("read_file".to_string(), read_file_ref()));

    ("self".to_string(), fields)
}
