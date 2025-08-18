mod members;
pub mod types;
pub mod utils;

use crate::{memory::MemObject, std::net::members::connect_ref};

pub fn generate_struct() -> (String, Vec<(String, MemObject)>) {
    let mut fields = vec![];

    fields.push(("connect".to_string(), connect_ref()));

    ("net".to_string(), fields)
}
