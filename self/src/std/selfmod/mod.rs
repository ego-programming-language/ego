mod members;

use crate::{heap::HeapObject, std::selfmod::members::get_stack_fn_ref};

pub fn generate_struct() -> (String, Vec<(String, HeapObject)>) {
    let mut fields = vec![];

    fields.push(("get_stack".to_string(), get_stack_fn_ref()));

    ("self".to_string(), fields)
}
