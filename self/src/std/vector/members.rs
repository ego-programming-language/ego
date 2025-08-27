use crate::{
    core::error::{self, type_errors::TypeError, VMError, VMErrorType},
    heap::HeapRef,
    memory::MemObject,
    types::{
        object::func::{Engine, Function},
        raw::{u32::U32, RawValue},
        Value,
    },
    vm::Vm,
};

pub fn len_obj() -> MemObject {
    MemObject::Function(Function::new(
        "len".to_string(),
        vec![],
        Engine::Native(len),
    ))
}

fn len(
    vm: &mut Vm,
    _self: Option<HeapRef>,
    params: Vec<Value>,
    debug: bool,
) -> Result<Value, VMError> {
    // resolve 'self'
    let _self = if let Some(_this) = _self {
        if let MemObject::Vector(vec) = vm.resolve_heap_mut_ref(_this) {
            vec
        } else {
            unreachable!()
        }
    } else {
        unreachable!()
    };

    Ok(Value::RawValue(RawValue::U32(U32::new(
        _self.elements.len() as u32,
    ))))
}

// map
pub fn map_obj() -> MemObject {
    MemObject::Function(Function::new(
        "map".to_string(),
        vec!["callback".to_string()],
        Engine::Native(map),
    ))
}

fn map(
    vm: &mut Vm,
    _self: Option<HeapRef>,
    params: Vec<Value>,
    debug: bool,
) -> Result<Value, VMError> {
    // resolve 'self'
    let _self = if let Some(_this) = _self {
        if let MemObject::Vector(vec) = vm.resolve_heap_ref(_this) {
            vec.clone()
        } else {
            unreachable!()
        }
    } else {
        unreachable!()
    };

    let callback = params[0].as_function_obj(vm)?;
    if callback.parameters.len() < 1 {
        return Err(error::throw(VMErrorType::TypeError(
            TypeError::InvalidArgsCount {
                expected: 1,
                received: 0,
            },
        )));
    }

    for ele in &_self.elements {
        let exec_result = vm.run_function(&callback, None, vec![ele.clone()], debug);
        if let Some(err) = exec_result.error {
            return Err(err);
        }

        // if we make this, we will have a vector with multiples
        // value types. i don't think is a good a idea to have a
        // vector with polimorfism. for the moment we'll return
        // nothing
        // ----
        //
        // match exec_result.result {
        //     Some(v) => v
        //     None => value::Nothing..
        // }
    }

    Ok(Value::RawValue(RawValue::Nothing))
}
