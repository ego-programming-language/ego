use std::env;
use std::fs;

fn main() {
    let args: Vec<String> = env::args().collect();
    let contents = match fs::read_to_string("./main.ego") {
        Ok(contents) => contents,
        Err(e) => {
            eprintln!("Error reading file: {}", e);
            return;
        }
    };
    let bytecode = ego::gen_bytecode("main".to_string(), contents, &args);
    let mut vm = self_vm::new(bytecode);
    if args.contains(&"-d".to_string()) {
        vm.debug_bytecode();
        println!("\n--- RUNTIME ----------\n");
    }
    let execution = vm.run(&args);
    if let Some(err) = execution.error {
        let error_msg = format!("[runtime error] {}: {}", err.message, err.semantic_message);
        println!("{error_msg}");
    }
}
