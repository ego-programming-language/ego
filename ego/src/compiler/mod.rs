mod bytecode;
mod handlers;

use bytecode::get_bytecode;
use self_vm::utils::{
    to_bytes::{bytes_from_32, bytes_from_64, bytes_from_float, bytes_from_utf8},
    Number,
};

use crate::ast::{
    assignament_statement::{AssignamentNode, VarType},
    group::Group,
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
                AstNodeType::AssignamentStatement(node) => {
                    Compiler::compile_assignament_statement(node)
                }
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

    fn compile_assignament_statement(node: &AssignamentNode) -> Vec<u8> {
        let mut operation_bytecode = vec![];
        // load value
        operation_bytecode.extend_from_slice(&Compiler::compile_expression(&node.init));

        // op
        operation_bytecode.push(get_bytecode("store_var".to_string()));

        // var_type
        operation_bytecode.push(match node.var_type {
            VarType::Const => get_bytecode("inmut".to_string()),
            _ => get_bytecode("mut".to_string()),
        });

        // identifier raw string
        operation_bytecode
            .extend_from_slice(&Compiler::compile_raw_string(node.identifier.name.clone()));

        operation_bytecode
    }

    fn compile_expression(node: &Expression) -> Vec<u8> {
        // all expressions push a load_const opcode
        // except of identifier which loads a load_var opcode
        match node {
            Expression::CallExpression(v) => {
                let call_expression_bytecode = match v.identifier.name.as_str() {
                    "print" => handlers::print_as_bytecode(v),
                    "println" => handlers::print_as_bytecode(v), // both print types can be handled by the same function
                    "call" => handlers::call_as_bytecode(v),
                    _ => {
                        // todo: handle custom defined callable members
                        vec![]
                    }
                };

                call_expression_bytecode
            }
            Expression::Number(v) => {
                let mut bytecode = vec![];
                bytecode.push(get_bytecode("load_const".to_string()));

                if v.value.is_sign_negative() {
                    panic!("Cannot compile negative numbers on self");
                }

                let (num_bytecode, num_type_bytecode) = if v.value.fract() != 0.0 {
                    (
                        bytes_from_float(Number::F64(v.value)).to_vec(),
                        get_bytecode("f64".to_string()),
                    )
                } else if v.value >= i32::MIN as f64 && v.value <= i32::MAX as f64 {
                    (
                        bytes_from_32(Number::I32(v.value as i32)).to_vec(),
                        get_bytecode("i32".to_string()),
                    )
                } else if v.value >= i64::MIN as f64 && v.value <= i64::MAX as f64 {
                    (
                        bytes_from_64(Number::I64(v.value as i64)).to_vec(),
                        get_bytecode("i64".to_string()),
                    )
                } else {
                    panic!("Unsupported number type or out of range");
                };

                // type
                bytecode.push(num_type_bytecode);

                // value
                bytecode.extend_from_slice(&num_bytecode);
                bytecode
            }
            Expression::StringLiteral(v) => {
                let mut bytecode = vec![];
                bytecode.push(get_bytecode("load_const".to_string()));

                // todo: handle larger string
                let string_bytes = v.raw_value.as_bytes();
                let string_length = string_bytes.len() as u32;

                bytecode.push(get_bytecode("utf8".to_string()));
                bytecode.push(get_bytecode("u32".to_string()));
                bytecode.extend_from_slice(&string_length.to_le_bytes());
                bytecode.extend_from_slice(string_bytes);
                bytecode
            }
            Expression::Bool(v) => {
                let mut bytecode = vec![];
                bytecode.push(get_bytecode("load_const".to_string()));
                bytecode.push(get_bytecode("bool".to_string()));
                if v.value {
                    bytecode.push(0x01);
                } else {
                    bytecode.push(0x00);
                };

                bytecode
            }
            Expression::Identifier(v) => {
                let mut bytecode = vec![];
                bytecode.push(get_bytecode("load_var".to_string()));

                let identifier_bytecode = Compiler::compile_raw_string(v.name.clone());
                bytecode.extend_from_slice(&identifier_bytecode);
                bytecode
            }
            Expression::BinaryExpression(v) => {
                let mut bytecode = vec![];

                // operands
                let left_operand = *v.left.clone();
                let right_operand = *v.right.clone();
                bytecode.extend_from_slice(&Compiler::compile_expression(&left_operand));
                bytecode.extend_from_slice(&Compiler::compile_expression(&right_operand));

                // operator
                match v.operator.as_str() {
                    "+" => bytecode.push(get_bytecode("add".to_string())),
                    "-" => bytecode.push(get_bytecode("substract".to_string())),
                    "*" => bytecode.push(get_bytecode("multiply".to_string())),
                    "/" => bytecode.push(get_bytecode("divide".to_string())),
                    _ => {}
                };

                bytecode
            }
            Expression::Nothing(_) => {
                let mut bytecode = vec![];
                bytecode.push(get_bytecode("load_const".to_string()));
                bytecode.push(get_bytecode("nothing".to_string()));

                bytecode
            }
            _ => {
                panic!("unhandled expression type")
            }
        }
    }

    fn compile_group(node: &Group) -> (usize, Vec<u8>) {
        let mut bytecode = vec![];
        for argument in &node.children {
            if let Some(arg) = argument {
                bytecode.extend_from_slice(&Compiler::compile_expression(&arg))
            } else {
                // push nothing to bytecode
            }
        }

        (node.children.len(), bytecode)
    }

    fn compile_raw_string(v: String) -> Vec<u8> {
        let mut bytecode = vec![];

        // todo: handle larger string
        let string_bytes = v.as_bytes();
        let string_length = string_bytes.len() as u32;

        bytecode.push(get_bytecode("utf8".to_string()));
        bytecode.push(get_bytecode("u32".to_string()));
        bytecode.extend_from_slice(&string_length.to_le_bytes());
        bytecode.extend_from_slice(string_bytes);
        bytecode
    }
}
