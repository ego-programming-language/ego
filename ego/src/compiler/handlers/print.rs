use std::collections::btree_map;

use crate::{
    ast::{call_expression::CallExpression, Expression},
    compiler::bytecode::get_bytecode,
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
            match arg {
                Expression::Number(v) => {
                    if v.value.is_sign_negative() {
                        panic!("Cannot compile negative numbers on self");
                    }

                    let (num_bytecode, num_type_bytecode) = match v.value {
                        value if value >= i32::MIN as f64 && value <= i32::MAX as f64 => (
                            bytes_from_32(Number::I32(value as i32)).to_vec(),
                            get_bytecode("i32".to_string()),
                        ),
                        value if value >= i64::MIN as f64 && value <= i64::MAX as f64 => (
                            bytes_from_64(Number::I64(value as i64)).to_vec(),
                            get_bytecode("i64".to_string()),
                        ),
                        _ => panic!("Unsupported number type or out of range"),
                    };

                    // type
                    bytecode.push(num_type_bytecode);

                    // value
                    bytecode.extend_from_slice(&num_bytecode);
                }
                Expression::StringLiteral(v) => {
                    let string_bytes = v.value.as_bytes();
                    // todo: handle larger string
                    let string_length = string_bytes.len() as u32;

                    bytecode.push(get_bytecode("utf8".to_string()));
                    bytecode.push(get_bytecode("u32".to_string()));
                    bytecode.extend_from_slice(&string_length.to_le_bytes());
                    bytecode.extend_from_slice(string_bytes);
                }
                Expression::Bool(v) => {
                    bytecode.push(get_bytecode("bool".to_string()));
                    if v.value {
                        bytecode.push(0x01);
                    } else {
                        bytecode.push(0x00);
                    };
                }
                _ => {
                    println!("- Argument skipped")
                }
            }
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
