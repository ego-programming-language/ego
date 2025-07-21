use crate::heap::HeapObject;

pub mod ai;
pub mod fs;
pub mod selfmod;

pub enum NativeModule {
    AI,
    SelfMod,
}

pub fn get_native_module_type(module_name: &str) -> Option<NativeModule> {
    match module_name {
        "ai" => Some(NativeModule::AI),
        "self" => Some(NativeModule::SelfMod),
        _ => None,
    }
}
pub fn generate_native_module(
    module: NativeModule,
) -> (std::string::String, Vec<(String, HeapObject)>) {
    match module {
        NativeModule::AI => ai::generate_struct(),
        NativeModule::SelfMod => selfmod::generate_struct(),
    }
}
