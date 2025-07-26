mod ast;
mod commands;
mod compiler;
mod core;
mod wasm;

pub use compiler::gen_bytecode;
use wasm::run_ego;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn exec_ego_code(code: String, vm: bool) -> Vec<String> {
    run_ego(code, vm)
}
