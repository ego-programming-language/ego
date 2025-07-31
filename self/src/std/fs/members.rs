use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::Path;

use crate::core::error::fs_errors::FsError;
use crate::core::error::type_errors::TypeError;
use crate::core::error::{self, VMErrorType};
use crate::heap::HeapRef;
use crate::std::heap_utils::put_string;
use crate::std::NativeMember;
use crate::types::raw::bool::Bool;
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

// read_file
pub fn read_file(
    vm: &mut Vm,
    _self: Option<HeapRef>,
    params: Vec<Value>,
    debug: bool,
) -> Result<Value, VMError> {
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

pub fn read_file_obj() -> HeapObject {
    HeapObject::Function(Function::new(
        "read_file".to_string(),
        vec![], // TODO: load params to native functions
        Engine::Native(read_file),
    ))
}

// write_file
pub fn write_file_def() -> NativeMember {
    NativeMember {
        name: "write_file".to_string(), 
        description: "write a file on the host filesystem on the given path. It can also create files depeding on the third flag".to_string(), 
        params: Some(vec![
            "path(string)".to_string(),
            "content(string)".to_string(),
            "create_or_overwrite(bool)".to_string(),
        ])
    }
}

pub fn write_file_obj() -> HeapObject {
    HeapObject::Function(Function::new(
        "write_file".to_string(),
        vec![
            "path".to_string(),
            "content".to_string(),
            "create_or_overwrite".to_string(),
        ],
        Engine::Native(write_file),
    ))
}

pub fn write_file(
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

    let path = match &params[0] {
        Value::HeapRef(r) => {
            let heap_obj = vm.resolve_heap_ref(r.clone());
            match heap_obj {
                HeapObject::String(s) => s,
                _ => {
                    return Err(error::throw(VMErrorType::TypeMismatch {
                        expected: "string".to_string(),
                        received: heap_obj.to_string(),
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

    let content = match &params[1] {
        Value::HeapRef(r) => {
            let heap_obj = vm.resolve_heap_ref(r.clone());
            match heap_obj {
                HeapObject::String(s) => s,
                _ => {
                    return Err(error::throw(VMErrorType::TypeMismatch {
                        expected: "string".to_string(),
                        received: heap_obj.to_string(),
                    }))
                }
            }
        }
        Value::RawValue(RawValue::Utf8(s)) => &s.value,
        _ => {
            return Err(error::throw(VMErrorType::TypeMismatch {
                expected: "string".to_string(),
                received: params[1].get_type(),
            }))
        }
    };

    let overwrite_or_create = if let Some(param2) = params.get(2) {
        match param2 {
            Value::RawValue(RawValue::Bool(b)) => b.value,
            _ => {
                return Err(error::throw(VMErrorType::TypeMismatch {
                    expected: "bool".to_string(),
                    received: param2.get_type(),
                }))
            }
        }
    } else {
        false // default if not passed
    };

    let path_obj = Path::new(path);

    if !path_obj.exists() && !overwrite_or_create {
        return Err(error::throw(VMErrorType::Fs(FsError::FileNotFound(
            path.to_string(),
        ))));
    }

    let file = if overwrite_or_create {
        OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(path_obj)
    } else {
        OpenOptions::new().append(true).open(path_obj)
    };

    match file {
        Ok(mut f) => {
            let write_result = f.write(content.as_bytes());
            match write_result {
                Ok(_) => Ok(Value::RawValue(RawValue::Bool(Bool::new(true)))),
                Err(err) => {
                    println!("err{:#?}", err);
                    Err(error::throw(VMErrorType::Fs(FsError::WriteError(
                        path.to_string(),
                    ))))
                }
            }
        }
        Err(err) => {
            println!("err{:#?}", err);
            Err(error::throw(VMErrorType::Fs(FsError::WriteError(
                path.to_string(),
            ))))
        }
    }
}
