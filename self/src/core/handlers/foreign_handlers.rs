use std::collections::HashMap;

use crate::types::Value;

struct ForeignHandler {
    name: String,
    binary: bool,
    interpreter: String,
    args: Vec<Value>,
}

impl ForeignHandler {
    fn new(name: String, binary: bool, interpreter: String, args: Vec<Value>) -> ForeignHandler {
        ForeignHandler {
            name,
            binary,
            interpreter,
            args,
        }
    }
}

pub struct ForeignHandlers {
    handlers: HashMap<String, ForeignHandler>,
}

impl ForeignHandlers {
    pub fn new() -> ForeignHandlers {
        ForeignHandlers {
            handlers: HashMap::new(),
        }
    }

    pub fn add(&mut self, name: String, binary: bool, interpreter: String, args: Vec<Value>) {
        self.handlers.insert(
            name.clone(),
            ForeignHandler::new(name, binary, interpreter, args),
        );
    }
}
