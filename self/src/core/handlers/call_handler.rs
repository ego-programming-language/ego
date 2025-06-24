use std::process::Command;

use crate::types::Value;

use super::foreign_handlers::ForeignHandlers;

pub fn call_handler(foreign_handlers: &ForeignHandlers, args: Vec<String>) {
    if args.len() < 1 {
        panic!("Call handler requires at least 1 arg");
    }

    let handler = foreign_handlers.handlers.get(&args[0]);
    let handler = match handler {
        Some(val) => val,
        None => panic!("Calling an unset handler"),
    };

    spawn_process(&handler.runtime, &handler.script, args[1..].to_vec());
}

fn spawn_process(binary: &String, script: &String, args: Vec<String>) {
    let output = Command::new(binary).arg(script).args(args).output();
    let output = match output {
        Ok(val) => val,
        Err(_) => panic!("Cannot spawn a foreign handler"),
    };

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        println!("{}", stdout);
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        eprintln!("Error executing foreign handler:\n{}", stderr);
    }
}
