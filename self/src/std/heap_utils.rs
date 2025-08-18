use crate::{heap::HeapRef, memory::MemObject, vm::Vm};

pub fn put_string(vm: &mut Vm, string: String) -> HeapRef {
    vm.heap.allocate(MemObject::String(string))
}
