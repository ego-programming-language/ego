/*
HERE WE DEFINE THE LOGIC OF THE AI STD MODULE.
CURRENTLY WE ARE USING THE OPENAI LLM
BUT WE COULD IN THE FUTURE IMPLEMENT ANOTHER
PROVIDER OR ENABLE USER IMPLEMENTATION OF
PROVIDER.
*/

use crate::{
    core::error::VMError,
    heap::{HeapObject, HeapRef},
    types::{
        object::func::{Engine, Function},
        raw::RawValue,
        Value,
    },
    vm::Vm,
};

pub fn get_stack(
    vm: &mut Vm,
    _self: Option<HeapRef>,
    params: Vec<Value>,
    debug: bool,
) -> Result<Value, VMError> {
    Ok(Value::RawValue(RawValue::Nothing))
}

pub fn get_stack_fn_ref() -> HeapObject {
    HeapObject::Function(Function::new(
        "get_stack".to_string(),
        vec![], // TODO: load params to native functions
        Engine::Native(get_stack),
    ))
}
