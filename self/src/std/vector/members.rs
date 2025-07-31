use crate::{
    core::error::VMError,
    heap::{HeapObject, HeapRef},
    types::{
        object::func::{Engine, Function},
        raw::{u32::U32, RawValue},
        Value,
    },
    vm::Vm,
};

pub fn len_obj() -> HeapObject {
    HeapObject::Function(Function::new(
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
        if let HeapObject::Vector(vec) = vm.resolve_heap_mut_ref(_this) {
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
