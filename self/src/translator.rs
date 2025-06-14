use crate::{
    instructions::Instruction,
    opcodes::{DataType, Opcode},
    types::{utf8::Utf8, Value},
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
            match Opcode::to_opcode(self.bytecode[self.pc]) {
                Opcode::Zero => instructions.push(Instruction::Zero),
                Opcode::LoadConst => {
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
                Opcode::LoadVar => {
                    if self.pc + 1 >= self.bytecode.len() {
                        panic!("Invalid LOAD_VAR instruction at position {}", self.pc);
                    }

                    self.pc += 1;
                    let (data_type, value_bytes) = self.get_value_length();

                    instructions.push(Instruction::LoadVar {
                        data_type,
                        identifier: value_bytes,
                    });
                }
                Opcode::Print => {
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
                Opcode::Println => {
                    // get u32 value. 4 bytes based on the type plus the current
                    let value_length = 4;
                    if self.pc + value_length >= self.bytecode.len() {
                        panic!("Invalid print instruction at position {}", self.pc);
                    }

                    let value_bytes = &self.bytecode[self.pc + 1..self.pc + 5];
                    let number_of_args = u32::from_le_bytes(
                        value_bytes.try_into().expect("Provided value is incorrect"),
                    );
                    instructions.push(Instruction::Println { number_of_args });
                    self.pc += 4;
                }
                Opcode::Add => instructions.push(Instruction::Add),
                Opcode::Substract => instructions.push(Instruction::Substract),
                Opcode::Multiply => instructions.push(Instruction::Multiply),
                Opcode::Divide => instructions.push(Instruction::Divide),
                Opcode::StoreVar => {
                    if self.pc + 1 >= self.bytecode.len() {
                        panic!("Invalid STORE_VAR instruction at position {}.", self.pc);
                    } else {
                        self.pc += 1;
                    }

                    // 0x00 inmutable | 0x00 mutable
                    let mutable = match self.bytecode[self.pc] {
                        0x00 => false,
                        0x01 => true,
                        _ => {
                            panic!("Invalid STORE_VAR instruction at position {}. Needed mutability property.", self.pc);
                        }
                    };
                    self.pc += 1;

                    // identifier
                    let (identifier_data_type, identifier_bytes) = self.get_value_length();
                    if identifier_data_type != DataType::Utf8 {
                        panic!("Identifier type should be a string encoded as utf8")
                    }

                    let identifier_name = String::from_utf8(identifier_bytes)
                        .expect("Identifier bytes should be valid UTF-8");

                    instructions.push(Instruction::StoreVar {
                        identifier: identifier_name,
                        mutable,
                    });
                }
                Opcode::Call => {
                    // get u32 value. 4 bytes based on the type plus the current
                    let value_length = 4;
                    if self.pc + value_length >= self.bytecode.len() {
                        panic!("Invalid print instruction at position {}", self.pc);
                    }

                    let value_bytes = &self.bytecode[self.pc + 1..self.pc + 5];
                    let number_of_args = u32::from_le_bytes(
                        value_bytes.try_into().expect("Provided value is incorrect"),
                    );
                    instructions.push(Instruction::Call { number_of_args });
                    self.pc += 4;
                }
                _ => {}
            };

            self.pc += 1;
        }

        instructions
    }

    fn get_value_length(&mut self) -> (DataType, Vec<u8>) {
        let data_type = DataType::to_opcode(self.bytecode[self.pc]);
        let value_length = match data_type {
            DataType::I32 => 4,
            DataType::I64 => 8,
            DataType::U32 => 4,
            DataType::U64 => 8,
            DataType::F64 => 8,
            DataType::Nothing => 0,
            DataType::Bool => 1,
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
