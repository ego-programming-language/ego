use crate::{
    core::error::{self, type_errors::TypeError, VMError, VMErrorType},
    heap::HeapRef,
    memory::{Handle, MemObject},
    types::{
        object::func::{Engine, Function},
        raw::RawValue,
        Value,
    },
    vm::Vm,
};
use std::env;

// environment variable set
pub fn set_obj() -> MemObject {
    MemObject::Function(Function::new(
        "set".to_string(),
        vec!["key".to_string(), "value".to_string()],
        Engine::Native(set),
    ))
}

pub fn set(
    vm: &mut Vm,
    _self: Option<Handle>,
    params: Vec<Value>,
    debug: bool,
) -> Result<Value, VMError> {
    if params.len() < 2 {
        return Err(error::throw(
            VMErrorType::TypeError(TypeError::InvalidArgsCount {
                expected: 2,
                received: params.len() as u32,
            }),
            vm,
        ));
    }

    let key = &params[0].as_string_obj(vm)?;
    let value = &params[1].as_string_obj(vm)?;

    if debug {
        println!("ENV_SET -> {}({})", key, value)
    }
    env::set_var(key, value);
    Ok(Value::RawValue(RawValue::Nothing))
}

// get environment variables
pub fn get_obj() -> MemObject {
    MemObject::Function(Function::new(
        "get".to_string(),
        vec!["key".to_string()],
        Engine::Native(get),
    ))
}

pub fn get(
    vm: &mut Vm,
    _self: Option<Handle>,
    params: Vec<Value>,
    debug: bool,
) -> Result<Value, VMError> {
    if params.len() < 1 {
        return Err(error::throw(
            VMErrorType::TypeError(TypeError::InvalidArgsCount {
                expected: 1,
                received: params.len() as u32,
            }),
            vm,
        ));
    }

    let key = &params[0].as_string_obj(vm)?;

    if debug {
        println!("ENV_GET -> {}", key)
    }
    let var = env::var(key);
    match var {
        Ok(v) => {
            let value_ref = vm.heap.allocate(MemObject::String(v));
            Ok(Value::HeapRef(value_ref))
        }
        Err(_) => Ok(Value::RawValue(RawValue::Nothing)),
    }
}
