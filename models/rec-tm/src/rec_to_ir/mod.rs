use std::collections::HashSet;
use std::fmt::Display;
use std::rc::Rc;

use turing_machine::machine::Sign;
use utils::{Compiler, Machine, TextCodec};

use crate::rec_tm_ir::{Block, Function, Program, Stmt};

pub mod auxiliary;
pub mod compile;

#[cfg(test)]
mod tests;

#[derive(Debug, Clone)]
pub enum S {
    B, // '-' blank
    L, // 'l' flag
    X, // 'x' partition
}

impl S {
    pub fn blank() -> Self {
        S::B
    }
    pub fn all() -> Vec<Self> {
        vec![S::B, S::L, S::X]
    }
}

impl From<S> for Sign {
    fn from(s: S) -> Self {
        match s {
            S::B => Sign::blank(), // "-" blank
            S::L => Sign::parse("l").unwrap(),
            S::X => Sign::parse("x").unwrap(),
        }
    }
}

impl Display for S {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s: Sign = self.clone().into();
        TextCodec::write_fmt(&s, f)
    }
}

use turing_machine::machine::Tape;
use utils::number::Number;

fn num_sings(num: Number) -> Vec<Sign> {
    (0..num.as_usize().unwrap()).map(|_| S::L.into()).collect()
}

pub fn write(tuple: Vec<Number>) -> Tape {
    let mut signs: Vec<Sign> = vec![];
    signs.push(S::X.into());

    for num in tuple {
        signs.push(Sign::blank());
        signs.extend_from_slice(&num_sings(num));
    }

    signs.push(S::X.into());

    Tape::from_vec(signs, 0).unwrap()
}

pub fn write_usize(tuple: Vec<usize>) -> Tape {
    let number_tuple: Vec<Number> = tuple.into_iter().map(|x| x.into()).collect();
    write(number_tuple)
}

fn read_one(signs: Vec<Sign>) -> Option<Vec<Number>> {
    let v = signs
        .split(|char| *char == Sign::blank())
        .map(|vec| vec.len().into())
        .skip(1);
    Some(v.collect::<Vec<_>>())
}

pub fn read_right_one(tape: &Tape) -> Option<Vec<Number>> {
    let (v, p) = tape.into_vec();
    if v[p] != S::X.into() {
        return None;
    }

    let iter = v
        .into_iter()
        // skip until blank `-` after the first x (x is the head position p)
        .skip(p + 1)
        .take_while(|char| *char != S::X.into());
    read_one(iter.collect())
}

pub fn read_right_one_usize(tape: &Tape) -> Option<Vec<usize>> {
    read_right_one(tape).map(|vec| vec.into_iter().map(|x| x.as_usize().unwrap()).collect())
}

// Naming convention: function names are unique and "main" is reserved.
pub(crate) fn wrap_function(function: Function) -> Program {
    let function = Rc::new(function);

    let main_function = Rc::new(Function {
        name: "main".to_string(),
        blocks: vec![Block {
            label: "main".to_string(),
            body: vec![Stmt::Call {
                func: function.clone(),
            }],
        }],
    });

    let mut functions = Vec::new();
    let mut seen = HashSet::new();
    collect_functions(&main_function, &mut seen, &mut functions);
    Program {
        alphabet: S::all().into_iter().map(Into::into).collect(),
        functions,
    }
}

fn collect_functions(
    func: &Rc<Function>,
    seen: &mut HashSet<*const Function>,
    out: &mut Vec<Rc<Function>>,
) {
    let ptr = Rc::as_ptr(func);
    if !seen.insert(ptr) {
        return;
    }
    out.push(func.clone());
    for block in &func.blocks {
        for stmt in &block.body {
            if let Stmt::Call { func: callee } = stmt {
                collect_functions(callee, seen, out);
            }
        }
    }
}

pub struct RecToRecTmIrCompiler;

impl Compiler for RecToRecTmIrCompiler {
    type Source = recursive_function::machine::Program;
    type Target = crate::rec_tm_ir::RecTmIrMachine;

    fn compile(
        source: <<Self as Compiler>::Source as Machine>::Code,
    ) -> Result<<<Self as Compiler>::Target as Machine>::Code, String> {
        Ok(compile::compile_to_program(&source))
    }

    fn encode_ainput(
        ainput: <<Self as Compiler>::Source as Machine>::AInput,
    ) -> Result<<<Self as Compiler>::Target as Machine>::AInput, String> {
        Ok(write(ainput))
    }

    fn encode_rinput(
        rinput: <<Self as Compiler>::Source as Machine>::RInput,
    ) -> Result<<<Self as Compiler>::Target as Machine>::RInput, String> {
        let _: () = rinput;
        Ok(())
    }

    fn decode_routput(
        output: <<Self as Compiler>::Target as Machine>::ROutput,
    ) -> Result<<<Self as Compiler>::Source as Machine>::ROutput, String> {
        let _: () = output;
        Ok(())
    }

    fn decode_foutput(
        output: <<Self as Compiler>::Target as Machine>::FOutput,
    ) -> Result<<<Self as Compiler>::Source as Machine>::FOutput, String> {
        let tuple = read_right_one(&output)
            .ok_or_else(|| "failed to decode tape as recursive_function output".to_string())?;
        match tuple.as_slice() {
            [value] => Ok(value.clone()),
            _ => Err(format!(
                "expected a single output value, but got tuple of length {}",
                tuple.len()
            )),
        }
    }
}
