mod ast;
mod commands;
mod compiler;
mod core;
mod runtime;
mod wasm;

use ast::{lex, Module};
use compiler::Compiler;
use wasm::run_ego;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn exec_ego_code(code: String, vm: bool) -> Vec<String> {
    run_ego(code, vm)
}

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
