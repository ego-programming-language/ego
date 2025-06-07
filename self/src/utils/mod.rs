pub mod foreign_handlers_utils;
pub mod from_bytes;
pub mod to_bytes;

pub enum Number {
    U64(u64),
    U32(u32),
    I64(i64),
    I32(i32),
    F64(f64),
}
