use std::ops::{Index, IndexMut};

use utils::{bool::Bool, number::Number};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Byte(Bool, Bool, Bool, Bool, Bool, Bool, Bool, Bool);

impl Default for Byte {
    fn default() -> Self {
        Byte(
            Bool::F,
            Bool::F,
            Bool::F,
            Bool::F,
            Bool::F,
            Bool::F,
            Bool::F,
            Bool::F,
        )
    }
}

impl Index<usize> for Byte {
    type Output = Bool;
    fn index(&self, index: usize) -> &Self::Output {
        match index {
            0 => &self.0,
            1 => &self.1,
            2 => &self.2,
            3 => &self.3,
            4 => &self.4,
            5 => &self.5,
            6 => &self.6,
            7 => &self.7,
            _ => panic!("out of index: {index}"),
        }
    }
}

impl From<u8> for Byte {
    fn from(value: u8) -> Self {
        let idx = |i: usize| -> Bool {
            if value & (1 << i) == 1 {
                Bool::T
            } else {
                Bool::F
            }
        };
        Byte(
            idx(0),
            idx(1),
            idx(2),
            idx(3),
            idx(4),
            idx(5),
            idx(6),
            idx(7),
        )
    }
}

pub enum Register {
    R0,
    R1,
    R2,
    R3,
}

pub struct Assumrator(Byte);

pub struct ProgramCounter(Number);

pub enum Operation {
    Add(Register, Register),
    Not(Register),
    And(Register),
    Cpy(Register, Register),
    Bck(Register),
    Ldm(Register, Number),
    Srm(Number),
    Jmp(Number),
    Brz(Number),
    Brc(Number),
    Nop(),
    Hlt(),
}

pub struct Memory {
    default: Byte, //
    mem: Vec<Byte>,
}

impl Index<Number> for Memory {
    type Output = Byte;
    fn index(&self, index: Number) -> &Self::Output {
        let u: usize = index.into();
        if self.mem.len() < u {
            &self.mem[u]
        } else {
            &self.default
        }
    }
}

impl IndexMut<Number> for Memory {
    fn index_mut(&mut self, index: Number) -> &mut Self::Output {
        let u: usize = index.into();
        if self.mem.len() < u {
            &mut self.mem[u]
        } else {
            let l = u - self.mem.len() + 1;
            self.mem.append(&mut vec![Byte::default(); l]);
            &mut self.mem[u]
        }
    }
}

// こう書くとエラーになる。解決できそうになかった。
// impl Index<Number> for Memory {
//     type Output = Byte;
//     fn index(&self, index: Number) -> &Self::Output {
//         let u: usize = index.into();
//         if self.mem.len() < u {
//             &self.mem[u]
//         } else {
//             &Byte::default() // temporary value への reference を返すのでうまく動かない。
//         }
//     }
// }
