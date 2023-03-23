use std::{collections::HashMap};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct RegisterIndex(usize);

#[derive(Debug, Clone)]
struct Number(usize);
impl Number {
    fn inc(self) -> Self {
        Number(self.0 + 1)
    }
    fn dec(self) -> Self {
        if self.0 != 0 {
            Number(self.0 - 1)
        } else {
            Number(0)
        }
    }
    fn clr() -> Self {
        Number(0)
    }
}

#[derive(Debug, Clone)]
struct ProgramIndex(usize);
impl ProgramIndex {
    fn next(&mut self) {
        self.0 += 1;
    }
    fn goto(&mut self, index: ProgramIndex) {
        todo!()
    }
}

enum Operation {
    Inc(RegisterIndex),
    Dec(RegisterIndex),
    Clr(RegisterIndex),
    Ifz(RegisterIndex, ProgramIndex),
}

struct Code(Vec<Operation>);

struct Registers(HashMap<RegisterIndex, Number>);
impl Registers {
    fn get(&self, index: &RegisterIndex) -> Number {
        match self.0.get(index) {
            Some(num) => num.clone(),
            None => Number(0)
        }
    }
    fn set(&mut self, index: RegisterIndex, num: Number) {
        match self.0.get_mut(&index) {
            Some(target) => {
                *target = num;
            },
            None => {
                self.0.insert(index, num);
            }
        }
    }
}

struct CounterMachine {
    code: Code,
    program_counter: ProgramIndex,
    registers: Registers,
}

impl CounterMachine {
    fn is_terminate(&self) -> bool {
        self.code.0.len() <= self.program_counter.0 
    }
    fn step(&mut self) {
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
                _ => todo!()
            }
        }
    }
}