use crate::{
    core::error::{self, VMError, VMErrorType},
    heap::HeapRef,
    memory::{Handle, MemObject},
    types::{
        object::{func::Function, BoundAccess},
        raw::RawValue,
    },
    vm::Vm,
};

pub mod object;
pub mod raw;

#[derive(Debug, Clone)]
pub enum Value {
    RawValue(RawValue),
    HeapRef(HeapRef),
    Handle(Handle),
    BoundAccess(BoundAccess),
}

impl Value {
    pub fn to_string(&self, vm: &Vm) -> String {
        match self {
            Value::RawValue(x) => x.to_string(),
            Value::HeapRef(x) => {
                if let Some(obj) = vm.heap.get(x.clone()) {
                    obj.to_string(vm)
                } else {
                    "unkown_value_type".to_string()
                }
            }
            Value::BoundAccess(x) => x.to_string(),
            Value::Handle(x) => vm.memory.resolve(x).to_string(vm),
            _ => "unkown_value_type".to_string(),
        }
    }

    pub fn get_type(&self) -> String {
        match self {
            Value::RawValue(x) => x.get_type_string(),
            Value::HeapRef(_) => "HEAP_REF".to_string(),
            Value::BoundAccess(_) => "BOUND_ACCESS".to_string(),
            Value::Handle(_) => "HANDLE".to_string(),
            _ => "unkown_value_type".to_string(),
        }
    }

    pub fn as_mem_obj<'vm>(&self, vm: &'vm Vm) -> Result<&'vm MemObject, VMError> {
        match self {
            Value::Handle(v) => Ok(vm.memory.resolve(&v)),
            Value::BoundAccess(_) => {
                panic!("TODO: implement structs from bound acceses ")
            }
            _ => {
                // TODO: use self-vm errors system
                panic!("invalid struct type")
            }
        }
    }

    pub fn as_string_obj(&self, vm: &Vm) -> Result<String, VMError> {
        match self {
            Value::Handle(r) => {
                let heap_obj = vm.memory.resolve(&r);
                let request = match heap_obj {
                    MemObject::String(s) => s,
                    _ => {
                        return Err(error::throw(
                            VMErrorType::TypeMismatch {
                                expected: "string".to_string(),
                                received: heap_obj.to_string(vm),
                            },
                            vm,
                        ));
                    }
                };
                Ok(request.clone())
            }
            Value::RawValue(r) => match r {
                RawValue::Utf8(s) => Ok(s.value.clone()),
                _ => {
                    return Err(error::throw(
                        VMErrorType::TypeMismatch {
                            expected: "string".to_string(),
                            received: r.get_type_string(),
                        },
                        vm,
                    ));
                }
            },
            Value::BoundAccess(_) => {
                return Err(error::throw(
                    VMErrorType::TypeMismatch {
                        expected: "string".to_string(),
                        received: "bound_access".to_string(),
                    },
                    vm,
                ));
            }
            Value::Handle(_) => {
                return Err(error::throw(
                    VMErrorType::TypeMismatch {
                        expected: "string".to_string(),
                        received: "handle".to_string(),
                    },
                    vm,
                ));
            }
            _ => {
                return Err(error::throw(
                    VMErrorType::TypeMismatch {
                        expected: "string".to_string(),
                        received: "unknown_type".to_string(),
                    },
                    vm,
                ));
            }
        }
    }

    pub fn as_function_obj(&self, vm: &Vm) -> Result<Function, VMError> {
        match self {
            Value::Handle(r) => {
                let heap_obj = vm.memory.resolve(&r);
                let request = match heap_obj {
                    MemObject::Function(f) => f.clone(),
                    _ => {
                        return Err(error::throw(
                            VMErrorType::TypeMismatch {
                                expected: "function".to_string(),
                                received: heap_obj.to_string(vm),
                            },
                            vm,
                        ));
                    }
                };
                Ok(request)
            }
            Value::RawValue(_) => {
                return Err(error::throw(
                    VMErrorType::TypeMismatch {
                        expected: "function".to_string(),
                        received: "raw_value".to_string(),
                    },
                    vm,
                ));
            }
            Value::BoundAccess(_) => {
                return Err(error::throw(
                    VMErrorType::TypeMismatch {
                        expected: "function".to_string(),
                        received: "bound_access".to_string(),
                    },
                    vm,
                ));
            }
            Value::Handle(_) => {
                return Err(error::throw(
                    VMErrorType::TypeMismatch {
                        expected: "function".to_string(),
                        received: "handle".to_string(),
                    },
                    vm,
                ));
            }
            _ => {
                return Err(error::throw(
                    VMErrorType::TypeMismatch {
                        expected: "string".to_string(),
                        received: "unknown_type".to_string(),
                    },
                    vm,
                ));
            }
        }
    }

    pub fn as_bool(&self, vm: &Vm) -> Result<bool, VMError> {
        match self {
            Value::RawValue(r) => match r {
                RawValue::Bool(v) => Ok(v.value),
                _ => {
                    return Err(error::throw(
                        VMErrorType::TypeMismatch {
                            expected: "string".to_string(),
                            received: r.get_type_string(),
                        },
                        vm,
                    ));
                }
            },
            Value::HeapRef(r) => {
                return Err(error::throw(
                    VMErrorType::TypeMismatch {
                        expected: "string".to_string(),
                        received: "heap_ref".to_string(),
                    },
                    vm,
                ));
            }
            Value::BoundAccess(_) => {
                return Err(error::throw(
                    VMErrorType::TypeMismatch {
                        expected: "string".to_string(),
                        received: "bound_access".to_string(),
                    },
                    vm,
                ));
            }
            Value::Handle(_) => {
                return Err(error::throw(
                    VMErrorType::TypeMismatch {
                        expected: "string".to_string(),
                        received: "handle".to_string(),
                    },
                    vm,
                ));
            }
            _ => {
                return Err(error::throw(
                    VMErrorType::TypeMismatch {
                        expected: "string".to_string(),
                        received: "unknown_type".to_string(),
                    },
                    vm,
                ));
            }
        }
    }
}
