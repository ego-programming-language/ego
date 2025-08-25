use std::collections::HashMap;

use crate::{
    heap::{Heap, HeapRef},
    types::object::{
        func::Function,
        native_struct::NativeStruct,
        structs::{StructDeclaration, StructLiteral},
        vector::Vector,
    },
    vm::Vm,
};

pub struct MemoryManager {
    heap: Heap,
    table: HashMap<u32, PointerType>,
    next_pointer: u32,
}

impl MemoryManager {
    pub fn new() -> MemoryManager {
        MemoryManager {
            heap: Heap::new(),
            table: HashMap::new(),
            next_pointer: 0,
        }
    }

    pub fn alloc(&mut self, obj: MemObject) -> Handle {
        match obj {
            MemObject::String(_)
            | MemObject::Function(_)
            | MemObject::NativeStruct(_)
            | MemObject::StructDeclaration(_)
            | MemObject::StructLiteral(_)
            | MemObject::Vector(_) => {
                let heap_ref = self.heap.allocate(obj);
                self.gen_handle(PointerType::HeapPointer(heap_ref))
            }
        }
    }

    pub fn resolve(&self, handle: Handle) -> Option<&MemObject> {
        let real_pointer = self.table.get(&handle.pointer);
        if let Some(rp) = real_pointer {
            match rp {
                PointerType::HeapPointer(p) => self.heap.get(p.clone()),
            }
        } else {
            panic!("handle pointer does not exist in memory table")
        }
    }

    fn gen_handle(&mut self, pointer: PointerType) -> Handle {
        let generated_pointer = self.next_pointer;
        self.next_pointer += 1;
        let handle = Handle::new(generated_pointer);
        self.table.insert(generated_pointer, pointer);
        handle
    }
}

pub struct Handle {
    pub pointer: u32,
}

impl Handle {
    pub fn new(handle_pointer: u32) -> Handle {
        Handle {
            pointer: handle_pointer,
        }
    }
}

pub enum PointerType {
    HeapPointer(HeapRef),
}

#[derive(Debug)]
pub enum MemObject {
    String(String),
    Function(Function),
    StructDeclaration(StructDeclaration),
    StructLiteral(StructLiteral),
    NativeStruct(NativeStruct),
    Vector(Vector),
}

impl MemObject {
    pub fn to_string(&self, vm: &Vm) -> String {
        match self {
            MemObject::String(x) => x.to_string(),
            MemObject::Function(x) => x.to_string(),
            MemObject::StructDeclaration(x) => x.to_string(),
            MemObject::StructLiteral(x) => x.struct_type.to_string(),
            MemObject::NativeStruct(x) => x.to_string(),
            MemObject::Vector(x) => x.to_string(vm),
        }
    }
}
