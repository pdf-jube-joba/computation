use serde::Serialize;
use utils::{Machine, TextCodec, number::Number};

#[derive(Clone, Serialize)]
pub enum Register {
    R0,
    R1,
    R2,
    R3,
}

fn decode_register(n: u8) -> Result<Register, String> {
    match n {
        0x0 => Ok(Register::R0),
        0x1 => Ok(Register::R1),
        0x2 => Ok(Register::R2),
        0x3 => Ok(Register::R3),
        _ => Err(format!("invalid register code: {}", n)),
    }
}

fn encode_register(reg: &Register) -> u8 {
    match reg {
        Register::R0 => 0x0,
        Register::R1 => 0x1,
        Register::R2 => 0x2,
        Register::R3 => 0x3,
    }
}

// Number is "natural number", there is no limit
// first byte is opcode (4 bit) + u8 arguments (4 bit)
// following bytes are Number arguments
#[derive(Clone, Serialize)]
pub enum Instruction {
    // rd <-- imm
    LoadImm {
        rd: Register,
        imm: Number,
    },
    // rd <-- MEM[addr]
    Load {
        rd: Register,
        addr: Number,
    },
    // MEM[addr] <-- rs
    Store {
        rs: Register,
        addr: Number,
    },
    // rd <-- rs
    Mov {
        rd: Register,
        rs: Register,
    },
    // rd <-- rd + rs
    Add {
        rd: Register,
        rs: Register,
    },
    // rd <-- rd - rs (saturating at 0)
    Sub {
        rd: Register,
        rs: Register,
    },
    // pc <-- rs
    JmpReg {
        rd: Register,
    },
    // pc <-- pc + imm
    JmpRel {
        imm: Number,
    },
    // if rd < rs then pc <-- pc + imm
    JltRel {
        rd: Register,
        rs: Register,
        imm: Number,
    },
    Halt,
    Nop,
}

#[derive(Clone, Serialize)]
pub struct Environment {
    pub memory: Vec<Number>,
    pub registers: [Number; 4], // 2^2 registers
    pub pc: Number,
}

pub fn decode_instruction(n: Number) -> Result<Instruction, String> {
    // read first byte
    let op = n.as_u8array()[0];

    let opcode = (op & 0b1111_0000) >> 4;
    let op_reg1 = (op & 0b0000_1100) >> 2;
    let op_reg2 = op & 0b0000_0011;
    let remain: Number = n.as_u8array()[1..].to_vec().into();

    // decode first 4 bit =>
    match opcode {
        0x0 => Ok(Instruction::Halt),
        0x1 => Ok(Instruction::Nop),
        // LoadImm
        0x2 => {
            let rd = decode_register(op_reg1)?;
            let imm = remain;
            Ok(Instruction::LoadImm { rd, imm })
        }
        // Load
        0x3 => {
            let rd = decode_register(op_reg1)?;
            let addr = remain;
            Ok(Instruction::Load { rd, addr })
        }
        // Store
        0x4 => {
            let rs = decode_register(op_reg1)?;
            let addr = remain;
            Ok(Instruction::Store { rs, addr })
        }
        // Mov
        0x5 => {
            let rd = decode_register(op_reg1)?;
            let rs = decode_register(op_reg2)?;
            Ok(Instruction::Mov { rd, rs })
        }
        // Add
        0x6 => {
            let rd = decode_register(op_reg1)?;
            let rs = decode_register(op_reg2)?;
            Ok(Instruction::Add { rd, rs })
        }
        // Sub
        0x7 => {
            let rd = decode_register(op_reg1)?;
            let rs = decode_register(op_reg2)?;
            Ok(Instruction::Sub { rd, rs })
        }
        // JmpReg
        0x8 => {
            let rd = decode_register(op_reg1)?;
            Ok(Instruction::JmpReg { rd })
        }
        // JmpRel
        0x9 => {
            let imm = remain;
            Ok(Instruction::JmpRel { imm })
        }
        // JltRel
        0xA => {
            let rd = decode_register(op_reg1)?;
            let rs = decode_register(op_reg2)?;
            let imm = remain;
            Ok(Instruction::JltRel { rd, rs, imm })
        }
        _ => Err(format!("invalid opcode: {}", op)),
    }
}

pub fn encode_instruction(inst: &Instruction) -> Number {
    let mut bytes = vec![];
    match inst {
        Instruction::LoadImm { rd, imm } => {
            let opcode: u8 = 0x2 << 4;
            let reg_code = encode_register(rd) << 2;
            let first_byte = opcode | reg_code;
            bytes.push(first_byte);
            bytes.extend(imm.as_u8array());
            Number::from_u8array(&bytes)
        }
        Instruction::Load { rd, addr } => todo!(),
        Instruction::Store { rs, addr } => todo!(),
        Instruction::Mov { rd, rs } => todo!(),
        Instruction::Add { rd, rs } => todo!(),
        Instruction::Sub { rd, rs } => todo!(),
        Instruction::JmpReg { rd } => todo!(),
        Instruction::JmpRel { imm } => todo!(),
        Instruction::JltRel { rd, rs, imm } => todo!(),
        Instruction::Halt => todo!(),
        Instruction::Nop => todo!(),
    }
}

#[derive(Clone, Serialize)]
pub struct Code(Vec<Instruction>);

// read and write as hex separated by newlines
impl TextCodec for Code {
    fn parse(text: &str) -> Result<Self, String> {
        let mut v = vec![];
        for line in text.lines() {
            let trimed = line.trim();
            if trimed.is_empty() {
                continue;
            }

            let mut bytes = vec![];
            let chars: Vec<char> = trimed.chars().collect();

            for i in (0..chars.len()).rev().step_by(2) {
                let c = chars[i];
                let c_prev = if i > 0 { chars[i - 1] } else { '0' };
                let byte_str = format!("{}{}", c_prev, c);
                let byte = u8::from_str_radix(&byte_str, 16)
                    .map_err(|e| format!("invalid hex byte '{}': {}", byte_str, e))?;
                bytes.push(byte);
            }

            let n: Number = Number::from_u8array(&bytes);
            let inst = decode_instruction(n)?;
            v.push(inst);
        }
        Ok(Code(v))
    }

    fn write_fmt(&self, f: &mut impl std::fmt::Write) -> std::fmt::Result {
        for inst in &self.0 {
            let n = encode_instruction(inst);
            let bytes = n.as_u8array();
            for &b in bytes.iter().rev() {
                write!(f, "{:02X}", b)?;
            }
            writeln!(f)?;
        }
        Ok(())
    }
}

impl Machine for Environment {
    type Code = Code;

    type AInput = ();

    type SnapShot = Environment;

    type RInput = ();

    type Output = ();

    fn make(code: Self::Code, ainput: Self::AInput) -> Result<Self, String> {
        todo!()
    }

    fn step(&mut self, rinput: Self::RInput) -> Result<Option<Self::Output>, String> {
        todo!()
    }

    fn current(&self) -> Self::SnapShot {
        todo!()
    }
}
