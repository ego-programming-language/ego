mod members;

use crate::{
    heap::HeapObject,
    std::{
        fs::members::{read_file_obj, write_file_def, write_file_obj},
        NativeModuleDef,
    },
};

pub fn generate_struct() -> (String, Vec<(String, HeapObject)>) {
    let mut fields = vec![];

    fields.push(("read_file".to_string(), read_file_obj()));
    fields.push(("write_file".to_string(), write_file_obj()));

    ("fs".to_string(), fields)
}

pub fn generate_mod_def() -> NativeModuleDef {
    let members = vec![write_file_def()];

    NativeModuleDef {
        module: "fs".to_string(),
        members,
    }
}
