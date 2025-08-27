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
    memory::MemObject,
    types::{
        object::func::{Engine, Function},
        raw::RawValue,
        Value,
    },
    vm::Vm,
};

// read_file
pub fn read_file_def() -> NativeMember {
    NativeMember {
        name: "read_file".to_string(),
        description: "read a file on the host filesystem on the given path.".to_string(),
        params: Some(vec!["path(string)".to_string()]),
    }
}

pub fn read_file_obj() -> MemObject {
    MemObject::Function(Function::new(
        "read_file".to_string(),
        vec![], // TODO: load params to native functions
        Engine::Native(read_file),
    ))
}

pub fn read_file(
    vm: &mut Vm,
    _self: Option<HeapRef>,
    params: Vec<Value>,
    debug: bool,
) -> Result<Value, VMError> {
    let path = params[0].as_string_obj(vm)?;
    let path_obj = Path::new(&path);
    if !path_obj.exists() {
        return Err(error::throw(
            VMErrorType::Fs(FsError::FileNotFound(format!("{}", path))),
            vm,
        ));
    }
    if !path_obj.is_file() {
        return Err(error::throw(
            VMErrorType::Fs(FsError::NotAFile(format!("{}", path))),
            vm,
        ));
    }

    match fs::read_to_string(path_obj) {
        Ok(content) => Ok(Value::HeapRef(put_string(vm, content))),
        Err(_) => Err(error::throw(
            VMErrorType::Fs(FsError::ReadError(format!("{}", path))),
            vm,
        )),
    }
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

pub fn write_file_obj() -> MemObject {
    MemObject::Function(Function::new(
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
        return Err(error::throw(
            VMErrorType::TypeError(TypeError::InvalidArgsCount {
                expected: 2,
                received: params.len() as u32,
            }),
            vm,
        ));
    }

    let path = &params[0].as_string_obj(vm)?;
    let content = &params[1].as_string_obj(vm)?;

    let overwrite_or_create = if let Some(param2) = params.get(2) {
        match param2 {
            Value::RawValue(RawValue::Bool(b)) => b.value,
            _ => {
                return Err(error::throw(
                    VMErrorType::TypeMismatch {
                        expected: "bool".to_string(),
                        received: param2.get_type(),
                    },
                    vm,
                ))
            }
        }
    } else {
        false // default if not passed
    };

    let path_obj = Path::new(path);

    if !path_obj.exists() && !overwrite_or_create {
        return Err(error::throw(
            VMErrorType::Fs(FsError::FileNotFound(path.to_string())),
            vm,
        ));
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
                    Err(error::throw(
                        VMErrorType::Fs(FsError::WriteError(path.to_string())),
                        vm,
                    ))
                }
            }
        }
        Err(err) => {
            println!("err{:#?}", err);
            Err(error::throw(
                VMErrorType::Fs(FsError::WriteError(path.to_string())),
                vm,
            ))
        }
    }
}

// delete_file
pub fn delete_def() -> NativeMember {
    NativeMember {
        name: "delete".to_string(), 
        description: "delete a file or a folder on the host filesystem on the given path. The second parameter serves as a flag to delete folders (recursively) or not".to_string(), 
        params: Some(vec![
            "path(string)".to_string(),
            "delete_folder_recursively(string)".to_string(),
        ])
    }
}

pub fn delete_obj() -> MemObject {
    MemObject::Function(Function::new(
        "delete".to_string(),
        vec!["path".to_string()],
        Engine::Native(delete),
    ))
}

pub fn delete(
    vm: &mut Vm,
    _self: Option<HeapRef>,
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

    let path = &params[0].as_string_obj(vm)?;
    let remove_recursively = if let Some(param2) = params.get(1) {
        param2.as_bool(vm)?
    } else {
        false // default if not passed
    };

    let path_obj = Path::new(path);
    if !path_obj.exists() {
        return Err(error::throw(
            VMErrorType::Fs(FsError::FileNotFound(path.to_string())),
            vm,
        ));
    }

    let op_result = if remove_recursively {
        fs::remove_dir_all(path_obj)
    } else {
        fs::remove_file(path_obj)
    };

    match op_result {
        Ok(_) => Ok(Value::RawValue(RawValue::Bool(Bool::new(true)))),
        Err(_) => Err(error::throw(
            VMErrorType::Fs(FsError::DeleteError(path.to_string())),
            vm,
        )),
    }
}
