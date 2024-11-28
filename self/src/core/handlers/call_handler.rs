use crate::types::Value;

use super::foreign_handlers::ForeignHandlers;

pub fn call_handler(handlers: &ForeignHandlers, args: Vec<Value>) {
    println!("available handlers: {:#?}", handlers)
}
