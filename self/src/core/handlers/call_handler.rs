use std::process::Command;

use crate::types::Value;

use super::foreign_handlers::ForeignHandlers;

pub fn call_handler(foreign_handlers: &ForeignHandlers, args: Vec<Value>) {
    if args.len() < 1 {
        panic!("Call handler requires at least 1 arg");
    }

    let requested_handler = &args[0];
    let handler = foreign_handlers
        .handlers
        .get(&requested_handler.to_string());

    let handler = match handler {
        Some(val) => val,
        None => panic!("Calling an unset handler"),
    };

    // parsed_args must be populated with the ego
    // call builtin handler args and match with the
    // foreign handler definition args
    let mut parsed_args: Vec<&String> = vec![];
    // for arg in &handler.args {
    //     parsed_args.push(&arg.name);
    // }

    spawn_process(&handler.runtime, &handler.script, parsed_args);
}

fn spawn_process(binary: &String, script: &String, args: Vec<&String>) {
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
