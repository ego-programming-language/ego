use crate::heap::Heap;

pub struct MemoryManager {
    heap: Heap,
}

impl MemoryManager {
    pub fn new() -> MemoryManager {
        MemoryManager { heap: Heap::new() }
    }
}
