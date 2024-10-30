use self_vm::{
    utils::{to_bytes::bytes_from_32, to_bytes::bytes_from_64, Number},
    vm::Vm,
};

fn main() {
    // print two nums
    let mut instructions: Vec<u8> = vec![0x01, 0x01];
    instructions.extend_from_slice(&bytes_from_32(Number::U32(14)));
    instructions.push(0x01);
    instructions.push(0x02);
    instructions.extend_from_slice(&bytes_from_64(Number::U64(20)));
    instructions.push(0x02);
    instructions.extend_from_slice(&bytes_from_32(Number::U32(2)));

    // store a variable
    instructions.extend_from_slice(&[0x04, 0x01]);
    instructions.extend_from_slice(&bytes_from_32(Number::U32(14)));
    instructions.push(0x01);

    let mut vm = Vm::new(instructions);
    vm.run();
}
