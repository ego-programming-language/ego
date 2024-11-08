use crate::opcodes::DataType;


#[derive(Debug)]
pub enum Instruction {
    Zero,
    LoadConst { data_type: DataType, value: Vec<u8> },
    StoreVar { data_type: DataType, value: Vec<u8>, mutable: bool },
    Add,
    Print { number_of_args: u32 },
}
