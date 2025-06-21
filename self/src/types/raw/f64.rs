#[derive(Debug, Clone)]
pub struct F64 {
    pub value: f64,
}
impl F64 {
    pub fn new(value: f64) -> F64 {
        F64 { value }
    }
}
