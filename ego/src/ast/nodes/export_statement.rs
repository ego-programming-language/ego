use super::Expression;

#[derive(Debug, Clone)]
pub struct ExportStatement {
    pub value: Expression,
    pub at: usize,
    pub line: usize,
}

impl ExportStatement {
    pub fn new(value: Expression, at: usize, line: usize) -> ExportStatement {
        ExportStatement { value, at, line }
    }
}
