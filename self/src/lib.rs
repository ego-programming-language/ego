mod instructions;
mod symbol_table;
mod translator;
mod types;

pub mod utils;
pub mod vm;

pub fn new(bytecode: Vec<u8>) -> vm::Vm {
    vm::Vm::new(bytecode)
}
