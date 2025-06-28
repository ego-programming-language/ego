use crate::types::Value;
use std::collections::HashMap;

// CALL STACK
pub struct CallStack {
    stack: Vec<StackFrame>,
}

impl CallStack {
    pub fn new() -> CallStack {
        CallStack {
            stack: vec![StackFrame::new(0)],
        }
    }
    pub fn push() {}
    pub fn pop() {}
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
}

pub struct StackFrame {
    return_pc: usize,
    symbols: HashMap<String, Value>,
}

impl StackFrame {
    pub fn new(return_pc: usize) -> StackFrame {
        StackFrame {
            return_pc: return_pc,
            symbols: HashMap::new(),
        }
    }

    pub fn put(&mut self, key: String, value: Value) -> Option<Value> {
        self.symbols.insert(key, value)
    }

    pub fn get(&self, key: &str) -> Option<Value> {
        if let Some(var) = self.symbols.get(key) {
            return Some(var.clone());
        }

        None
    }
}

// OPERANDS_STACK VALUE
#[derive(Debug, Clone)]
pub struct OperandsStackValue {
    pub value: Value,
    pub origin: Option<String>,
}
