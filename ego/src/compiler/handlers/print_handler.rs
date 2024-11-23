use std::collections::btree_map;

use crate::{
    ast::{call_expression::CallExpression, Expression},
    compiler::{bytecode::get_bytecode, Compiler},
};

use self_vm::utils::{
    to_bytes::{bytes_from_32, bytes_from_64, bytes_from_utf8},
    Number,
};

pub fn print_as_bytecode(node: &CallExpression) -> Vec<u8> {
    let mut bytecode = vec![];

    // load arguments
    // todo: handle different var types, not only conts
    let load_const_bytecode = get_bytecode("load_const".to_string());
    for argument in &node.arguments.children {
        if let Some(arg) = argument {
            // todo: handle different var types, not only conts
            // refactor: create a function to compile expressions to bytecode
            bytecode.push(load_const_bytecode);
            bytecode.extend_from_slice(&Compiler::compile_expression(&arg))
        } else {
            // push nothing to bytecode
        }
    }

    // print instruction bytecode
    let print_bytecode = get_bytecode(node.identifier.name.to_string());
    bytecode.push(print_bytecode);

    // number of args bytecode
    let num_of_args = node.arguments.children.len() as u32;
    let num_of_args = bytes_from_32(Number::U32(num_of_args));
    bytecode.extend_from_slice(&num_of_args);

    bytecode
}
