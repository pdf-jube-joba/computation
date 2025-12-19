use utils::number::Number;

// Number is "natural number", there is no limit
pub enum Instruction {
    LoadImm { rd: u8, imm: Number },
    Load { rd: u8, addr: Number },
    Store { rs: u8, addr: Number },
    Mov { rd: u8, rs: u8 },
    Add { rd: u8, rs: u8 },
    Sub { rd: u8, rs: u8 },
    JmpReg { rd: u8 },
    JmpRel { imm: Number },
    JltRel { rd: u8, rs: u8, imm: Number },
    Halt,
    Nop,
}

pub struct Environment {
    pub memory: Vec<Number>,
    pub registers: [Number; 8],
    pub pc: Number,
}

pub fn decode_instruction(n: Number) -> Instruction {
    // Dummy implementation for illustration purposes
    // let op: u8 = n.as_usize() as u8;
    Instruction::Nop
}
