mod members;

use crate::{memory::MemObject, std::selfmod::members::get_stack_fn_ref};

pub fn generate_struct() -> (String, Vec<(String, MemObject)>) {
    let mut fields = vec![];

    fields.push(("get_stack".to_string(), get_stack_fn_ref()));

    ("self".to_string(), fields)
}
