use crate::types::Value;
use std::collections::HashMap;

// CALL STACK
#[derive(Debug)]
pub struct CallStack {
    stack: Vec<StackFrame>,
}

impl CallStack {
    pub fn new() -> CallStack {
        CallStack {
            stack: vec![StackFrame::new(0)],
        }
    }
    pub fn push(&mut self) {
        // we (maybe) should save here the return pc
        self.stack.push(StackFrame::new(0));
    }
    pub fn pop(&mut self) -> Option<StackFrame> {
        self.stack.pop()
    }
    pub fn put_to_frame(&mut self, key: String, value: Value) {
        let last = self.stack.len() - 1;
        self.stack[last].put(key, value);
    }
    pub fn resolve(&self, key: &str) -> Option<Value> {
        for frame in self.stack.iter().rev() {
            if let Some(var) = frame.get(key) {
                return Some(var.clone());
            }
        }

        None
    }
    pub fn add_export(&mut self, key: String) {
        let last = self.stack.len() - 1;
        self.stack[last].add_export(key);
    }
}

#[derive(Debug)]
pub struct StackFrame {
    return_pc: usize,
    symbols: HashMap<String, Value>,
    exports: Vec<String>,
}

impl StackFrame {
    pub fn new(return_pc: usize) -> StackFrame {
        StackFrame {
            return_pc: return_pc,
            symbols: HashMap::new(),
            exports: vec![],
        }
    }

    pub fn put(&mut self, key: String, value: Value) -> Option<Value> {
        self.symbols.insert(key, value)
    }

    pub fn add_export(&mut self, key: String) {
        self.exports.push(key);
    }

    pub fn get(&self, key: &str) -> Option<Value> {
        if let Some(var) = self.symbols.get(key) {
            return Some(var.clone());
        }

        None
    }

    pub fn get_exports(&mut self) -> HashMap<String, Value> {
        let mut exports = HashMap::new();
        for export in &self.exports {
            let var = self.get(export);
            if let Some(v) = var {
                exports.insert(export.to_string(), v);
            } else {
                // here we should handle the case that
                // we're exporting an undefined identifier
                // TODO: use self-vm errors system
                println!("cannot export");
            }
        }

        exports
    }
}

// OPERANDS_STACK VALUE
#[derive(Debug, Clone)]
pub struct OperandsStackValue {
    pub value: Value,
    pub origin: Option<String>,
}
