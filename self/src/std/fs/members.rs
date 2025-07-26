use std::fs;
use std::path::Path;

use crate::core::error::fs_errors::FsError;
use crate::core::error::{self, VMErrorType};
use crate::std::heap_utils::put_string;
use crate::{
    core::error::VMError,
    heap::HeapObject,
    types::{
        object::func::{Engine, Function},
        raw::RawValue,
        Value,
    },
    vm::Vm,
};

pub fn read_file(vm: &mut Vm, params: Vec<Value>, debug: bool) -> Result<Value, VMError> {
    let path_ref = params[0].clone();
    let path = match path_ref {
        Value::HeapRef(r) => {
            let heap_obj = vm.resolve_heap_ref(r);
            let request = match heap_obj {
                HeapObject::String(s) => s,
                _ => {
                    return Err(error::throw(VMErrorType::TypeMismatch {
                        expected: "string".to_string(),
                        received: heap_obj.to_string(),
                    }));
                }
            };
            request
        }
        Value::RawValue(r) => {
            return Err(error::throw(VMErrorType::TypeMismatch {
                expected: "string".to_string(),
                received: r.get_type_string(),
            }));
        }
        Value::BoundAccess(_) => {
            return Err(error::throw(VMErrorType::TypeMismatch {
                expected: "string".to_string(),
                received: "bound_access".to_string(),
            }));
        }
    };

    let path_obj = Path::new(path);
    if !path_obj.exists() {
        return Err(error::throw(VMErrorType::Fs(FsError::FileNotFound(
            format!("{}", path),
        ))));
    }
    if !path_obj.is_file() {
        return Err(error::throw(VMErrorType::Fs(FsError::NotAFile(format!(
            "{}",
            path
        )))));
    }

    match fs::read_to_string(path_obj) {
        Ok(content) => Ok(Value::HeapRef(put_string(vm, content))),
        Err(_) => Err(error::throw(VMErrorType::Fs(FsError::ReadError(format!(
            "{}",
            path
        ))))),
    }
}

pub fn read_file_ref() -> HeapObject {
    HeapObject::Function(Function::new(
        "read_file".to_string(),
        vec![], // TODO: load params to native functions
        Engine::Native(read_file),
    ))
}
