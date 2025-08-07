mod bytecode;
mod handlers;

use std::fs;

use crate::ast::export_statement::ExportStatement;
use crate::ast::return_statement::ReturnStatement;
use crate::ast::structs::StructTypeExpr;
use crate::ast::{lex, Module};
use crate::{
    ast::{
        import_statement::{ImportStatement, ModuleType},
        objects::ObjectLiteral,
        string_literal::StringLiteral,
    },
    core::error::{self, ErrorType},
};
use bytecode::get_bytecode;
use self_vm::utils::{
    to_bytes::{bytes_from_32, bytes_from_64, bytes_from_float},
    Number,
};

use crate::ast::{
    assignament_statement::{AssignamentNode, VarType},
    block::Block,
    function_declaration::FunctionDeclaration,
    group::Group,
    if_statement::IfStatement,
    module::ModuleAst,
    objects::ObjectType,
    structs::Struct,
    while_statement::WhileStatement,
    AstNodeType, Expression, Type,
};

pub fn gen_bytecode(modulename: String, code: String, args: &Vec<String>) -> Vec<u8> {
    let debug = args.contains(&"-d".to_string());
    let tokens = lex(code);
    if debug {
        println!("\n--- TOKEN ----------\n");
        println!("{:#?}", tokens);
    }
    let mut module = Module::new(modulename, tokens);
    let ast = module.parse();
    if debug {
        println!("\n--- AST ----------\n");
        println!("{:#?}", ast);
    }
    let mut compiler = Compiler::new(ast);
    compiler.gen_bytecode()
}

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
            let node_bytecode = Compiler::gen_node_bytecode(&self.ast.children[counter]);
            self.bytecode.extend_from_slice(&node_bytecode);
            counter += 1;
        }

        self.bytecode.clone()
    }

    fn gen_node_bytecode(node: &AstNodeType) -> Vec<u8> {
        match node {
            AstNodeType::AssignamentStatement(node) => {
                Compiler::compile_assignament_statement(node)
            }
            AstNodeType::FunctionDeclaration(node) => Compiler::compile_function_declaration(node),
            AstNodeType::IfStatement(node) => Compiler::compile_if_statement(node),
            AstNodeType::Expression(node) => Compiler::compile_expression(node, true),
            AstNodeType::WhileStatement(node) => Compiler::compile_while_statement(node),
            AstNodeType::ReturnStatement(node) => Compiler::compile_return_statement(node),
            AstNodeType::Struct(node) => Compiler::compile_struct_declaration(node),
            AstNodeType::ImportStatement(node) => Compiler::compile_import(node),
            AstNodeType::ExportStatement(node) => Compiler::compile_export(node),
            _ => {
                // panic!("unhandled node type")
                // here we should, in the near future throw an error
                vec![]
            }
        }
    }

    fn compile_assignament_statement(node: &AssignamentNode) -> Vec<u8> {
        let mut operation_bytecode = vec![];
        // load value
        operation_bytecode.extend_from_slice(&Compiler::compile_expression(&node.init, false));

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

    fn compile_function_declaration(node: &FunctionDeclaration) -> Vec<u8> {
        let mut bytecode = vec![];

        // load function args num/type/...
        let parameters: Vec<String> = node
            .parameters
            .children
            .iter()
            .map(|c| match c {
                Some(Expression::Identifier(x)) => x.name.clone(),
                Some(_) => panic!(
                    "bad param type on '{}' function declaration",
                    node.identifier.name
                ),
                None => panic!(
                    "empty parameter in '{}' function declaration",
                    node.identifier.name
                ),
            })
            .collect();
        let params_length = parameters.len();
        for param in parameters {
            let param_bytecode = Compiler::compile_expression(
                &Expression::StringLiteral(StringLiteral {
                    value: param.to_string(),
                    raw_value: param,
                    at: node.parameters.at,
                    line: node.parameters.line,
                }),
                false,
            );
            bytecode.extend_from_slice(&param_bytecode);
        }

        // op
        bytecode.push(get_bytecode("function_declaration".to_string()));

        // load function name
        bytecode.extend_from_slice(&Compiler::compile_raw_string(node.identifier.name.clone()));

        // // load function parameters_num
        bytecode.extend_from_slice(&Compiler::compile_offset(params_length as i32));

        // load body of the function
        let body_bytecode = Compiler::compile_block(&node.body);
        let body_bytecode_length = if body_bytecode.len() > i32::MAX as usize {
            panic!(
                "{} function declaration body is bigger than the limits",
                node.identifier.name
            );
        } else {
            body_bytecode.len() as i32
        };

        bytecode.extend_from_slice(&Compiler::compile_offset(body_bytecode_length));
        bytecode.extend_from_slice(&body_bytecode);

        bytecode
    }

    fn compile_struct_declaration(node: &Struct) -> Vec<u8> {
        let mut bytecode = vec![];
        // op
        bytecode.push(get_bytecode("struct_declaration".to_string()));

        // identifier raw string
        bytecode.extend_from_slice(&Compiler::compile_raw_string(node.identifier.name.clone()));

        // compile object fields number
        bytecode.extend_from_slice(&Compiler::compile_offset(node.fields.fields.len() as i32));

        // object type
        bytecode.extend_from_slice(&Compiler::compile_object_type(&node.fields));

        bytecode
    }

    fn compile_if_statement(node: &IfStatement) -> Vec<u8> {
        let mut bytecode = vec![];

        let condition_bytecode = &Compiler::compile_expression(&node.condition, false);
        let then_bytecode = Compiler::compile_block(&node.body);
        let else_bytecode = if let Some(else_node) = &node.else_node {
            Compiler::compile_block(&else_node.body)
        } else {
            vec![]
        };
        let offset_to_else = Compiler::compile_offset((then_bytecode.len() + 4 + 1) as i32);
        let offset_skip_else = Compiler::compile_offset((else_bytecode.len() + 1) as i32);

        bytecode.extend_from_slice(&condition_bytecode);
        bytecode.push(get_bytecode("jump_if_false".to_string()));
        bytecode.extend_from_slice(&offset_to_else);
        bytecode.extend_from_slice(&then_bytecode);
        bytecode.push(get_bytecode("jump".to_string()));
        bytecode.extend_from_slice(&offset_skip_else);
        bytecode.extend_from_slice(&else_bytecode);

        bytecode
    }

    fn compile_while_statement(node: &WhileStatement) -> Vec<u8> {
        // body offset and while offset are calculated based on
        // two euristics to handle the circular reference
        // "to calculate body offset you need while offset and
        // viceversa"
        // 4: offset bytecode size
        // 1: opcode size
        let mut bytecode = vec![];
        let condition_bytecode = Compiler::compile_expression(&node.condition, false);
        let body_bytecode = Compiler::compile_block(&node.body);
        let body_offset = Compiler::compile_offset((body_bytecode.len() + 4 + 1) as i32);
        let while_offset = Compiler::compile_offset(
            -((condition_bytecode.len() + body_offset.len() + 1 + body_bytecode.len() + 4) as i32),
        );

        bytecode.extend_from_slice(&condition_bytecode);
        bytecode.push(get_bytecode("jump_if_false".to_string()));
        bytecode.extend_from_slice(&body_offset);
        bytecode.extend_from_slice(&body_bytecode);
        bytecode.push(get_bytecode("jump".to_string()));
        bytecode.extend_from_slice(&while_offset);
        bytecode
    }

    fn compile_return_statement(node: &ReturnStatement) -> Vec<u8> {
        let mut bytecode = vec![];

        bytecode.extend_from_slice(&Compiler::compile_expression(&node.value, false));
        bytecode.push(get_bytecode("return".to_string()));

        bytecode
    }

    // drop value: if the value must not be persisted like module level declared string
    //             or function calling with no receiver of the return value, the value
    //             must be dropped
    fn compile_expression(node: &Expression, drop_value: bool) -> Vec<u8> {
        // all expressions push a load_const opcode
        // except of identifier which loads a load_var opcode
        match node {
            Expression::CallExpression(v) => {
                let call_expression_bytecode = match v.get_callee().as_str() {
                    "print" => handlers::print_as_bytecode(v),
                    "println" => handlers::print_as_bytecode(v), // both print types can be handled by the same function
                    "call" => handlers::call_as_bytecode(v),
                    "ai" => handlers::function_call_as_bytecode(v, drop_value),
                    _ => handlers::function_call_as_bytecode(v, drop_value),
                };

                call_expression_bytecode
            }
            Expression::StructLiteral(v) => {
                let mut bytecode = vec![];

                // first, load field values onto the stack
                let (fields_num, object_literal_bytecode) =
                    &Compiler::compile_object_literal(&v.fields);
                bytecode.extend_from_slice(&object_literal_bytecode);

                // compile struct
                bytecode.push(get_bytecode("load_const".to_string()));
                bytecode.push(get_bytecode("struct_literal".to_string()));

                // struct type compilation
                let struct_type = match &v.identifier {
                    StructTypeExpr::Identifier(i) => &Expression::StringLiteral(
                        StringLiteral::new(i.name.clone(), i.name.clone(), v.at, v.line),
                    ),
                    StructTypeExpr::MemberExpression(m) => {
                        &Expression::MemberExpression(m.as_ref().clone())
                    }
                };
                bytecode.extend_from_slice(&Compiler::compile_expression(struct_type, false));

                // compile object fields number
                bytecode.extend_from_slice(&Compiler::compile_offset(*fields_num as i32));

                bytecode
            }
            Expression::Vector(v) => {
                let mut bytecode = vec![];
                let elements_num = v.children.len();

                for child in &v.children {
                    bytecode.extend_from_slice(&Compiler::compile_expression(child, false));
                }

                // compile struct
                bytecode.push(get_bytecode("load_const".to_string()));
                bytecode.push(get_bytecode("vector".to_string()));
                bytecode.extend_from_slice(&Compiler::compile_offset(elements_num as i32));

                bytecode
            }
            Expression::Number(v) => {
                let mut bytecode = vec![];
                bytecode.push(get_bytecode("load_const".to_string()));

                // if v.value.is_sign_negative() {
                //     panic!("Cannot compile negative numbers on self");
                // }

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
                bytecode.extend_from_slice(&Compiler::compile_expression(&left_operand, false));
                bytecode.extend_from_slice(&Compiler::compile_expression(&right_operand, false));

                // operator
                match v.operator.as_str() {
                    "+" => bytecode.push(get_bytecode("add".to_string())),
                    "-" => bytecode.push(get_bytecode("substract".to_string())),
                    "*" => bytecode.push(get_bytecode("multiply".to_string())),
                    "/" => bytecode.push(get_bytecode("divide".to_string())),
                    ">" => bytecode.push(get_bytecode("greater_than".to_string())),
                    "<" => bytecode.push(get_bytecode("less_than".to_string())),
                    "==" => bytecode.push(get_bytecode("equals".to_string())),
                    "!=" => bytecode.push(get_bytecode("not_equals".to_string())),
                    _ => {}
                };

                bytecode
            }
            Expression::MemberExpression(v) => {
                let mut bytecode = vec![];
                let property = v.property.clone();
                let object = v.object.clone();

                // compile de property
                let string_literal = StringLiteral::new(
                    property.name.clone(),
                    property.name,
                    property.at,
                    property.line,
                );
                let property_bytecode =
                    Compiler::compile_expression(&Expression::StringLiteral(string_literal), false);

                // compile object (a potential nested object_expression)
                let object_bytecode = Compiler::compile_expression(&object, false);

                bytecode.extend_from_slice(&object_bytecode);
                bytecode.extend_from_slice(&property_bytecode);
                bytecode.push(get_bytecode("get_property".to_string()));
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

    fn compile_import(node: &ImportStatement) -> Vec<u8> {
        let mut bytecode = vec![];

        match node.module_type {
            ModuleType::Native => {
                // for the moment let's only enable
                // one deepth
                let module = node.module[0].clone();
                bytecode.extend_from_slice(&Compiler::compile_expression(
                    &Expression::StringLiteral(StringLiteral::new(
                        module.to_string(),
                        module,
                        node.at,
                        node.line,
                    )),
                    false,
                ));

                bytecode.push(get_bytecode("import".to_string()));
                // compile an offset to make compatible with the custom module
                // execution
                bytecode.extend_from_slice(&Compiler::compile_offset(0 as i32));
                bytecode
            }
            ModuleType::Custom => {
                // here we should handle circular dependency
                // on imports, to avoid an infinite loop of
                // compilation
                let module_name = node.module[0].clone();
                let path = format!("{}.ego", module_name);
                let code =
                    fs::read_to_string(&path).expect(&format!("Failed to read module '{}'", path));
                let mod_bytecode = gen_bytecode(module_name.to_string(), code, &vec![]);

                // push module_name to stack
                bytecode.extend_from_slice(&Compiler::compile_expression(
                    &Expression::StringLiteral(StringLiteral::new(
                        module_name.to_string(),
                        module_name,
                        node.at,
                        node.line,
                    )),
                    false,
                ));
                bytecode.push(get_bytecode("import".to_string()));
                bytecode.extend_from_slice(&Compiler::compile_offset(mod_bytecode.len() as i32));
                bytecode.extend_from_slice(&mod_bytecode);

                bytecode
            }
        }
    }

    fn compile_export(node: &ExportStatement) -> Vec<u8> {
        let mut bytecode = vec![];
        match &node.value {
            Expression::Identifier(n) => {
                let identifier = n.name.clone();
                bytecode.extend_from_slice(&Compiler::compile_expression(
                    &Expression::StringLiteral(StringLiteral {
                        value: identifier.clone(),
                        raw_value: identifier,
                        at: node.at,
                        line: node.line,
                    }),
                    false,
                ));
            }
            _ => {
                error::throw(
                    ErrorType::CompilationError,
                    "exported member must be an <identifier>",
                    Some(node.line),
                );
                std::process::exit(1);
            }
        }

        bytecode.push(get_bytecode("export".to_string()));
        bytecode
    }

    fn compile_block(node: &Block) -> Vec<u8> {
        let mut bytecode = vec![];
        for node in &node.children {
            let node_bytecode = Compiler::gen_node_bytecode(node);
            bytecode.extend_from_slice(&node_bytecode);
        }

        bytecode
    }

    fn compile_group(node: &Group) -> (usize, Vec<u8>) {
        let mut bytecode = vec![];
        for argument in &node.children {
            if let Some(arg) = argument {
                bytecode.extend_from_slice(&Compiler::compile_expression(&arg, false))
            } else {
                // push nothing to bytecode
            }
        }

        (node.children.len(), bytecode)
    }

    fn compile_object_type(node: &ObjectType) -> Vec<u8> {
        let mut bytecode = vec![];

        for field in &node.fields {
            bytecode.extend_from_slice(&Compiler::compile_raw_string(field.name.to_string()));
            if let Some(annotation) = field.annotation {
                // tyte it's a funny name for type_byte
                let tyte = match annotation {
                    Type::Bool => get_bytecode("bool".to_string()),
                    Type::String => get_bytecode("utf8".to_string()),
                    Type::Number => get_bytecode("f64".to_string()),
                    Type::Nothing => get_bytecode("nothing".to_string()),
                };
                bytecode.push(tyte);
            } else {
                // this code should be unreachable
                error::throw(
                    ErrorType::CompilationError,
                    "object type's identifier must be annotated",
                    Some(node.line),
                );
            }
        }

        bytecode
    }

    fn compile_object_literal(node: &ObjectLiteral) -> (usize, Vec<u8>) {
        let mut bytecode = vec![];

        for field in &node.fields {
            // load field_namde
            bytecode.push(get_bytecode("load_const".to_string()));
            bytecode.extend_from_slice(&Compiler::compile_raw_string(field.0.name.clone()));
            // load expression
            bytecode.extend_from_slice(&Compiler::compile_expression(&field.1, false));
        }

        (node.fields.len(), bytecode)
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

    fn compile_offset(v: i32) -> [u8; 4] {
        let mut bytecode = [0u8; 4];
        bytecode[0..4].copy_from_slice(&v.to_le_bytes());
        bytecode
    }
}
