use crate::ast::{
    identifier::Identifier,
    objects::{ObjectLiteral, ObjectType},
};

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

#[derive(Debug, Clone)]
pub struct StructLiteral {
    pub identifier: Identifier,
    pub fields: ObjectLiteral,
    pub at: usize,
    pub line: usize,
}

impl StructLiteral {
    pub fn new(
        identifier: Identifier,
        fields: ObjectLiteral,
        at: usize,
        line: usize,
    ) -> StructLiteral {
        StructLiteral {
            identifier,
            fields,
            at,
            line,
        }
    }
}
