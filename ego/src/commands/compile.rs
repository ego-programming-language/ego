use std::fs;
use std::fs::File;
use std::io::Write;

use crate::ast::lex;
use crate::ast::Module;
use crate::compiler::Compiler;
use crate::core::error;
use crate::core::error::ErrorType;

pub struct Compile {
    args: Vec<String>,
}

impl Compile {
    pub fn new(args: Vec<String>) -> Compile {
        Compile { args }
    }
    pub fn debug(&self) -> bool {
        self.args.contains(&"-d".to_string())
    }
    pub fn exec(&self) {
        let (module_name, out_name) = if self.args.len() > 1 {
            (self.args[0].clone(), self.args[1].clone())
        } else {
            ("main.ego".to_string(), "main.b".to_string()) // default lookup on a ego project
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

        let mut compiler = Compiler::new(ast);
        let bytecode = compiler.gen_bytecode();

        let mut file = match File::create(&out_name) {
            Ok(file) => file,
            Err(_) => {
                error::throw(ErrorType::SyntaxError, "Cannot write file", None);
                unreachable!()
            }
        };
        match file.write_all(&bytecode) {
            Ok(_) => {}
            Err(_) => {
                error::throw(ErrorType::SyntaxError, "Cannot write file", None);
                unreachable!()
            }
        };
    }
}
