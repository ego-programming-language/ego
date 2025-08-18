use crate::{
    heap::Heap,
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

impl MemoryManager {
    pub fn new() -> MemoryManager {
        MemoryManager { heap: Heap::new() }
    }
}
