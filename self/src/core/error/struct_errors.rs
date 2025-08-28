#[derive(Debug)]
pub enum StructError {
    FieldNotFound { field: String, struct_type: String },
}
