use crate::{
    heap::HeapObject,
    types::object::func::{Engine, Function},
    vm::Vm,
};

pub mod ai;
pub mod fs;
pub mod heap_utils;
pub mod net;
pub mod selfmod;
mod utils;

pub enum NativeModule {
    AI,
    SelfMod,
    Fs,
    Net,
}

pub struct NativeMember {
    pub name: String,
    pub description: String,
    pub params: Option<Vec<String>>,
}

impl NativeMember {
    pub fn to_string(&self) -> String {
        format!(
            "{}: {} [{}]",
            self.name,
            self.description,
            if let Some(params) = &self.params {
                params.join(", ")
            } else {
                "".to_string()
            }
        )
    }
}

pub struct NativeModuleDef {
    module: String,
    members: Vec<NativeMember>,
}

impl NativeModuleDef {
    pub fn to_string(&self) -> String {
        let formatted_members: Vec<String> = self
            .members
            .iter()
            .map(|member| member.to_string())
            .collect();
        format!(
            "
module: {}
members: 

{}",
            self.module,
            formatted_members.join("\n\n")
        )
    }
}

pub fn get_native_module_type(module_name: &str) -> Option<NativeModule> {
    match module_name {
        "ai" => Some(NativeModule::AI),
        "self" => Some(NativeModule::SelfMod),
        "fs" => Some(NativeModule::Fs),
        "net" => Some(NativeModule::Net),
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
        NativeModule::Net => net::generate_struct(),
    }
}

pub fn gen_native_modules_defs() -> Vec<NativeModuleDef> {
    return vec![fs::generate_mod_def()];
}
