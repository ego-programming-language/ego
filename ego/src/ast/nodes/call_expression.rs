use crate::ast::Expression;

use super::group::Group;

#[derive(Debug, Clone)]
pub struct CallExpression {
    //pub type: String,
    pub callee: Box<Expression>,
    pub arguments: Group,
    pub at: usize,
    pub line: usize,
}

impl CallExpression {
    pub fn new(
        callee: Box<Expression>,
        arguments: Group,
        at: usize,
        line: usize,
    ) -> CallExpression {
        CallExpression {
            callee,
            arguments,
            at,
            line,
        }
    }

    pub fn get_callee(&self) -> String {
        match self.callee.as_ref() {
            Expression::Identifier(i) => i.name.clone(),
            Expression::MemberExpression(x) => x.property.name.clone(),
            _ => {
                panic!("Unhandled callee for CallExpression")
            }
        }
    }
}
