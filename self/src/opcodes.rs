use std::collections::HashMap;

pub fn get_codes_map() -> HashMap<String, u8> {
    let mut m = HashMap::new();
    // bytecode is generated using opcodes
    // that are structured on a level system.
    // More level means more nesting inside the
    // bytecode interpretation. Opcode can be repeated
    // if they are on different levels.

    // instructions opcodes - level: 0
    m.insert("zero".to_string(), 0x00);
    m.insert("load_const".to_string(), 0x01);
    m.insert("add".to_string(), 0x03);
    m.insert("store_var".to_string(), 0x04);

    // builtin functions opcode - level: 0
    m.insert("print".to_string(), 0x02);
    m.insert("call".to_string(), 0x06);

    // params - level 1
    m.insert("inmut".to_string(), 0x00);
    m.insert("mut".to_string(), 0x01);

    // typecodes - level 2
    m.insert("nothing".to_string(), 0x00);
    m.insert("i32".to_string(), 0x01);
    m.insert("i64".to_string(), 0x02);
    m.insert("u32".to_string(), 0x03);
    m.insert("u64".to_string(), 0x04);
    m.insert("utf8".to_string(), 0x05);
    m.insert("bool".to_string(), 0x06);
    m
}

#[derive(Debug)]
pub enum Opcode {
    Zero,
    LoadConst,
    Print,
    Add,
    StoreVar,
    Call,
    Unknown,
}

impl Opcode {
    pub fn to_opcode(opcode: u8) -> Opcode {
        match opcode {
            0x00 => Opcode::Zero,
            0x01 => Opcode::LoadConst,
            0x02 => Opcode::Print,
            0x03 => Opcode::Add,
            0x04 => Opcode::StoreVar,
            0x06 => Opcode::Call,
            _ => Opcode::Unknown,
        }
    }
}

#[derive(Debug)]
pub enum DataType {
    I32,
    I64,
    U32,
    U64,
    Utf8,
    Nothing,
    Bool,
    Unknown,
}

impl DataType {
    pub fn to_opcode(opcode: u8) -> DataType {
        match opcode {
            0x00 => DataType::Nothing,
            0x01 => DataType::I32,
            0x02 => DataType::I64,
            0x03 => DataType::U32,
            0x04 => DataType::U64,
            0x05 => DataType::Utf8,
            0x06 => DataType::Bool,
            _ => DataType::Unknown,
        }
    }
}

impl PartialEq for DataType {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (DataType::I32, DataType::I32) => true,
            (DataType::I64, DataType::I64) => true,
            (DataType::U32, DataType::U32) => true,
            (DataType::U64, DataType::U64) => true,
            (DataType::Utf8, DataType::Utf8) => true,
            (DataType::Bool, DataType::Bool) => true,
            (DataType::Nothing, DataType::Nothing) => true,
            _ => false,
        }
    }
}
