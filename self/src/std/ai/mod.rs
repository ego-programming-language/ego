use crate::{
    heap::HeapObject,
    types::object::func::{Engine, Function},
};

pub fn generate_struct() -> (String, Vec<(String, HeapObject)>) {
    let mut fields = vec![];
    let infer_ref =
        HeapObject::Function(Function::new("infer".to_string(), vec![], Engine::Native));
    fields.push(("infer".to_string(), infer_ref));

    ("ai".to_string(), fields)
}
