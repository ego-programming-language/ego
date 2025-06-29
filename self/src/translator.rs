/*
    THIS FILE ONLY RUNS ON DEBUGGING MODE
    TO ALLOW ENUMERATION OF EACH INSTRUCTION
    AND EACH DATA
*/

use crate::{
    instructions::Instruction,
    opcodes::{DataType, Opcode},
    types::{raw::RawValue, Value},
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

    fn new_with_pc(bytecode: Vec<u8>, pc: usize) -> Translator {
        Translator { bytecode, pc: pc }
    }

    pub fn get_instruction(pc: usize, bytecode: &Vec<u8>) -> (Instruction, usize) {
        let mut t = Translator::new_with_pc(bytecode.clone(), pc);

        match Opcode::to_opcode(t.bytecode[t.pc]) {
            Opcode::Zero => (Instruction::Zero, 1),
            Opcode::LoadConst => {
                if t.pc + 1 >= t.bytecode.len() {
                    panic!("Invalid LOAD_CONST instruction at position {}", t.pc);
                }

                t.pc += 1;
                let (data_type, value_bytes) = t.get_value_length();

                (
                    Instruction::LoadConst {
                        data_type,
                        value: value_bytes,
                    },
                    pc.abs_diff(t.pc),
                )
            }
            Opcode::LoadVar => {
                if t.pc + 1 >= t.bytecode.len() {
                    panic!("Invalid LOAD_VAR instruction at position {}", t.pc);
                }

                t.pc += 1;
                let (data_type, value_bytes) = t.get_value_length();

                (
                    Instruction::LoadVar {
                        data_type,
                        identifier: value_bytes,
                    },
                    pc.abs_diff(t.pc),
                )
            }
            Opcode::JumpIfFalse => {
                t.pc += 4;
                (Instruction::JumpIfFalse, pc.abs_diff(t.pc))
            }
            Opcode::Jump => {
                t.pc += 4;
                (Instruction::Jump, pc.abs_diff(t.pc))
            }
            Opcode::Print => {
                // get u32 value. 4 bytes based on the type plus the current
                let value_length = 4;
                if t.pc + value_length >= t.bytecode.len() {
                    panic!("Invalid print instruction at position {}", t.pc);
                }

                let value_bytes = &t.bytecode[t.pc + 1..t.pc + 5];
                let number_of_args = u32::from_le_bytes(
                    value_bytes.try_into().expect("Provided value is incorrect"),
                );
                t.pc += 4;
                (Instruction::Print { number_of_args }, pc.abs_diff(t.pc))
            }
            Opcode::Println => {
                // get u32 value. 4 bytes based on the type plus the current
                let value_length = 4;
                if t.pc + value_length >= t.bytecode.len() {
                    panic!("Invalid print instruction at position {}", t.pc);
                }

                let value_bytes = &t.bytecode[t.pc + 1..t.pc + 5];
                let number_of_args = u32::from_le_bytes(
                    value_bytes.try_into().expect("Provided value is incorrect"),
                );
                t.pc += 4;
                (Instruction::Println { number_of_args }, pc.abs_diff(t.pc))
            }
            Opcode::Add => (Instruction::Add, 0),
            Opcode::Substract => (Instruction::Substract, 0),
            Opcode::Multiply => (Instruction::Multiply, 0),
            Opcode::Divide => (Instruction::Divide, 0),
            Opcode::GreaterThan => (Instruction::GreaterThan, 0),
            Opcode::LessThan => (Instruction::LessThan, 0),
            Opcode::Equals => (Instruction::Equals, 0),
            Opcode::NotEquals => (Instruction::NotEquals, 0),
            Opcode::FuncDec => {
                // identifier
                if t.pc + 1 >= t.bytecode.len() {
                    panic!("Invalid FUNC_DEC instruction at position {}", t.pc);
                }

                t.pc += 1;
                let (_, value_bytes) = t.get_value_length(); // t's pc gets modified by get_value_length
                let identifier_name =
                    String::from_utf8(value_bytes).expect("Identifier bytes should be valid UTF-8");

                // function body length
                if t.pc + 4 >= t.bytecode.len() {
                    panic!("Invalid FUNC_DEC instruction at position {}", t.pc);
                }
                t.pc += 4;

                let value_bytes = &t.bytecode[t.pc + 1..t.pc + 5];
                let body_length = u32::from_le_bytes(
                    value_bytes.try_into().expect("Provided value is incorrect"),
                );
                t.pc += 4 + body_length as usize;

                (
                    Instruction::FuncDec {
                        identifier: identifier_name,
                    },
                    pc.abs_diff(t.pc),
                )
            }
            Opcode::StoreVar => {
                if t.pc + 1 >= t.bytecode.len() {
                    panic!("Invalid STORE_VAR instruction at position {}.", t.pc);
                } else {
                    t.pc += 1;
                }

                // 0x00 inmutable | 0x00 mutable
                let mutable = match t.bytecode[t.pc] {
                    0x00 => false,
                    0x01 => true,
                    _ => {
                        panic!("Invalid STORE_VAR instruction at position {}. Needed mutability property.", t.pc);
                    }
                };
                t.pc += 1;

                // identifier
                let (identifier_data_type, identifier_bytes) = t.get_value_length();
                if identifier_data_type != DataType::Utf8 {
                    panic!("Identifier type should be a string encoded as utf8")
                }

                let identifier_name = String::from_utf8(identifier_bytes)
                    .expect("Identifier bytes should be valid UTF-8");

                (
                    Instruction::StoreVar {
                        identifier: identifier_name,
                        mutable,
                    },
                    pc.abs_diff(t.pc),
                )
            }
            Opcode::Call => {
                // get u32 value. 4 bytes based on the type plus the current
                let value_length = 4;
                if t.pc + value_length >= t.bytecode.len() {
                    panic!("Invalid print instruction at position {}", t.pc);
                }

                let value_bytes = &t.bytecode[t.pc + 1..t.pc + 5];
                let number_of_args = u32::from_le_bytes(
                    value_bytes.try_into().expect("Provided value is incorrect"),
                );
                t.pc += 4;
                (Instruction::Call { number_of_args }, pc.abs_diff(t.pc))
            }
            Opcode::Unknown => (Instruction::Unknown, 1),
            _ => (Instruction::Unknown, 1),
        }
    }

    pub fn get_instruction_info(instruction: &Instruction) -> String {
        match instruction {
            Instruction::LoadConst { data_type, value } => data_type.as_str().to_string(),
            Instruction::LoadVar {
                data_type,
                identifier,
            } => data_type.as_str().to_string(),
            Instruction::StoreVar {
                identifier,
                mutable,
            } => identifier.to_string(),
            Instruction::Print { number_of_args } => number_of_args.to_string(),
            Instruction::Println { number_of_args } => number_of_args.to_string(),
            Instruction::Call { number_of_args } => number_of_args.to_string(),
            Instruction::FuncDec { identifier } => identifier.to_string(),
            _ => "".to_string(),
        }
    }

    // pub fn translate(&mut self) -> Vec<Instruction> {
    //     let mut instructions = vec![];

    //     while self.pc < self.bytecode.len() {
    //         match Opcode::to_opcode(self.bytecode[self.pc]) {
    //             Opcode::Zero => instructions.push(Instruction::Zero),
    //             Opcode::LoadConst => {
    //                 if self.pc + 1 >= self.bytecode.len() {
    //                     panic!("Invalid LOAD_CONST instruction at position {}", self.pc);
    //                 }

    //                 self.pc += 1;
    //                 let (data_type, value_bytes) = self.get_value_length();

    //                 instructions.push(Instruction::LoadConst {
    //                     data_type,
    //                     value: value_bytes,
    //                 });
    //             }
    //             Opcode::LoadVar => {
    //                 if self.pc + 1 >= self.bytecode.len() {
    //                     panic!("Invalid LOAD_VAR instruction at position {}", self.pc);
    //                 }

    //                 self.pc += 1;
    //                 let (data_type, value_bytes) = self.get_value_length();

    //                 instructions.push(Instruction::LoadVar {
    //                     data_type,
    //                     identifier: value_bytes,
    //                 });
    //             }
    //             Opcode::JumpIfFalse => {
    //                 instructions.push(Instruction::JumpIfFalse);
    //                 self.pc += 4;
    //             }
    //             Opcode::Jump => {
    //                 instructions.push(Instruction::Jump);
    //                 self.pc += 4;
    //             }
    //             Opcode::Print => {
    //                 // get u32 value. 4 bytes based on the type plus the current
    //                 let value_length = 4;
    //                 if self.pc + value_length >= self.bytecode.len() {
    //                     panic!("Invalid print instruction at position {}", self.pc);
    //                 }

    //                 let value_bytes = &self.bytecode[self.pc + 1..self.pc + 5];
    //                 let number_of_args = u32::from_le_bytes(
    //                     value_bytes.try_into().expect("Provided value is incorrect"),
    //                 );
    //                 instructions.push(Instruction::Print { number_of_args });
    //                 self.pc += 4;
    //             }
    //             Opcode::Println => {
    //                 // get u32 value. 4 bytes based on the type plus the current
    //                 let value_length = 4;
    //                 if self.pc + value_length >= self.bytecode.len() {
    //                     panic!("Invalid print instruction at position {}", self.pc);
    //                 }

    //                 let value_bytes = &self.bytecode[self.pc + 1..self.pc + 5];
    //                 let number_of_args = u32::from_le_bytes(
    //                     value_bytes.try_into().expect("Provided value is incorrect"),
    //                 );
    //                 instructions.push(Instruction::Println { number_of_args });
    //                 self.pc += 4;
    //             }
    //             Opcode::Add => instructions.push(Instruction::Add),
    //             Opcode::Substract => instructions.push(Instruction::Substract),
    //             Opcode::Multiply => instructions.push(Instruction::Multiply),
    //             Opcode::Divide => instructions.push(Instruction::Divide),
    //             Opcode::GreaterThan => instructions.push(Instruction::GreaterThan),
    //             Opcode::LessThan => instructions.push(Instruction::LessThan),
    //             Opcode::Equals => instructions.push(Instruction::Equals),
    //             Opcode::NotEquals => instructions.push(Instruction::NotEquals),
    //             Opcode::StoreVar => {
    //                 if self.pc + 1 >= self.bytecode.len() {
    //                     panic!("Invalid STORE_VAR instruction at position {}.", self.pc);
    //                 } else {
    //                     self.pc += 1;
    //                 }

    //                 // 0x00 inmutable | 0x00 mutable
    //                 let mutable = match self.bytecode[self.pc] {
    //                     0x00 => false,
    //                     0x01 => true,
    //                     _ => {
    //                         panic!("Invalid STORE_VAR instruction at position {}. Needed mutability property.", self.pc);
    //                     }
    //                 };
    //                 self.pc += 1;

    //                 // identifier
    //                 let (identifier_data_type, identifier_bytes) = self.get_value_length();
    //                 if identifier_data_type != DataType::Utf8 {
    //                     panic!("Identifier type should be a string encoded as utf8")
    //                 }

    //                 let identifier_name = String::from_utf8(identifier_bytes)
    //                     .expect("Identifier bytes should be valid UTF-8");

    //                 instructions.push(Instruction::StoreVar {
    //                     identifier: identifier_name,
    //                     mutable,
    //                 });
    //             }
    //             Opcode::Call => {
    //                 // get u32 value. 4 bytes based on the type plus the current
    //                 let value_length = 4;
    //                 if self.pc + value_length >= self.bytecode.len() {
    //                     panic!("Invalid print instruction at position {}", self.pc);
    //                 }

    //                 let value_bytes = &self.bytecode[self.pc + 1..self.pc + 5];
    //                 let number_of_args = u32::from_le_bytes(
    //                     value_bytes.try_into().expect("Provided value is incorrect"),
    //                 );
    //                 instructions.push(Instruction::Call { number_of_args });
    //                 self.pc += 4;
    //             }
    //             _ => {}
    //         };

    //         self.pc += 1;
    //     }

    //     instructions
    // }

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
                if let Value::RawValue(RawValue::U32(val)) = string_length {
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
