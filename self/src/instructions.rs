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
        mutable: bool,
    },
    JumpIfFalse,
    Jump,
    Add,
    Substract,
    Multiply,
    Divide,
    GreaterThan,
    LessThan,
    Equals,
    NotEquals,
    FuncDec {
        identifier: String,
    },
    Print {
        number_of_args: u32,
    },
    Println {
        number_of_args: u32,
    },
    Call {
        number_of_args: u32,
    },
    Unknown,
}

impl Instruction {
    pub fn get_type(&self) -> String {
        match self {
            Instruction::Zero => "Zero".to_string(),
            Instruction::LoadConst { data_type, value } => "LoadConst".to_string(),
            Instruction::LoadVar {
                data_type,
                identifier,
            } => "LoadVar".to_string(),
            Instruction::StoreVar {
                identifier,
                mutable,
            } => "StoreVar".to_string(),
            Instruction::JumpIfFalse => "JumpIfFalse".to_string(),
            Instruction::Jump => "Jump".to_string(),
            Instruction::Add => "Add".to_string(),
            Instruction::Substract => "Substract".to_string(),
            Instruction::Multiply => "Multiply".to_string(),
            Instruction::Divide => "Divide".to_string(),
            Instruction::GreaterThan => "GreaterThan".to_string(),
            Instruction::LessThan => "LessThan".to_string(),
            Instruction::Equals => "Equals".to_string(),
            Instruction::NotEquals => "NotEquals".to_string(),
            Instruction::FuncDec { identifier } => "FuncDec".to_string(),
            Instruction::Print { number_of_args } => "Print".to_string(),
            Instruction::Println { number_of_args } => "Println".to_string(),
            Instruction::Call { number_of_args } => "Call".to_string(),
            Instruction::Unknown => "Unknown".to_string(),
        }
    }
}
