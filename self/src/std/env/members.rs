use crate::{
    core::error::{self, type_errors::TypeError, VMError, VMErrorType},
    heap::HeapRef,
    memory::MemObject,
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
    _self: Option<HeapRef>,
    params: Vec<Value>,
    debug: bool,
) -> Result<Value, VMError> {
    if params.len() < 2 {
        return Err(error::throw(VMErrorType::TypeError(
            TypeError::InvalidArgsCount {
                expected: 2,
                received: params.len() as u32,
            },
        )));
    }

    let key = match &params[0] {
        Value::HeapRef(r) => {
            let heap_obj = vm.resolve_heap_ref(r.clone());
            match heap_obj {
                MemObject::String(s) => s,
                _ => {
                    return Err(error::throw(VMErrorType::TypeMismatch {
                        expected: "string".to_string(),
                        received: heap_obj.to_string(vm),
                    }))
                }
            }
        }
        Value::RawValue(RawValue::Utf8(s)) => &s.value,
        _ => {
            return Err(error::throw(VMErrorType::TypeMismatch {
                expected: "string".to_string(),
                received: params[0].get_type(),
            }))
        }
    };

    let value = match &params[1] {
        Value::HeapRef(r) => {
            let heap_obj = vm.resolve_heap_ref(r.clone());
            match heap_obj {
                MemObject::String(s) => s,
                _ => {
                    return Err(error::throw(VMErrorType::TypeMismatch {
                        expected: "string".to_string(),
                        received: heap_obj.to_string(vm),
                    }))
                }
            }
        }
        Value::RawValue(RawValue::Utf8(s)) => &s.value,
        _ => {
            return Err(error::throw(VMErrorType::TypeMismatch {
                expected: "string".to_string(),
                received: params[0].get_type(),
            }))
        }
    };

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
    _self: Option<HeapRef>,
    params: Vec<Value>,
    debug: bool,
) -> Result<Value, VMError> {
    if params.len() < 1 {
        return Err(error::throw(VMErrorType::TypeError(
            TypeError::InvalidArgsCount {
                expected: 1,
                received: params.len() as u32,
            },
        )));
    }

    let key = match &params[0] {
        Value::HeapRef(r) => {
            let heap_obj = vm.resolve_heap_ref(r.clone());
            match heap_obj {
                MemObject::String(s) => s,
                _ => {
                    return Err(error::throw(VMErrorType::TypeMismatch {
                        expected: "string".to_string(),
                        received: heap_obj.to_string(vm),
                    }))
                }
            }
        }
        Value::RawValue(RawValue::Utf8(s)) => &s.value,
        _ => {
            return Err(error::throw(VMErrorType::TypeMismatch {
                expected: "string".to_string(),
                received: params[0].get_type(),
            }))
        }
    };

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
