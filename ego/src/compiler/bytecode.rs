use std::collections::HashMap;

use self_vm::get_codes_map;

use crate::core::error::{self, ErrorType};

pub struct Bytecode {
    table: HashMap<String, u8>,
}

impl Bytecode {
    pub fn get_handler() -> Bytecode {
        Bytecode {
            table: get_codes_map(),
        }
    }

    pub fn get_bytecode_representation(&mut self, key: String) -> Option<u8> {
        self.table.get(&key).copied()
    }
}

pub fn get_bytecode(item: String) -> u8 {
    let mut bytecode_handler = Bytecode::get_handler();

    let error_msg = format!("instruction not recognized: {}", item);
    if let Some(bytecode) = bytecode_handler.get_bytecode_representation(item) {
        bytecode
    } else {
        error::throw(ErrorType::CompilationError, &error_msg, None);
        std::process::exit(1)
    }
}
