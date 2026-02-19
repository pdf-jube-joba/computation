use std::collections::HashMap;
use std::fmt::Display;

use turing_machine::machine::Sign;
use utils::TextCodec;

use crate::rec_tm_ir::{Block, Function, Program, Stmt};
use crate::rec_to_ir::auxiliary::basic::{move_left_till_x_n_times, move_right_till_x_n_times};
use crate::rec_to_ir::auxiliary::copy;

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
    if v.last()? != &S::X.into() {
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
    let mut registry = Registry::new();
    registry.resolve(&function);

    let main_function = Function {
        name: "main".to_string(),
        blocks: vec![Block {
            label: "main".to_string(),
            body: vec![Stmt::Call {
                name: function.name.clone(),
            }],
        }],
    };

    registry
        .functions
        .insert(main_function.name.clone(), main_function);

    Program {
        alphabet: S::all().into_iter().map(Into::into).collect(),
        functions: registry.functions.into_values().collect(),
    }
}

struct Registry {
    functions: HashMap<String, Function>, // this contains "visited"
    stack: Vec<String>,                   // this is for DFS and contains "visited but not finished"
}

impl Registry {
    fn new() -> Self {
        Self {
            functions: HashMap::new(),
            stack: Vec::new(),
        }
    }

    fn adhoc_insert(&mut self, name: &str) -> bool {
        if let Some(n) = parse_copy_n_name(name) {
            let func = copy::copy_n_times(n);
            self.functions.insert(func.name.clone(), func);
            return true;
        }
        if let Some(n) = parse_copy_to_end_name(name) {
            let func = copy::copy_to_end(n);
            self.functions.insert(func.name.clone(), func);
            return true;
        }
        if let Some(n) = parse_move_right_till_x_name(name) {
            let func = move_right_till_x_n_times(n);
            self.functions.insert(func.name.clone(), func);
            return true;
        }
        if let Some(n) = parse_move_left_till_x_name(name) {
            let func = move_left_till_x_n_times(n);
            self.functions.insert(func.name.clone(), func);
            return true;
        }
        // currently no adhoc functions, but we can add some if needed
        #[allow(clippy::match_single_binding)]
        match name {
            _ => false,
        }
    }

    fn resolve_inner(&mut self) {
        while let Some(name) = self.stack.pop() {
            if !self.functions.contains_key(&name) && !self.adhoc_insert(&name) {
                continue;
            }

            let Some(func) = self.functions.get(&name) else {
                continue;
            };

            let stmts: Vec<&_> = func
                .blocks
                .iter()
                .flat_map(|block| block.body.as_slice())
                .collect();

            for callee in collect_calls(&stmts) {
                if self.functions.contains_key(&callee) || self.stack.contains(&callee) {
                    continue;
                }
                self.stack.push(callee);
            }
        }
    }

    fn resolve(&mut self, function: &Function) {
        self.functions
            .insert(function.name.clone(), function.clone());
        self.stack.push(function.name.clone());
        self.resolve_inner();
    }
}

fn collect_calls(stmts: &[&Stmt]) -> Vec<String> {
    let mut out = Vec::new();
    for stmt in stmts {
        if let Stmt::Call { name, .. } = stmt {
            out.push(name.clone())
        }
    }
    out
}

fn parse_copy_n_name(name: &str) -> Option<usize> {
    parse_trailing_usize(name, "copy_")
}

fn parse_copy_to_end_name(name: &str) -> Option<usize> {
    parse_trailing_usize(name, "copy_to_end_")
}

fn parse_move_right_till_x_name(name: &str) -> Option<usize> {
    parse_trailing_usize(name, "move_right_till_x_")
}

fn parse_move_left_till_x_name(name: &str) -> Option<usize> {
    parse_trailing_usize(name, "move_left_till_x_")
}

fn parse_trailing_usize(name: &str, prefix: &str) -> Option<usize> {
    let n_str = name.strip_prefix(prefix)?;
    if n_str.is_empty() || !n_str.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    n_str.parse().ok()
}
