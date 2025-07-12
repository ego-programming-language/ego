use std::collections::btree_map;

use crate::{
    ast::{
        call_expression::CallExpression, identifier::Identifier, string_literal::StringLiteral,
        Expression,
    },
    compiler::{self, bytecode::get_bytecode, Compiler},
};

use self_vm::utils::{
    to_bytes::{bytes_from_32, bytes_from_64, bytes_from_utf8},
    Number,
};

pub fn function_call_as_bytecode(node: &CallExpression) -> Vec<u8> {
    let mut bytecode = vec![];

    // callee
    let identifier_bytecode = match node.callee.as_ref() {
        Expression::MemberExpression(x) => {
            Compiler::compile_expression(&Expression::MemberExpression(x.clone()))
        }
        Expression::Identifier(x) => Compiler::compile_expression(&Expression::StringLiteral(
            StringLiteral::new(node.get_callee(), node.get_callee(), 0, 0),
        )),
        _ => {
            // TODO: use self-vm errors system
            panic!("compilation error: invalid callee for a function call")
        }
    };
    bytecode.extend_from_slice(&identifier_bytecode);

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

    bytecode
}
