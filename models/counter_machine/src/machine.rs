use std::{collections::HashMap, fmt::Display};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RegisterIndex(usize);

#[derive(Debug, Clone)]
pub struct Number(usize);

impl Number {
    pub fn inc(self) -> Self {
        Number(self.0 + 1)
    }
    pub fn dec(self) -> Self {
        if self.0 != 0 {
            Number(self.0 - 1)
        } else {
            Number(0)
        }
    }
    pub fn clr() -> Self {
        Number(0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ProgramIndex(usize);

impl ProgramIndex {
    pub fn next(&mut self) {
        self.0 += 1;
    }
    pub fn goto(&mut self, _index: ProgramIndex) {
        todo!()
    }
}

impl From<usize> for ProgramIndex {
    fn from(value: usize) -> Self {
        ProgramIndex(value)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Operation {
    Inc(RegisterIndex),
    Dec(RegisterIndex),
    Clr(RegisterIndex),
    Ifz(RegisterIndex, ProgramIndex),
}

impl Display for Operation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Inc(index) => write!(f, "INC register:{}", index.0),
            Self::Dec(index) => write!(f, "DEC register:{}", index.0),
            Self::Clr(index) => write!(f, "CLR register:{}", index.0),
            Self::Ifz(r_index, p_index) => {
                write!(f, "IFZ register:{} program:{}", r_index.0, p_index.0)
            }
            _ => todo!(),
        }
    }
}

pub struct Code(Vec<Operation>);

pub struct Registers(HashMap<RegisterIndex, Number>);

impl Registers {
    pub fn get(&self, index: &RegisterIndex) -> Number {
        match self.0.get(index) {
            Some(num) => num.clone(),
            None => Number(0),
        }
    }
    pub fn set(&mut self, index: RegisterIndex, num: Number) {
        match self.0.get_mut(&index) {
            Some(target) => {
                *target = num;
            }
            None => {
                self.0.insert(index, num);
            }
        }
    }
}

pub struct CounterMachine {
    pub code: Code,
    pub program_counter: ProgramIndex,
    pub registers: Registers,
}

impl CounterMachine {
    pub fn code_as_vec(&self) -> Vec<Operation> {
        self.code.0.clone()
    }
    pub fn is_terminate(&self) -> bool {
        self.code.0.len() <= self.program_counter.0
    }
    pub fn step(&mut self) {
        if !self.is_terminate() {
            let operation = &self.code.0[self.program_counter.0];
            match operation {
                Operation::Inc(index) => {
                    let num = self.registers.get(index);
                    self.registers.set(index.clone(), num.inc());
                    self.program_counter.next();
                }
                Operation::Dec(index) => {
                    let num = self.registers.get(index);
                    self.registers.set(index.clone(), num.dec());
                    self.program_counter.next();
                }
                _ => todo!(),
            }
        }
    }
}
