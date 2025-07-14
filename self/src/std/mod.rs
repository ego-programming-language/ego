use crate::{heap::HeapObject, types::object::structs::StructLiteral, vm::Vm};

pub mod ai;
pub mod fs;

pub enum NativeModule {
    AI,
}

pub fn get_native_module_type(module_name: &str) -> Option<NativeModule> {
    match module_name {
        "ai" => Some(NativeModule::AI),
        _ => None,
    }
}
pub fn generate_native_module(
    module: NativeModule,
) -> (std::string::String, Vec<(String, HeapObject)>) {
    match module {
        NativeModule::AI => ai::generate_struct(),
    }
}
