use crate::core::error::os_errors::OsError;
use crate::core::error::{self, VMError, VMErrorType};
use crate::memory::{Handle, MemObject};
use crate::std::heap_utils::put_string;
use crate::std::{NativeMember, NativeModuleDef};
use crate::types::object::func::{Engine, Function};
use crate::types::Value;
use crate::vm::Vm;

fn get_cwd(
    vm: &mut Vm,
    _self: Option<Handle>,
    _params: Vec<Value>,
    debug: bool,
) -> Result<Value, VMError> {
    match std::env::current_dir() {
        Ok(path) => {
            if let Some(path) = path.to_str() {
                if debug {
                    println!("OS.GET_CWD -> {}", path);
                }
                Ok(Value::Handle(put_string(vm, path.to_string())))
            } else {
                Err(error::throw(
                    VMErrorType::Os(OsError::__placeholder("non utf8 path".to_string())),
                    vm,
                ))
            }
        }
        Err(e) => Err(error::throw(
            VMErrorType::Os(OsError::__placeholder(e.to_string())),
            vm,
        )),
    }
}

pub fn generate_struct() -> (String, Vec<(String, MemObject)>) {
    (
        "os".to_string(),
        vec![(
            "get_cwd".to_string(),
            MemObject::Function(Function::new(
                "get_cwd".to_string(),
                vec![],
                Engine::Native(get_cwd),
            )),
        )],
    )
}

pub fn generate_mod_def() -> NativeModuleDef {
    NativeModuleDef {
        module: "os".to_string(),
        members: vec![NativeMember {
            name: "get_cwd".to_string(),
            description: "get the current working directory".to_string(),
            params: None,
        }],
    }
}
