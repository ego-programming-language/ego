#[derive(Debug)]
pub struct U32 {
    pub value: u32,
}
impl U32 {
    pub fn new(value: u32) -> U32 {
        U32 { value }
    }
}
