use super::Expression;

#[derive(Debug, Clone)]
pub struct Vector {
    pub children: Vec<Expression>,
    pub at: usize,
    pub line: usize,
}

impl Vector {
    pub fn new(at: usize, line: usize) -> Vector {
        Vector {
            children: vec![],
            at,
            line,
        }
    }
    pub fn add_child(&mut self, node: Expression) {
        self.children.push(node);
    }
}
