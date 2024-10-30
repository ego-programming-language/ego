fn main() {
    let bytecode = ego::gen_bytecode("print(12); ".to_string());
    let mut vm = self_vm::new(bytecode);
    vm.run();
}
