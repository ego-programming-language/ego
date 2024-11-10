mod bytecode;
mod handlers;

use bytecode::get_bytecode;
use self_vm::utils::{
    to_bytes::{bytes_from_32, bytes_from_64, bytes_from_utf8},
    Number,
};

use crate::ast::{
    assignament_statement::{AssignamentNode, VarType},
    identifier,
    module::ModuleAst,
    AstNodeType, Expression,
};

pub struct Compiler {
    ast: ModuleAst,
    bytecode: Vec<u8>,
}

impl Compiler {
    pub fn new(ast: ModuleAst) -> Compiler {
        Compiler {
            ast,
            bytecode: vec![],
        }
    }

    pub fn gen_bytecode(&mut self) -> Vec<u8> {
        let mut counter = 0;
        while counter < self.ast.children.len() {
            let node_bytecode = match &self.ast.children[counter] {
                // [op][var_type][identifier][value]
                AstNodeType::Expression(node) => Compiler::compile_expression(node),
                _ => {
                    // panic!("unhandled node type")
                    vec![]
                }
            };
            self.bytecode.extend_from_slice(&node_bytecode);
            counter += 1;
        }

        self.bytecode.clone()
    }

    fn compile_expression(node: &Expression) -> Vec<u8> {
        match node {
            Expression::CallExpression(v) => {
                let call_expression_bytecode = match v.identifier.name.as_str() {
                    "print" => handlers::print_as_bytecode(v),
                    _ => {
                        // todo: handle custom defined callable members
                        vec![]
                    }
                };

                call_expression_bytecode
            }
            Expression::Number(v) => {
                let mut bytecode = vec![];
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
                bytecode
            }
            Expression::StringLiteral(v) => {
                let mut bytecode = vec![];
                let string_bytes = v.raw_value.as_bytes();
                // todo: handle larger string
                let string_length = string_bytes.len() as u32;

                bytecode.push(get_bytecode("utf8".to_string()));
                bytecode.push(get_bytecode("u32".to_string()));
                bytecode.extend_from_slice(&string_length.to_le_bytes());
                bytecode.extend_from_slice(string_bytes);
                bytecode
            }
            Expression::Bool(v) => {
                let mut bytecode = vec![];
                bytecode.push(get_bytecode("bool".to_string()));
                if v.value {
                    bytecode.push(0x01);
                } else {
                    bytecode.push(0x00);
                };

                bytecode
            }
            _ => {
                panic!("unhandled expression type")
            }
        }
    }
}
