mod bytecode;
mod handlers;

use crate::ast::{module::ModuleAst, AstNodeType, Expression};

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
            match &self.ast.children[counter] {
                AstNodeType::Expression(node) => match node {
                    Expression::CallExpression(v) => {
                        let call_expression_bytecode = match v.identifier.name.as_str() {
                            "print" => handlers::print_as_bytecode(v),
                            _ => {
                                // todo: handle custom defined callable members
                                vec![]
                            }
                        };

                        self.bytecode.extend_from_slice(&call_expression_bytecode);
                    }
                    _ => {}
                },
                _ => {}
            }
            counter += 1;
        }

        self.bytecode.clone()
    }
}
