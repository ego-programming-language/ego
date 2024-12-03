#[derive(Debug, Clone)]
pub struct U64 {
    pub value: u64,
}
impl U64 {
    pub fn new(value: u64) -> U64 {
        U64 { value }
    }
}
