use self_vm::vm::Vm;

fn main() {
    let bytecode = ego::gen_bytecode("print(12)".to_string());
    let mut vm = Vm::new(bytecode);
    vm.run();
}
