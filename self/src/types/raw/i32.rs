#[derive(Debug, Clone)]
pub struct I32 {
    pub value: i32,
}
impl I32 {
    pub fn new(value: i32) -> I32 {
        I32 { value }
    }
}
