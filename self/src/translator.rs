use crate::{
    instructions::Instruction,
    types::{DataType, Value},
    utils::from_bytes::bytes_to_data,
};

pub struct Translator {
    bytecode: Vec<u8>,
    pc: usize,
}

impl Translator {
    pub fn new(bytecode: Vec<u8>) -> Translator {
        Translator { bytecode, pc: 0 }
    }

    pub fn translate(&mut self) -> Vec<Instruction> {
        let mut instructions = vec![];

        while self.pc < self.bytecode.len() {
            match self.bytecode[self.pc] {
                // ZERO
                0 => instructions.push(Instruction::Zero),
                // LOAD_CONST
                0x01 => {
                    if self.pc + 1 >= self.bytecode.len() {
                        panic!("Invalid LOAD_CONST instruction at position {}", self.pc);
                    }

                    self.pc += 1;
                    let (data_type, value_bytes) = self.get_value_length();

                    instructions.push(Instruction::LoadConst {
                        data_type,
                        value: value_bytes,
                    });
                }
                // PRINT
                0x02 => {
                    // get u32 value. 4 bytes based on the type plus the current
                    let value_length = 4;
                    if self.pc + value_length >= self.bytecode.len() {
                        panic!("Invalid print instruction at position {}", self.pc);
                    }

                    let value_bytes = &self.bytecode[self.pc + 1..self.pc + 5];
                    let number_of_args = u32::from_le_bytes(
                        value_bytes.try_into().expect("Provided value is incorrect"),
                    );
                    instructions.push(Instruction::Print { number_of_args });
                    self.pc += 4;
                }
                // ADD
                0x03 => instructions.push(Instruction::Add),
                // STORE_VAR
                0x04 => {
                    if self.pc + 1 >= self.bytecode.len() {
                        panic!("Invalid STORE_VAR instruction at position {}.", self.pc);
                    }

                    self.pc += 1;
                    let (data_type, value_bytes) = self.get_value_length();

                    // 0x01 inmutable | 0x02 mutable
                    self.pc += 1;
                    let mutable = match self.bytecode[self.pc] {
                        0x01 => false,
                        0x02 => true,
                        _ => {
                            panic!("Invalid STORE_VAR instruction at position {}. Needed mutability property.", self.pc);
                        }
                    };

                    instructions.push(Instruction::StoreVar {
                        data_type,
                        mutable,
                        value: value_bytes,
                    });
                }
                _ => {}
            };

            self.pc += 1;
        }

        instructions
    }

    fn get_value_length(&mut self) -> (DataType, Vec<u8>) {
        let data_type = match self.bytecode[self.pc] {
            0x00 => DataType::Nothing,
            0x01 => DataType::U32,
            0x02 => DataType::U64,
            0x03 => DataType::I32,
            0x04 => DataType::I64,
            0x05 => DataType::Utf8,
            _ => panic!("Unknown data type"),
        };

        let value_length = match data_type {
            DataType::I32 => 4,
            DataType::I64 => 8,
            DataType::U32 => 4,
            DataType::U64 => 8,
            DataType::Utf8 => {
                self.pc += 1;
                let (data_type, value) = self.get_value_length();
                if data_type != DataType::U32 {
                    panic!("bad utf8 value length")
                }

                let (string_length, _) = bytes_to_data(&DataType::U32, &value);
                if let Value::U32(val) = string_length {
                    val.value as usize
                } else {
                    panic!("Unexpected value type for string length");
                }
            } // hardcoded for the moment
            DataType::Nothing => 0,
            _ => {
                panic!("Unsupported datatype")
            }
        };

        if (self.pc + value_length) >= self.bytecode.len() {
            panic!("Invalid value size at position {}", self.pc + 1);
        };

        let value_bytes = self.bytecode[self.pc + 1..self.pc + 1 + value_length].to_vec();
        self.pc += value_length;

        (data_type, value_bytes)
    }
}
