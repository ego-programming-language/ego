use crate::ast::{
    identifier::Identifier,
    member_expression::MemberExpression,
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
pub enum StructTypeExpr {
    Identifier(Identifier),
    MemberExpression(Box<MemberExpression>),
}

#[derive(Debug, Clone)]
pub struct StructLiteral {
    pub identifier: StructTypeExpr,
    pub fields: ObjectLiteral,
    pub at: usize,
    pub line: usize,
}

impl StructLiteral {
    pub fn new(
        identifier: StructTypeExpr,
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
