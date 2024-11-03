#[derive(Debug, Clone)]
pub struct StringLiteral {
    pub value: String,
    pub raw_value: String,
    pub at: usize,
    pub line: usize,
}

impl StringLiteral {
    pub fn new(value: String, raw_value: String, at: usize, line: usize) -> StringLiteral {
        StringLiteral {
            value,
            raw_value,
            at,
            line,
        }
    }
}
