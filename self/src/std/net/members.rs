use std::collections::HashMap;
use std::io::{Read, Write};
use std::net::TcpStream;

use crate::core::error::net_errors::NetErrors;
use crate::core::error::{self, VMErrorType};
use crate::heap::HeapRef;
use crate::std::net::types::{NetStream, StreamKind};
use crate::std::net::utils::tls;
use crate::types::object::native_struct::NativeStruct;
use crate::types::raw::u64::U64;
use crate::types::raw::RawValue;
use crate::{
    core::error::VMError,
    memory::MemObject,
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
    let data = params[0].as_string_obj(vm)?;
    // resolve 'self'
    let _self = if let Some(_this) = _self {
        if let MemObject::NativeStruct(NativeStruct::NetStream(ns)) = vm.resolve_heap_mut_ref(_this)
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
        if let MemObject::NativeStruct(NativeStruct::NetStream(ns)) = vm.resolve_heap_mut_ref(_this)
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
    let read_obj = MemObject::String(String::from_utf8_lossy(&buffer[..bytes_count]).to_string());
    Ok(Value::HeapRef(vm.heap.allocate(read_obj)))
}

pub fn connect(
    vm: &mut Vm,
    _self: Option<HeapRef>,
    params: Vec<Value>,
    debug: bool,
) -> Result<Value, VMError> {
    let host = params[0].as_string_obj(vm)?;
    let use_tls = if let Some(second) = params.get(1) {
        second.as_bool(vm)?
    } else {
        false // default if not passed
    };

    let stream = if use_tls {
        let tls_stream = tls(&host);
        if let Ok(_stream) = tls_stream {
            StreamKind::Tls(_stream)
        } else {
            return Err(error::throw(VMErrorType::Net(NetErrors::NetConnectError(
                format!("host {}", host),
            ))));
        }
    } else {
        if let Ok(stream) = TcpStream::connect(host.clone()) {
            StreamKind::Plain(stream)
        } else {
            return Err(error::throw(VMErrorType::Net(NetErrors::NetConnectError(
                format!("host {}", host),
            ))));
        }
    };

    let mut shape = HashMap::new();
    let owned_host = host.clone();
    let host_ref = vm.heap.allocate(MemObject::String(host.clone()));
    let write_ref = vm.heap.allocate(MemObject::Function(Function::new(
        "write".to_string(),
        vec![],
        Engine::Native(write),
    )));
    let read_ref = vm.heap.allocate(MemObject::Function(Function::new(
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
        .allocate(MemObject::NativeStruct(NativeStruct::NetStream(net_stream)));

    return Ok(Value::HeapRef(net_stream_ref));
}

pub fn connect_ref() -> MemObject {
    MemObject::Function(Function::new(
        "connect".to_string(),
        vec!["host".to_string()],
        Engine::Native(connect),
    ))
}
