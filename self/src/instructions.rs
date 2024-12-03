use crate::opcodes::DataType;

#[derive(Clone, Debug)]
pub enum Instruction {
    Zero,
    LoadConst {
        data_type: DataType,
        value: Vec<u8>,
    },
    LoadVar {
        data_type: DataType,
        identifier: Vec<u8>,
    },
    StoreVar {
        identifier: String,
        data_type: DataType,
        value: Vec<u8>,
        mutable: bool,
    },
    Add,
    Print {
        number_of_args: u32,
    },
    Call {
        number_of_args: u32,
    },
}
