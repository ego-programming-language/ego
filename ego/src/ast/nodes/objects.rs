use std::collections::HashMap;

use crate::ast::{identifier::Identifier, Expression};

#[derive(Debug, Clone)]
pub struct ObjectType {
    pub fields: Vec<Identifier>,
    pub at: usize,
    pub line: usize,
}

impl ObjectType {
    pub fn new(at: usize, line: usize) -> ObjectType {
        ObjectType {
            fields: vec![],
            at,
            line,
        }
    }

    pub fn add_field(&mut self, field: Identifier) {
        self.fields.push(field);
    }
}

#[derive(Debug, Clone)]
pub struct ObjectLiteral {
    pub fields: Vec<(Identifier, Expression)>,
    pub at: usize,
    pub line: usize,
}

impl ObjectLiteral {
    pub fn new(at: usize, line: usize) -> ObjectLiteral {
        ObjectLiteral {
            fields: vec![],
            at,
            line,
        }
    }

    pub fn add_field(&mut self, field: Identifier, value: Expression) {
        self.fields.push((field, value));
    }
}
