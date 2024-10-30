use crate::types::Value;
use std::collections::HashMap;

pub struct SymbolTable {
    scopes: Vec<HashMap<String, Value>>,
    sc: usize,
}

impl SymbolTable {
    pub fn new() -> SymbolTable {
        SymbolTable {
            scopes: vec![HashMap::new()],
            sc: 0,
        }
    }

    pub fn add_key_value(&mut self, key: String, value: Value) -> Option<Value> {
        self.scopes[self.sc].insert(key, value)
    }
}
