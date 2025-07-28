pub enum TypeError {
    InvalidArgsCount { expected: u32, received: u32 },
}
