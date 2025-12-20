use serde::Serialize;
use utils::{Machine, alphabet::Alphabet, number::Number};

#[derive(Clone, Serialize)]
pub enum Register {
    R0,
    R1,
    R2,
    R3,
}

#[derive(Clone, Serialize)]
pub struct Label(pub(crate) Alphabet);

#[derive(Clone, Serialize)]
pub enum Value {
    Imm(Number),
    Label(Label),
}

#[derive(Clone, Serialize)]
pub enum ImR {
    Nop,
    Halt,
    LoadImm {
        dest: Register,
        value: Value,
    },
    Load {
        dest: Register,
        addr: Value,
    },
    Store {
        src: Register,
        value: Value,
    },
    Mov {
        dest: Register,
        src: Register,
    },
    Add {
        dest: Register,
        src: Register,
    },
    Sub {
        dest: Register,
        src: Register,
    },
    ReadPc {
        dest: Register,
    },
    JmpReg {
        target: Register,
    },
    JmpImm {
        value: Value,
    },
    JmpRelReg {
        rd: Register,
    },
    JmpRelImm {
        imm: Number,
    },
    JltRel {
        rd: Register,
        rs: Register,
        imm: Number,
    },
}

#[derive(Clone, Serialize)]
pub struct Dat(Vec<(Label, Number)>);

#[derive(Clone, Serialize)]
pub struct Code(Vec<(Label, Vec<ImR>)>);

#[derive(Clone, Serialize)]
pub struct Asm {
    pub dat: Dat,
    pub code: Code,
}

// we need to implement Machine for asm
// i.e. we need to define semantics for this assembly language (independently from the VM)
