# self

## opcode
`self` opcodes can have different meaning in different environments, the important thing is the base level. this is the main level where you're not lookaheading bytes. at this level these are the opcodes:

### instructions
- `0x00`: zero
- `0x01`: load_const
- `0x02`: print
- `0x03`: add

### data types
  - `0x00`: nothing
  - `0x01`: u32
  - `0x02`: u64
  - `0x03`: i32
  - `0x04`: i64

since the data types cannot be used without a instruction that is assigned to the opcodes can be the same. 

> note: all the bytecode data is stored using the little endian encoding. 
