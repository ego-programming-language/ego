mod core;
mod heap;
mod instructions;
mod opcodes;
mod stack;
mod translator;
mod types;

pub mod utils;
pub mod vm;
pub use opcodes::get_codes_map;

pub fn new(bytecode: Vec<u8>) -> vm::Vm {
    vm::Vm::new(bytecode)
}
