use crate::{
    memory::{Handle, MemObject},
    vm::Vm,
};

pub fn put_string(vm: &mut Vm, string: String) -> Handle {
    vm.memory.alloc(MemObject::String(string))
}
