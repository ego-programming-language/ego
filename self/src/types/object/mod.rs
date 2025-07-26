use crate::heap::HeapRef;

pub mod func;
pub mod native_struct;
pub mod structs;

#[derive(Debug, Clone)]
pub struct BoundAccess {
    object: HeapRef,
    property: HeapRef,
}

impl BoundAccess {
    pub fn new(object: HeapRef, property: HeapRef) -> Self {
        BoundAccess { object, property }
    }

    pub fn to_string(&self) -> String {
        format!(
            "property({}) of struct({})",
            self.property.get_address(),
            self.object.get_address()
        )
    }
}
