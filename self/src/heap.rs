use std::collections::HashMap;

use crate::memory::MemObject;
use crate::vm::Vm;

#[derive(Debug)]
pub struct Heap {
    memory: HashMap<usize, MemObject>,
    next_address: usize,
}

#[derive(Debug, Clone)]
pub struct HeapRef {
    address: usize,
}

impl Heap {
    pub fn new() -> Self {
        Heap {
            memory: HashMap::new(),
            next_address: 0,
        }
    }

    pub fn allocate(&mut self, obj: MemObject) -> HeapRef {
        let address = self.next_address;
        self.next_address += 1;
        self.memory.insert(address, obj);
        HeapRef::new(address)
    }

    pub fn get(&self, heap_ref: HeapRef) -> Option<&MemObject> {
        self.memory.get(&heap_ref.address)
    }

    pub fn get_mut(&mut self, heap_ref: HeapRef) -> Option<&mut MemObject> {
        self.memory.get_mut(&heap_ref.address)
    }

    pub fn free(&mut self, heap_ref: HeapRef) -> Option<MemObject> {
        self.memory.remove(&heap_ref.address)
    }

    // we do not free memory for the moment xD
}

impl HeapRef {
    pub fn new(address: usize) -> Self {
        HeapRef { address }
    }

    pub fn get_address(&self) -> usize {
        self.address
    }
}
