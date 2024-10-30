use crate::{
    ast::{call_expression::CallExpression, Expression},
    compiler::bytecode::get_bytecode,
};

use self_vm::utils::{
    to_bytes::{bytes_from_32, bytes_from_64},
    Number,
};

pub fn print_as_bytecode(node: &CallExpression) -> Vec<u8> {
    let mut bytecode = vec![];

    // load arguments
    // todo: handle different var types, not only conts
    let load_const_bytecode = get_bytecode("load_const".to_string());
    for argument in &node.arguments.children {
        if let Some(arg) = argument {
            match arg {
                Expression::Number(v) => {
                    bytecode.push(load_const_bytecode);
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

                    // Push type bytecode
                    bytecode.push(num_type_bytecode);

                    // value bytecode
                    bytecode.extend_from_slice(&num_bytecode);
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
