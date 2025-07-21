use crate::{
    heap::{HeapObject, HeapRef},
    vm::Vm,
};

pub fn put_string(vm: &mut Vm, string: String) -> HeapRef {
    vm.heap.allocate(HeapObject::String(string))
}
