use crate::ast::{identifier::Identifier, object_type::ObjectType};

#[derive(Debug, Clone)]
pub struct Struct {
    pub identifier: Identifier,
    pub fields: ObjectType,
    pub at: usize,
    pub line: usize,
}

impl Struct {
    pub fn new(identifier: Identifier, fields: ObjectType, at: usize, line: usize) -> Struct {
        Struct {
            identifier,
            fields,
            at,
            line,
        }
    }
}
