#[derive(Debug, Clone)]
pub struct Bool {
    pub value: bool,
}
impl Bool {
    pub fn new(value: bool) -> Bool {
        Bool { value }
    }
}
