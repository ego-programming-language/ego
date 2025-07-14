use crate::{
    core::error::{VMError, VMErrorType},
    types::{raw::RawValue, Value},
    vm::Vm,
};

pub fn infer(vm: &mut Vm, params: Vec<Value>) -> Result<Value, VMError> {
    println!("infering with ai");
    return Ok(Value::RawValue(RawValue::Nothing));
}
