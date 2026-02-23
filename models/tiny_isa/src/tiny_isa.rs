// use serde::Serialize;
// use utils::{Machine, TextCodec, number::Number};

// #[derive(Clone, Copy, Serialize)]
// pub enum Register {
//     R0,
//     R1,
//     R2,
//     R3,
// }

// fn decode_register(n: u8) -> Result<Register, String> {
//     match n {
//         0x0 => Ok(Register::R0),
//         0x1 => Ok(Register::R1),
//         0x2 => Ok(Register::R2),
//         0x3 => Ok(Register::R3),
//         _ => Err(format!("invalid register code: {}", n)),
//     }
// }

// fn encode_register(reg: &Register) -> u8 {
//     match reg {
//         Register::R0 => 0x0,
//         Register::R1 => 0x1,
//         Register::R2 => 0x2,
//         Register::R3 => 0x3,
//     }
// }

// fn which_register(n: Register) -> usize {
//     match n {
//         Register::R0 => 0,
//         Register::R1 => 1,
//         Register::R2 => 2,
//         Register::R3 => 3,
//     }
// }

// // Number is "natural number", there is no limit
// // first byte is opcode (4 bit) + u8 arguments (4 bit)
// // following bytes are Number arguments
// #[derive(Clone, Serialize)]
// pub enum Instruction {
//     Halt,
//     Nop,
//     // rd <-- imm
//     LoadImm {
//         rd: Register,
//         imm: Number,
//     },
//     // rd <-- MEM[addr]
//     Load {
//         rd: Register,
//         addr: Number,
//     },
//     // MEM[addr] <-- rs
//     Store {
//         rs: Register,
//         addr: Number,
//     },
//     // rd <-- rs
//     Mov {
//         rd: Register,
//         rs: Register,
//     },
//     // rd <-- rd + rs
//     Add {
//         rd: Register,
//         rs: Register,
//     },
//     // rd <-- rd - rs (saturating at 0)
//     Sub {
//         rd: Register,
//         rs: Register,
//     },
//     // rd <-- pc
//     ReadPc {
//         rd: Register,
//     },
//     // pc <-- rs
//     JmpReg {
//         rd: Register,
//     },
//     // pc <-- imm
//     JmpImm {
//         imm: Number,
//     },
//     // pc <-- pc + rd
//     JmpRelReg {
//         rd: Register,
//     },
//     // pc <-- pc + imm
//     JmpRelImm {
//         imm: Number,
//     },
//     // if rd < rs then pc <-- pc + imm
//     JltRel {
//         rd: Register,
//         rs: Register,
//         imm: Number,
//     },
//     // if rd < rs then pc <-- pc - imm
//     JLtRelBack {
//         rd: Register,
//         rs: Register,
//         imm: Number,
//     },
// }

// #[derive(Clone, Serialize)]
// pub struct Environment {
//     pub code_len: Number,
//     pub memory: Vec<Number>,
//     pub registers: [Number; 4], // 2^2 registers
//     pub pc: Number,
// }

// pub fn decode_instruction(n: Number) -> Result<Instruction, String> {
//     // read first byte
//     let op = n.as_u8array()[0];

//     let opcode = (op & 0b1111_0000) >> 4;
//     let op_reg1 = (op & 0b0000_1100) >> 2;
//     let op_reg2 = op & 0b0000_0011;
//     let remain: Number = n.as_u8array()[1..].to_vec().into();

//     // decode first 4 bit =>
//     match opcode {
//         0x0 => Ok(Instruction::Nop),
//         0x1 => Ok(Instruction::Halt),
//         // LoadImm
//         0x2 => {
//             let rd = decode_register(op_reg1)?;
//             let imm = remain;
//             Ok(Instruction::LoadImm { rd, imm })
//         }
//         // Load
//         0x3 => {
//             let rd = decode_register(op_reg1)?;
//             let addr = remain;
//             Ok(Instruction::Load { rd, addr })
//         }
//         // Store
//         0x4 => {
//             let rs = decode_register(op_reg1)?;
//             let addr = remain;
//             Ok(Instruction::Store { rs, addr })
//         }
//         // Mov
//         0x5 => {
//             let rd = decode_register(op_reg1)?;
//             let rs = decode_register(op_reg2)?;
//             Ok(Instruction::Mov { rd, rs })
//         }
//         // Add
//         0x6 => {
//             let rd = decode_register(op_reg1)?;
//             let rs = decode_register(op_reg2)?;
//             Ok(Instruction::Add { rd, rs })
//         }
//         // Sub
//         0x7 => {
//             let rd = decode_register(op_reg1)?;
//             let rs = decode_register(op_reg2)?;
//             Ok(Instruction::Sub { rd, rs })
//         }
//         // ReadPc
//         0x8 => {
//             let rd = decode_register(op_reg1)?;
//             Ok(Instruction::ReadPc { rd })
//         }
//         // JmpReg
//         0x9 => {
//             let rd = decode_register(op_reg1)?;
//             Ok(Instruction::JmpReg { rd })
//         }
//         // JmpImm
//         0xA => {
//             let imm = remain;
//             Ok(Instruction::JmpImm { imm })
//         }
//         // JmpRelReg
//         0xB => {
//             let rd = decode_register(op_reg1)?;
//             Ok(Instruction::JmpRelReg { rd })
//         }
//         // JmpRelImm
//         0xC => {
//             let imm = remain;
//             Ok(Instruction::JmpRelImm { imm })
//         }
//         // JltRel
//         0xD => {
//             let rd = decode_register(op_reg1)?;
//             let rs = decode_register(op_reg2)?;
//             let imm = remain;
//             Ok(Instruction::JltRel { rd, rs, imm })
//         }
//         // JLtRelback
//         0xE => {
//             let rd = decode_register(op_reg1)?;
//             let rs = decode_register(op_reg2)?;
//             let imm = remain;
//             Ok(Instruction::JLtRelBack { rd, rs, imm })
//         }
//         _ => Err(format!("invalid opcode: {}", op)),
//     }
// }

// pub fn encode_instruction(inst: &Instruction) -> Number {
//     let mut bytes = vec![];
//     match inst {
//         Instruction::Nop => {
//             let opcode: u8 = 0x0 << 4;
//             let first_byte = opcode;
//             bytes.push(first_byte);
//             Number::from_u8array(&bytes)
//         }
//         Instruction::Halt => {
//             let opcode: u8 = 0x1 << 4;
//             let first_byte = opcode;
//             bytes.push(first_byte);
//             Number::from_u8array(&bytes)
//         }
//         Instruction::LoadImm { rd, imm } => {
//             let opcode: u8 = 0x2 << 4;
//             let reg_code = encode_register(rd) << 2;
//             let first_byte = opcode | reg_code;
//             bytes.push(first_byte);
//             bytes.extend(imm.as_u8array());
//             Number::from_u8array(&bytes)
//         }
//         Instruction::Load { rd, addr } => {
//             let opcode: u8 = 0x3 << 4;
//             let reg_code = encode_register(rd) << 2;
//             let first_byte = opcode | reg_code;
//             bytes.push(first_byte);
//             bytes.extend(addr.as_u8array());
//             Number::from_u8array(&bytes)
//         }
//         Instruction::Store { rs, addr } => {
//             let opcode: u8 = 0x4 << 4;
//             let reg_code = encode_register(rs) << 2;
//             let first_byte = opcode | reg_code;
//             bytes.push(first_byte);
//             bytes.extend(addr.as_u8array());
//             Number::from_u8array(&bytes)
//         }
//         Instruction::Mov { rd, rs } => {
//             let opcode: u8 = 0x5 << 4;
//             let reg1_code = encode_register(rd) << 2;
//             let reg2_code = encode_register(rs);
//             let first_byte = opcode | reg1_code | reg2_code;
//             bytes.push(first_byte);
//             Number::from_u8array(&bytes)
//         }
//         Instruction::Add { rd, rs } => {
//             let opcode: u8 = 0x6 << 4;
//             let reg1_code = encode_register(rd) << 2;
//             let reg2_code = encode_register(rs);
//             let first_byte = opcode | reg1_code | reg2_code;
//             bytes.push(first_byte);
//             Number::from_u8array(&bytes)
//         }
//         Instruction::Sub { rd, rs } => {
//             let opcode: u8 = 0x7 << 4;
//             let reg1_code = encode_register(rd) << 2;
//             let reg2_code = encode_register(rs);
//             let first_byte = opcode | reg1_code | reg2_code;
//             bytes.push(first_byte);
//             Number::from_u8array(&bytes)
//         }
//         Instruction::ReadPc { rd } => {
//             let opcode: u8 = 0x8 << 4;
//             let reg_code = encode_register(rd) << 2;
//             let first_byte = opcode | reg_code;
//             bytes.push(first_byte);
//             Number::from_u8array(&bytes)
//         }
//         Instruction::JmpReg { rd } => {
//             let opcode: u8 = 0x9 << 4;
//             let reg_code = encode_register(rd) << 2;
//             let first_byte = opcode | reg_code;
//             bytes.push(first_byte);
//             Number::from_u8array(&bytes)
//         }
//         Instruction::JmpImm { imm } => {
//             let opcode: u8 = 0xA << 4;
//             let first_byte = opcode;
//             bytes.push(first_byte);
//             bytes.extend(imm.as_u8array());
//             Number::from_u8array(&bytes)
//         }
//         Instruction::JmpRelReg { rd } => {
//             let opcode: u8 = 0xB << 4;
//             let reg_code = encode_register(rd) << 2;
//             let first_byte = opcode | reg_code;
//             bytes.push(first_byte);
//             Number::from_u8array(&bytes)
//         }
//         Instruction::JmpRelImm { imm } => {
//             let opcode: u8 = 0xC << 4;
//             let first_byte = opcode;
//             bytes.push(first_byte);
//             bytes.extend(imm.as_u8array());
//             Number::from_u8array(&bytes)
//         }
//         Instruction::JltRel { rd, rs, imm } => {
//             let opcode: u8 = 0xD << 4;
//             let reg1_code = encode_register(rd) << 2;
//             let reg2_code = encode_register(rs);
//             let first_byte = opcode | reg1_code | reg2_code;
//             bytes.push(first_byte);
//             bytes.extend(imm.as_u8array());
//             Number::from_u8array(&bytes)
//         }
//         Instruction::JLtRelBack { rd, rs, imm } => {
//             let opcode: u8 = 0xE << 4;
//             let reg1_code = encode_register(rd) << 2;
//             let reg2_code = encode_register(rs);
//             let first_byte = opcode | reg1_code | reg2_code;
//             bytes.push(first_byte);
//             bytes.extend(imm.as_u8array());
//             Number::from_u8array(&bytes)
//         }
//     }
// }

// #[derive(Clone, Serialize)]
// pub struct Code(Vec<Instruction>);

// // read and write as hex separated by newlines
// impl TextCodec for Code {
//     fn parse(text: &str) -> Result<Self, String> {
//         let mut v = vec![];
//         for line in text.lines() {
//             let trimed = line.trim();
//             if trimed.is_empty() {
//                 continue;
//             }

//             let mut bytes = vec![];
//             let chars: Vec<char> = trimed.chars().collect();

//             for i in (0..chars.len()).rev().step_by(2) {
//                 let c = chars[i];
//                 let c_prev = if i > 0 { chars[i - 1] } else { '0' };
//                 let byte_str = format!("{}{}", c_prev, c);
//                 let byte = u8::from_str_radix(&byte_str, 16)
//                     .map_err(|e| format!("invalid hex byte '{}': {}", byte_str, e))?;
//                 bytes.push(byte);
//             }

//             let n: Number = Number::from_u8array(&bytes);
//             let inst = decode_instruction(n)?;
//             v.push(inst);
//         }
//         Ok(Code(v))
//     }

//     fn write_fmt(&self, f: &mut impl std::fmt::Write) -> std::fmt::Result {
//         for inst in &self.0 {
//             let n = encode_instruction(inst);
//             let bytes = n.as_u8array();
//             for &b in bytes.iter().rev() {
//                 write!(f, "{:02X}", b)?;
//             }
//             writeln!(f)?;
//         }
//         Ok(())
//     }
// }

// impl Machine for Environment {
//     type Code = Code;
//     type AInput = Vec<Number>;
//     type SnapShot = Environment;
//     type RInput = ();
//     type Output = Vec<Number>;

//     fn make(code: Self::Code, ainput: Self::AInput) -> Result<Self, String> {
//         let mut array: Vec<Number> = vec![];
//         array.extend(code.0.iter().map(encode_instruction));
//         array.extend(ainput);
//         Ok(Environment {
//             code_len: code.0.len().into(),
//             memory: array,
//             registers: std::array::from_fn(|_| 0.into()),
//             pc: 0.into(),
//         })
//     }

//     fn step(&mut self, _rinput: Self::RInput) -> Result<Option<Self::Output>, String> {
//         // fetch instruction
//         let pc_usize = self.pc.as_usize().map_err(|e| e.to_string())?;
//         if pc_usize >= self.memory.len() {
//             return Err("PC out of bounds".to_string());
//         }
//         let inst_n = self.memory[pc_usize].clone();
//         let inst = decode_instruction(inst_n)?;

//         // execute instruction
//         match inst {
//             Instruction::Nop => {
//                 // Do nothing
//                 self.pc += 1;
//             }
//             Instruction::Halt => {
//                 // Halt the machine and return the current memory as output
//                 return Ok(Some(self.memory.clone()));
//             }
//             Instruction::LoadImm { rd, imm } => {
//                 // Load immediate value into register
//                 self.registers[which_register(rd)] = imm;
//                 self.pc += 1;
//             }
//             Instruction::Load { rd, addr } => {
//                 // Load value from memory into register
//                 let addr_usize = addr.as_usize().map_err(|e| e.to_string())?;
//                 if addr_usize >= self.memory.len() {
//                     return Err("Memory address out of bounds".to_string());
//                 }
//                 self.registers[which_register(rd)] = self.memory[addr_usize].clone();
//                 self.pc += 1;
//             }
//             Instruction::Store { rs, addr } => {
//                 // Store value from register into memory
//                 let addr_usize = addr.as_usize().map_err(|e| e.to_string())?;
//                 if addr_usize >= self.memory.len() {
//                     return Err("Memory address out of bounds".to_string());
//                 }
//                 self.memory[addr_usize] = self.registers[which_register(rs)].clone();
//                 self.pc += 1;
//             }
//             Instruction::Mov { rd, rs } => {
//                 // Move value from one register to another
//                 self.registers[which_register(rd)] = self.registers[which_register(rs)].clone();
//                 self.pc += 1;
//             }
//             Instruction::Add { rd, rs } => {
//                 // Add values of two registers and store in destination register
//                 self.registers[which_register(rd)] += self.registers[which_register(rs)].clone();
//                 self.pc += 1;
//             }
//             Instruction::Sub { rd, rs } => {
//                 // Subtract value of one register from another with saturation at 0
//                 let result = self.registers[which_register(rd)].clone()
//                     - (self.registers[which_register(rs)].clone());
//                 self.registers[which_register(rd)] = result;
//                 self.pc += 1;
//             }
//             Instruction::ReadPc { rd } => {
//                 // Read the program counter into a register
//                 self.registers[which_register(rd)] = self.pc.clone();
//                 self.pc += 1;
//             }
//             Instruction::JmpReg { rd } => {
//                 // Jump to the address in the specified register
//                 self.pc = self.registers[which_register(rd)].clone();
//             }
//             Instruction::JmpImm { imm } => {
//                 // Jump to the immediate address
//                 self.pc = imm;
//             }
//             Instruction::JmpRelReg { rd } => {
//                 // Jump relative to the current PC by the value in the specified register
//                 self.pc += self.registers[which_register(rd)].clone();
//             }
//             Instruction::JmpRelImm { imm } => {
//                 // Jump relative to the current PC by the immediate value
//                 self.pc += imm;
//             }
//             Instruction::JltRel { rd, rs, imm } => {
//                 // If the value in one register is less than another, jump relative by the immediate value
//                 if self.registers[which_register(rd)] < self.registers[which_register(rs)] {
//                     self.pc += imm;
//                 } else {
//                     self.pc += 1;
//                 }
//             }
//             Instruction::JLtRelBack { rd, rs, imm } => {
//                 // If the value in one register is less than another, jump back relative by the immediate value
//                 if self.registers[which_register(rd)] < self.registers[which_register(rs)] {
//                     self.pc -= imm;
//                 } else {
//                     self.pc += 1;
//                 }
//             }
//         }

//         Ok(None)
//     }

//     fn current(&self) -> Self::SnapShot {
//         self.clone()
//     }
// }
