use crate::{
    ast::{call_expression::CallExpression, Expression},
    compiler::{bytecode::get_bytecode, Compiler},
};

use self_vm::utils::{
    to_bytes::{bytes_from_32, bytes_from_64, bytes_from_utf8},
    Number,
};

pub fn call_as_bytecode(node: &CallExpression) -> Vec<u8> {
    let mut bytecode = vec![];

    // load arguments
    for argument in &node.arguments.children {
        if let Some(arg) = argument {
            bytecode.extend_from_slice(&Compiler::compile_expression(&arg))
        } else {
            // push nothing to bytecode
        }
    }

    // call instruction bytecode
    let call_bytecode = get_bytecode(node.identifier.name.to_string());
    bytecode.push(call_bytecode);

    // number of args bytecode
    let num_of_args = node.arguments.children.len() as u32;
    let num_of_args = bytes_from_32(Number::U32(num_of_args));
    bytecode.extend_from_slice(&num_of_args);

    bytecode
}
