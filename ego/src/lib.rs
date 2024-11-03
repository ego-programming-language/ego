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
    let tokens = lex(code);
    let mut module = Module::new(modulename, tokens);
    let ast = module.parse();
    let debug = args.contains(&"-d".to_string());
    if debug {
        println!("\n--- AST ----------\n");
        println!("{:#?}", ast);
    }
    Compiler::gen_bytecode(ast)
}
