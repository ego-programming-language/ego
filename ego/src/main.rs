mod ast;
mod commands;
mod compiler;
mod core;

use commands::Command;

fn main() {
    let command = Command::parse();
    command.exec();
}
