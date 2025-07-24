use crate::{
    heap::HeapObject,
    types::object::func::{Engine, Function},
    vm::Vm,
};

pub mod ai;
pub mod fs;
pub mod heap_utils;
pub mod selfmod;

pub enum NativeModule {
    AI,
    SelfMod,
    Fs,
}

pub fn get_native_module_type(module_name: &str) -> Option<NativeModule> {
    match module_name {
        "ai" => Some(NativeModule::AI),
        "self" => Some(NativeModule::SelfMod),
        "fs" => Some(NativeModule::Fs),
        _ => None,
    }
}
pub fn generate_native_module(
    module: NativeModule,
) -> (std::string::String, Vec<(String, HeapObject)>) {
    match module {
        NativeModule::AI => ai::generate_struct(),
        NativeModule::SelfMod => selfmod::generate_struct(),
        NativeModule::Fs => fs::generate_struct(),
    }
}
