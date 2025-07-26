use std::collections::HashMap;
use std::net::TcpStream;

use crate::core::error::net_errors::NetErrors;
use crate::core::error::{self, VMErrorType};
use crate::std::net::types::NetStream;
use crate::types::object::native_struct::NativeStruct;
use crate::{
    core::error::VMError,
    heap::HeapObject,
    types::{
        object::func::{Engine, Function},
        Value,
    },
    vm::Vm,
};

pub fn connect(vm: &mut Vm, params: Vec<Value>, debug: bool) -> Result<Value, VMError> {
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
    let host_ref = vm.heap.allocate(HeapObject::String(host.clone()));
    shape.insert("host".to_string(), Value::HeapRef(host_ref));

    let net_stream = NetStream::new(stream, shape);
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
