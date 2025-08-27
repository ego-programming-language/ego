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
    vm::Vm,
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
            Opcode::StructDec => {
                // skip StructDec opcode
                t.pc += 1;

                // identifier
                let (identifier_data_type, identifier_bytes) = t.get_value_length();
                if identifier_data_type != DataType::Utf8 {
                    // TODO: use self-vm errors
                    panic!("Identifier type should be a string encoded as utf8")
                }

                // TODO: use self-vm errors
                let identifier_name = String::from_utf8(identifier_bytes)
                    .expect("Identifier bytes should be valid UTF-8");

                // read fields number
                t.pc += 1;
                let fields_num = Vm::read_offset(&bytecode[t.pc..t.pc + 4]);
                t.pc += 4;

                // struct fields [raw_string][type][raw_string][type]
                //               (x)B        1B    (x)B        1B
                let mut counter = 0;
                let mut fields = vec![];
                while counter < fields_num {
                    // field
                    let (field_data_type, field_bytes) = t.get_value_length();
                    if field_data_type != DataType::Utf8 {
                        // TODO: use self-vm errors
                        panic!("Identifier type should be a string encoded as utf8")
                    }
                    let field_name =
                        String::from_utf8(field_bytes).expect("Field bytes should be valid UTF-8"); // TODO: use self-vm errors
                    t.pc += 1;

                    // annotation
                    let annotation = DataType::to_opcode(bytecode[t.pc]);
                    t.pc += 1;

                    fields.push(format!("{}: {}", field_name, annotation.as_str()));
                    counter += 1;
                }

                t.pc += 1;

                (
                    Instruction::StructDec {
                        identifier: identifier_name.to_string(),
                        fields,
                    },
                    pc.abs_diff(t.pc),
                )
            }
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
                let _ = u32::from_le_bytes(
                    value_bytes.try_into().expect("Provided value is incorrect"),
                );
                t.pc += 4;

                // identifier
                t.pc += 1;
                let (_, _) = t.get_value_length();

                (Instruction::Call, pc.abs_diff(t.pc))
            }
            Opcode::FFI_Call => {
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
                (Instruction::FFI_Call { number_of_args }, pc.abs_diff(t.pc))
            }
            Opcode::GetProperty => (Instruction::GetProperty, 1),
            Opcode::Unknown => (Instruction::Unknown, 1),
            _ => (Instruction::Unknown, 1),
        }
    }

    pub fn get_instruction_info(instruction: &Instruction) -> String {
        match instruction {
            Instruction::LoadConst {
                data_type,
                value: _,
            } => data_type.as_str().to_string(),
            Instruction::LoadVar {
                data_type,
                identifier: _,
            } => data_type.as_str().to_string(),
            Instruction::StoreVar {
                identifier,
                mutable: _,
            } => identifier.to_string(),
            Instruction::Print { number_of_args } => number_of_args.to_string(),
            Instruction::Println { number_of_args } => number_of_args.to_string(),
            Instruction::FFI_Call { number_of_args } => number_of_args.to_string(),
            Instruction::FuncDec { identifier } => identifier.to_string(),
            Instruction::StructDec { identifier, fields } => {
                let mut mem = identifier.to_string();
                for field in fields {
                    mem += format!("\n * {field}").as_str()
                }
                mem
            }
            _ => "".to_string(),
        }
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
