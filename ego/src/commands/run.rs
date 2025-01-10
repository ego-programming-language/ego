use std::fs;
use std::fs::File;
use std::io::Write;

use crate::ast::lex;
use crate::ast::Module;
use crate::compiler::Compiler;
use crate::core::error;
use crate::core::error::ErrorType;
use crate::runtime::Interpreter;

pub struct Run {
    args: Vec<String>,
}

impl Run {
    pub fn new(args: Vec<String>) -> Run {
        Run { args }
    }
    pub fn debug(&self) -> bool {
        self.args.contains(&"-d".to_string())
    }
    pub fn exec(&self) {
        let module_name = if self.args.len() > 0 {
            self.args[0].clone()
        } else {
            "main.ego".to_string() // default lookup on a ego project
        };

        let file_content = fs::read_to_string(&module_name).unwrap_or_else(|_| {
            error::throw(
                ErrorType::FatalError,
                format!("Cannot read {}\n", module_name).as_str(),
                None,
            );
            std::process::exit(1); // to avoid types error
        });

        let tokens = lex(file_content);
        if self.debug() {
            println!("\nLexer tokens: \n-------------");
            for (i, token) in tokens.iter().enumerate() {
                println!("{i}. {token}");
            }
        }

        let mut module = Module::new(module_name, tokens);
        let ast = module.parse();
        if self.debug() {
            println!("\nAst nodes: \n---------------\n{:#?}", ast);
        }

        if self.args.contains(&"--bytes".to_string()) {
            let mut compiler = Compiler::new(ast);
            let bytecode = compiler.gen_bytecode();
            let bytecode_string: String = bytecode
                .iter()
                .map(|byte| format!("{:02X}", byte))
                .collect::<Vec<String>>()
                .join(" ");

            let mut file = match File::create("bytecode.bin") {
                Ok(file) => file,
                Err(_) => {
                    error::throw(ErrorType::SyntaxError, "Cannot write file", None);
                    unreachable!()
                }
            };
            match file.write_all(&bytecode) {
                Ok(_) => println!("{}", bytecode_string),
                Err(_) => {
                    error::throw(ErrorType::SyntaxError, "Cannot write file", None);
                    unreachable!()
                }
            };
        } else if self.args.contains(&"-vm".to_string()) {
            let mut compiler = Compiler::new(ast);
            let bytecode = compiler.gen_bytecode();
            let mut vm = self_vm::new(bytecode);
            vm.run(&self.args);
        } else {
            let mut interpreter = Interpreter::new(ast.clone());
            interpreter.exec(self.debug());
        }
    }
}
