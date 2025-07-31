use std::collections::HashMap;

use crate::{
    heap::HeapObject,
    types::{object::vector::Vector, Value},
    vm::Vm,
};
mod members;

pub fn init_lib() -> Vec<(String, HeapObject)> {
    let mut fields = vec![];

    fields.push(("vector.len".to_string(), members::len_obj()));

    fields
}

pub fn init_vector_members(vector: &mut Vector, vm: &Vm) {
    let mut members = HashMap::new();
    if let Some(mem) = vm.get_handler("vector.len") {
        members.insert("len".to_string(), Value::HeapRef(mem));
    }

    vector.init_vector_members(members);
}
