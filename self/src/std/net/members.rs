use std::collections::HashMap;
use std::io::{Read, Write};
use std::net::TcpStream;

use crate::core::error::net_errors::NetErrors;
use crate::core::error::{self, VMErrorType};
use crate::heap::{self, HeapRef};
use crate::std::net::types::NetStream;
use crate::types::object::native_struct::NativeStruct;
use crate::types::raw::u64::U64;
use crate::types::raw::RawValue;
use crate::{
    core::error::VMError,
    heap::HeapObject,
    types::{
        object::func::{Engine, Function},
        Value,
    },
    vm::Vm,
};

fn write(
    vm: &mut Vm,
    _self: Option<HeapRef>,
    params: Vec<Value>,
    debug: bool,
) -> Result<Value, VMError> {
    // get params
    let data_ref = params[0].clone();
    let data = match data_ref {
        Value::HeapRef(r) => {
            let heap_obj = vm.resolve_heap_ref(r);
            let request = match heap_obj {
                HeapObject::String(s) => s.to_string(),
                _ => {
                    return Err(error::throw(VMErrorType::TypeMismatch {
                        expected: "string".to_string(),
                        received: heap_obj.to_string(),
                    }));
                }
            };
            request
        }
        Value::RawValue(RawValue::Utf8(s)) => s.value,
        _ => {
            return Err(error::throw(VMErrorType::TypeMismatch {
                expected: "string".to_string(),
                received: "bound_access".to_string(),
            }));
        }
    };

    // resolve 'self'
    let _self = if let Some(_this) = _self {
        if let HeapObject::NativeStruct(NativeStruct::NetStream(ns)) =
            vm.resolve_heap_mut_ref(_this)
        {
            ns
        } else {
            unreachable!()
        }
    } else {
        unreachable!()
    };

    let write_result = _self.stream.write(data.as_bytes());
    if let Ok(bytes) = write_result {
        Ok(Value::RawValue(RawValue::U64(U64::new(bytes as u64))))
    } else {
        Err(error::throw(VMErrorType::Net(NetErrors::WriteError(
            _self.host.to_string(),
        ))))
    }
}

fn read(
    vm: &mut Vm,
    _self: Option<HeapRef>,
    params: Vec<Value>,
    debug: bool,
) -> Result<Value, VMError> {
    // resolve 'self'
    let _self = if let Some(_this) = _self {
        if let HeapObject::NativeStruct(NativeStruct::NetStream(ns)) =
            vm.resolve_heap_mut_ref(_this)
        {
            ns
        } else {
            unreachable!()
        }
    } else {
        unreachable!()
    };

    let mut buffer = [0; 4096];
    let read_result = _self.stream.read(&mut buffer);
    let bytes_count = if let Ok(bytes_count) = read_result {
        bytes_count
    } else {
        return Err(error::throw(VMErrorType::Net(NetErrors::ReadError(
            _self.host.to_string(),
        ))));
    };
    let read_obj = HeapObject::String(String::from_utf8_lossy(&buffer[..bytes_count]).to_string());
    Ok(Value::HeapRef(vm.heap.allocate(read_obj)))
}

pub fn connect(
    vm: &mut Vm,
    _self: Option<HeapRef>,
    params: Vec<Value>,
    debug: bool,
) -> Result<Value, VMError> {
    let host_ref = params[0].clone();
    let host = match host_ref {
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

    let stream = if let Ok(stream) = TcpStream::connect(host) {
        stream
    } else {
        return Err(error::throw(VMErrorType::Net(NetErrors::NetConnectError(
            format!("host {}", host),
        ))));
    };

    let mut shape = HashMap::new();
    let owned_host = host.clone();
    let host_ref = vm.heap.allocate(HeapObject::String(host.clone()));
    let write_ref = vm.heap.allocate(HeapObject::Function(Function::new(
        "write".to_string(),
        vec![],
        Engine::Native(write),
    )));
    let read_ref = vm.heap.allocate(HeapObject::Function(Function::new(
        "read".to_string(),
        vec![],
        Engine::Native(read),
    )));

    shape.insert("host".to_string(), Value::HeapRef(host_ref));
    shape.insert("write".to_string(), Value::HeapRef(write_ref));
    shape.insert("read".to_string(), Value::HeapRef(read_ref));

    let net_stream = NetStream::new(owned_host, stream, shape);
    let net_stream_ref = vm
        .heap
        .allocate(HeapObject::NativeStruct(NativeStruct::NetStream(
            net_stream,
        )));

    return Ok(Value::HeapRef(net_stream_ref));
}

pub fn connect_ref() -> HeapObject {
    HeapObject::Function(Function::new(
        "connect".to_string(),
        vec!["host".to_string()],
        Engine::Native(connect),
    ))
}
