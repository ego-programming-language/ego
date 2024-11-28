use serde::Deserialize;
use std::collections::HashMap;

#[derive(Debug, Deserialize)]
pub struct Argument {
    pub name: String,
    pub arg_type: String,
}

#[derive(Debug, Deserialize)]
pub struct ForeignHandler {
    pub name: String,
    pub runtime: String,
    pub args: Vec<Argument>,
}

#[derive(Debug, Deserialize)]
pub struct ForeignHandlersToml {
    pub functions: Vec<ForeignHandler>,
}

#[derive(Debug)]
pub struct ForeignHandlers {
    pub handlers: HashMap<String, ForeignHandler>,
}

impl ForeignHandlers {
    pub fn new() -> ForeignHandlers {
        ForeignHandlers {
            handlers: HashMap::new(),
        }
    }

    pub fn add(&mut self, handler: ForeignHandler) {
        self.handlers.insert(handler.name.clone(), handler);
    }
}
