use utils::number::Number;

pub enum Register {
    R0,
    R1,
    R2,
    R3,
}

fn read_register(n: u8) -> Result<Register, String> {
    match n {
        0x0 => Ok(Register::R0),
        0x1 => Ok(Register::R1),
        0x2 => Ok(Register::R2),
        0x3 => Ok(Register::R3),
        _ => Err(format!("invalid register code: {}", n)),
    }
}

// Number is "natural number", there is no limit
// first byte is opcode (4 bit) + u8 arguments (4 bit)
// following bytes are Number arguments
// 
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

    // decode first 4 bit =>
    match (op & 0b1111_0000) >> 4 {
        0x0 => Ok(Instruction::Halt),
        0x1 => Ok(Instruction::Nop),
        // LoadImm
        0x2 => {
            let rd = read_register(op & 0x0F)?;
            let imm = (n.as_usize() >> 8).into();
            Ok(Instruction::LoadImm { rd, imm })
        }
        _ => Err(format!("invalid opcode: {}", op)),
    }
}
