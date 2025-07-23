#[derive(Debug, Clone)]
pub enum ModuleType {
    Native,
    Custom,
}

#[derive(Debug, Clone)]
pub struct ImportStatement {
    pub module: Vec<String>,
    pub module_type: ModuleType,
    pub members: Vec<String>,
    pub at: usize,
    pub line: usize,
}

impl ImportStatement {
    pub fn new(
        module: Vec<String>,
        module_type: ModuleType,
        members: Vec<String>,
        at: usize,
        line: usize,
    ) -> ImportStatement {
        ImportStatement {
            module,
            module_type,
            members,
            at,
            line,
        }
    }
}
