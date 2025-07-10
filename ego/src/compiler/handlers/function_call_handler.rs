use std::collections::btree_map;

use crate::{
    ast::{call_expression::CallExpression, Expression},
    compiler::{self, bytecode::get_bytecode, Compiler},
};

use self_vm::utils::{
    to_bytes::{bytes_from_32, bytes_from_64, bytes_from_utf8},
    Number,
};

pub fn function_call_as_bytecode(node: &CallExpression) -> Vec<u8> {
    let mut bytecode = vec![];

    // load arguments
    let (args_len, args) = compiler::Compiler::compile_group(&node.arguments);
    bytecode.extend_from_slice(&args);

    // print instruction bytecode
    let opcode_bytecode = get_bytecode("call".to_string());
    bytecode.push(opcode_bytecode);

    // number of args bytecode
    let num_of_args = args_len as u32;
    let num_of_args = bytes_from_32(Number::U32(num_of_args));
    bytecode.extend_from_slice(&num_of_args);

    // identifier
    let identifier_bytecode = Compiler::compile_raw_string(node.get_callee());
    bytecode.extend_from_slice(&identifier_bytecode);

    bytecode
}
