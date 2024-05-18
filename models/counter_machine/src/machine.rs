use std::{collections::HashMap, fmt::Display};
use utils::number::Number;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RegisterIndex(Number);

#[derive(Debug, Clone, PartialEq)]
pub struct ProgramIndex(Number);

impl From<Number> for ProgramIndex {
    fn from(value: Number) -> Self {
        ProgramIndex(value)
    }
}

impl ProgramIndex {
    pub fn next(&mut self) {
        self.0 += 1;
    }
    pub fn is_eq_number(&self, num: Number) -> bool {
        self.0 == num
    }
}

impl From<Number> for RegisterIndex {
    fn from(value: Number) -> Self {
        RegisterIndex(value)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Operation {
    Inc(RegisterIndex),
    Dec(RegisterIndex),
    Clr(RegisterIndex),
    Copy(RegisterIndex, RegisterIndex),
    Ifz(RegisterIndex, ProgramIndex),
}

impl Display for Operation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Inc(index) => write!(f, "INC register:{}", index.0),
            Self::Dec(index) => write!(f, "DEC register:{}", index.0),
            Self::Clr(index) => write!(f, "CLR register:{}", index.0),
            Self::Copy(index0, index1) => write!(f, "CPY register:{} {}", index0.0, index1.0),
            Self::Ifz(r_index, p_index) => {
                write!(f, "IFZ register:{} program:{}", r_index.0, p_index.0)
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Code(pub Vec<Operation>);

#[derive(Debug, Clone, PartialEq)]
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

#[derive(Debug, Clone, PartialEq)]
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
        self.code.0.len() <= self.program_counter.0.clone().into()
    }
    pub fn step(&mut self) {
        if !self.is_terminate() {
            let pc: usize = self.program_counter.0.clone().into();
            let operation = &self.code.0[pc];
            match operation {
                Operation::Inc(index) => {
                    let num = self.registers.get(index);
                    self.registers.set(index.clone(), num + 1);
                    self.program_counter.next();
                }
                Operation::Dec(index) => {
                    let num = self.registers.get(index);
                    self.registers.set(index.clone(), num - 1);
                    self.program_counter.next();
                }
                Operation::Clr(index) => {
                    self.registers.set(index.clone(), 0.into());
                    self.program_counter.next();
                }
                Operation::Copy(index0, index1) => {
                    let n = self.registers.get(index0);
                    self.registers.set(index1.clone(), n);
                }
                Operation::Ifz(register, index) => {
                    if self.registers.get(register) == 0.into() {
                        self.program_counter = index.clone();
                    } else {
                        self.program_counter.next();
                    }
                }
            }
        }
    }
}
