use crate::ast::{identifier::Identifier, Expression};

#[derive(Debug, Clone)]
pub struct MemberExpression {
    pub object: Box<Expression>,
    pub property: Identifier,
    pub at: usize,
    pub line: usize,
}

impl MemberExpression {
    pub fn new(
        object: Box<Expression>,
        property: Identifier,
        at: usize,
        line: usize,
    ) -> MemberExpression {
        MemberExpression {
            object,
            property,
            at,
            line,
        }
    }
}
