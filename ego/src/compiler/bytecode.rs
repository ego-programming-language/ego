use std::collections::HashMap;

use crate::core::error::{self, ErrorType};

pub struct Bytecode {
    table: HashMap<String, u8>,
}

impl Bytecode {
    pub fn get_handler() -> Bytecode {
        let mut hash_map = HashMap::new();
        // instructions
        hash_map.insert("load_const".to_string(), 0x01);
        hash_map.insert("print".to_string(), 0x02);

        // values
        hash_map.insert("i64".to_string(), 0x04);
        hash_map.insert("i32".to_string(), 0x03);
        hash_map.insert("u32".to_string(), 0x01);
        hash_map.insert("utf8".to_string(), 0x05);
        hash_map.insert("bool".to_string(), 0x06);
        Bytecode { table: hash_map }
    }

    pub fn get_bytecode_representation(&mut self, key: String) -> Option<u8> {
        self.table.get(&key).copied()
    }
}

pub fn get_bytecode(item: String) -> u8 {
    let mut bytecode_handler = Bytecode::get_handler();

    if let Some(bytecode) = bytecode_handler.get_bytecode_representation(item) {
        bytecode
    } else {
        error::throw(
            ErrorType::CompilationError,
            "Member name not recognized",
            None,
        );
        std::process::exit(1)
    }
}
