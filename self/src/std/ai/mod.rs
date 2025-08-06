mod members;
mod prompts;
mod providers;
pub mod types;

use crate::{
    heap::HeapObject,
    std::ai::members::{do_fn, infer},
    types::object::func::{Engine, Function},
};

pub fn generate_struct() -> (String, Vec<(String, HeapObject)>) {
    let mut fields = vec![];
    let infer_ref = HeapObject::Function(Function::new(
        "infer".to_string(),
        vec![], // TODO: load params to native functions
        Engine::Native(infer),
    ));
    let do_ref = HeapObject::Function(Function::new(
        "do".to_string(),
        vec![], // TODO: load params to native functions
        Engine::Native(do_fn),
    ));
    fields.push(("infer".to_string(), infer_ref));
    fields.push(("do".to_string(), do_ref));

    ("ai".to_string(), fields)
}
