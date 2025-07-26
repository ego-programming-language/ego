use std::{clone, collections::HashMap};

use crate::std::net::types::NetStream;
use crate::types::object::native_struct::NativeStruct;
use crate::types::object::{
    func::Function,
    structs::{StructDeclaration, StructLiteral},
};

#[derive(Debug)]
pub struct Heap {
    memory: HashMap<usize, HeapObject>,
    next_address: usize,
}

#[derive(Debug)]
pub enum HeapObject {
    String(String),
    Function(Function),
    StructDeclaration(StructDeclaration),
    StructLiteral(StructLiteral),
    NativeStruct(NativeStruct),
}

#[derive(Debug, Clone)]
pub struct HeapRef {
    address: usize,
}

impl Heap {
    pub fn new() -> Self {
        Heap {
            memory: HashMap::new(),
            next_address: 0,
        }
    }

    pub fn allocate(&mut self, obj: HeapObject) -> HeapRef {
        let address = self.next_address;
        self.next_address += 1;
        self.memory.insert(address, obj);
        HeapRef::new(address)
    }

    pub fn get(&self, heap_ref: HeapRef) -> Option<&HeapObject> {
        self.memory.get(&heap_ref.address)
    }

    pub fn free(&mut self, heap_ref: HeapRef) -> Option<HeapObject> {
        self.memory.remove(&heap_ref.address)
    }

    // we do not free memory for the moment xD
}

impl HeapRef {
    pub fn new(address: usize) -> Self {
        HeapRef { address }
    }

    pub fn get_address(&self) -> usize {
        self.address
    }
}

impl HeapObject {
    pub fn to_string(&self) -> String {
        match self {
            HeapObject::String(x) => x.to_string(),
            HeapObject::Function(x) => x.to_string(),
            HeapObject::StructDeclaration(x) => x.to_string(),
            HeapObject::StructLiteral(x) => x.identifier.to_string(),
            HeapObject::NativeStruct(x) => x.to_string(),
        }
    }
}
