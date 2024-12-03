#[derive(Debug, Clone)]
pub struct Utf8 {
    pub value: String,
}
impl Utf8 {
    pub fn new(value: String) -> Utf8 {
        Utf8 { value }
    }
}
