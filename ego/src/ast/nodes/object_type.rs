use crate::ast::identifier::Identifier;

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
