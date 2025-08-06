mod members;
use crate::{
    heap::HeapObject,
    std::env::members::{get_obj, set_obj},
};

pub fn generate_struct() -> (String, Vec<(String, HeapObject)>) {
    let mut fields = vec![];

    fields.push(("set".to_string(), set_obj()));
    fields.push(("get".to_string(), get_obj()));

    ("env".to_string(), fields)
}
