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

    pub fn free(&mut self, handle: Handle) -> MemObject {
        let mem_obj = self.resolve(&handle);
        match mem_obj {
            // heap objects
            MemObject::String(_)
            | MemObject::Function(_)
            | MemObject::NativeStruct(_)
            | MemObject::StructDeclaration(_)
            | MemObject::StructLiteral(_)
            | MemObject::Vector(_) => {
                // free handle from table
                let heap_ref = self.free_handle(&handle).1.as_heap_pointer();
                // free heap
                let mem_obj = self.heap.free(heap_ref);
                if mem_obj.is_none() {
                    panic!("handle pointer does not exist in memory table")
                }
                mem_obj.unwrap()
            }
        }
    }

    pub fn resolve(&self, handle: &Handle) -> &MemObject {
        let real_pointer = self.table.get(&handle.pointer);
        if let Some(rp) = real_pointer {
            match rp {
                PointerType::HeapPointer(p) => match self.heap.get(p.clone()) {
                    Some(v) => v,
                    None => panic!("handle pointer does not exist in memory table"),
                },
            }
        } else {
            panic!("handle pointer does not exist in memory table")
        }
    }

    pub fn resolve_mut(&mut self, handle: &Handle) -> &mut MemObject {
        let real_pointer = self.table.get(&handle.pointer);
        if let Some(rp) = real_pointer {
            match rp {
                PointerType::HeapPointer(p) => match self.heap.get_mut(p.clone()) {
                    Some(v) => v,
                    None => panic!("handle pointer does not exist in memory table"),
                },
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

    fn free_handle(&mut self, handle: &Handle) -> (u32, PointerType) {
        let val = self.table.remove(&handle.pointer);
        if val.is_none() {
            panic!("unset pointer exception")
        }

        (handle.pointer, val.unwrap().clone())
    }
}

#[derive(Debug, Clone)]
pub struct Handle {
    pub pointer: u32,
}

impl Handle {
    pub fn new(handle_pointer: u32) -> Handle {
        Handle {
            pointer: handle_pointer,
        }
    }

    pub fn to_string(&self) -> String {
        self.pointer.to_string()
    }
}

#[derive(Clone)]
pub enum PointerType {
    HeapPointer(HeapRef),
}

impl PointerType {
    pub fn as_heap_pointer(&self) -> HeapRef {
        match self {
            PointerType::HeapPointer(v) => v.clone(),
            _ => panic!("invalid parse on PointerType as_heap_pointer method"),
        }
    }
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
