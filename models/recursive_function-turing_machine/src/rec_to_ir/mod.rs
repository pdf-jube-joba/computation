use std::fmt::Display;

use turing_machine::machine::Sign;
use utils::TextCodec;

use crate::rec_tm_ir::{Function, Program, Stmt};
use crate::rec_to_ir::auxiliary::basic::{move_left_till_x, move_right_till_x};

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

    let iter = v.into_iter().skip(p);
    read_one(iter.collect())
}

pub fn read_right_one_usize(tape: &Tape) -> Option<Vec<usize>> {
    read_right_one(tape).map(|vec| vec.into_iter().map(|x| x.as_usize().unwrap()).collect())
}

fn wrap_function(function: Function) -> Program {
    let mut functions = Vec::new();

    let aux_functions = vec![move_right_till_x(), move_left_till_x()];
    for aux in aux_functions {
        if aux.name != function.name {
            functions.push(aux);
        }
    }

    let main_function = Function {
        name: "main".to_string(),
        params: vec![],
        body: vec![Stmt::Call {
            name: function.name.clone(),
            args: vec![],
        }],
    };

    Program {
        alphabet: vec![S::B.into(), S::L.into(), S::X.into()],
        functions: {
            functions.push(function);
            functions.push(main_function);
            functions
        },
    }
}

/*
use turing_machine::manipulation::graph_compose::{GraphOfBuilder, builder_composition};
use turing_machine::{machine::*, manipulation::builder::TuringMachineBuilder};
#[cfg(test)]
use utils::TextCodec;
use utils::parse::ParseTextCodec;

pub mod auxiliary;
pub mod compile;
pub mod symbols;

#[cfg(test)]
fn builder_test(
    builder: &mut TuringMachineBuilder,
    step: usize,
    tests: Vec<(Result<Tape, String>, Result<Tape, String>)>,
) {
    eprintln!("test start");
    for (input, expect) in tests {
        let input = input.unwrap();
        let expect = expect.unwrap();
        eprintln!("input: {}", input.print());
        let mut machine = builder.build(input).unwrap();
        eprintln!(
            "{:?}\n    {}",
            machine.now_state(),
            machine.now_tape().print()
        );
        for _ in 0..step {
            let _ = machine.step(1);
            eprintln!(
                "__{:?}\n    {}",
                machine.now_state(),
                machine.now_tape().print()
            );
            if machine.is_terminate() {
                break;
            }
        }
        assert!(machine.is_accepted());
        assert!(machine.now_tape().eq(&expect));
    }
}

#[cfg(test)]
fn builder_test_predicate(
    builder: &mut TuringMachineBuilder,
    step: usize,
    tests: Vec<(Result<Tape, String>, State)>,
) {
    eprintln!("test start");
    for (input, result) in tests {
        let input = input.unwrap();
        let mut machine = builder.build(input).unwrap();
        eprintln!(
            "{:?}\n    {}",
            machine.now_state(),
            machine.now_tape().print()
        );
        for _ in 0..step {
            let _ = machine.step(1);
            eprintln!(
                "__{:?}\n    {}",
                machine.now_state(),
                machine.now_tape().print()
            );
            if machine.is_terminate() {
                break;
            }
        }
        assert!(machine.is_accepted());
        assert_eq!(*machine.now_state(), result);
    }
}

pub(crate) fn chain_builders(
    name: impl Into<String>,
    builders: Vec<TuringMachineBuilder>,
) -> TuringMachineBuilder {
    let len = builders.len();
    let graph = GraphOfBuilder {
        name: name.into(),
        init_state: "start".parse_tc().unwrap(),
        assign_vertex_to_builder: builders,
        assign_edge_to_state: series_edge_end_only(len.saturating_sub(1)),
        acceptable: accept_end_only(len.saturating_sub(1)),
    };
    builder_composition(graph).unwrap()
}

// start state: "start"
// accept state: "end"
struct Builder<'a> {
    name: String,
    code: Vec<&'a str>,
}

impl<'a> From<Builder<'a>> for TuringMachineBuilder {
    fn from(builder: Builder) -> Self {
        let mut tm_builder =
            TuringMachineBuilder::new(&builder.name, "start".parse_tc().unwrap()).unwrap();
        tm_builder.accepted_state = vec!["end".parse_tc().unwrap()];
        for entry in builder.code {
            let entry = turing_machine::parse::parse_one_code_entry(entry).unwrap();
            tm_builder.code.push(entry.into());
        }
        tm_builder
    }
}

// 最後の edge の番号 = n
fn accept_end_only(n: usize) -> Vec<Vec<State>> {
    let mut v = vec![vec![]; n];
    v.push(vec!["end".parse_tc().unwrap()]);
    v
}

// 最後の edge の番号 = n
fn series_edge_end_only(n: usize) -> Vec<((usize, usize), State)> {
    (0..n)
        .map(|i| ((i, i + 1), "end".parse_tc().unwrap()))
        .collect()
}

#[cfg(test)]
fn vec_sign(vec: Vec<&str>) -> Vec<Sign> {
    vec.into_iter().map(|s| s.parse_tc().unwrap()).collect()
}

#[cfg(test)]
pub(crate) fn tape_from(symbols: &[&str], head: usize) -> Result<Tape, String> {
    Tape::from_vec(vec_sign(symbols.to_vec()), head)
}

pub fn zero_builder() -> TuringMachineBuilder {
    let definition: TuringMachineDefinition = include_str!("zero_builder.txt").parse_tc().unwrap();
    let mut builder =
        TuringMachineBuilder::new("zero_builder", definition.init_state().clone()).unwrap();
    builder.accepted_state = definition.accepted_state().clone();
    builder.code = definition
        .code()
        .clone()
        .into_iter()
        .map(Into::into)
        .collect();
    builder
}

pub fn succ_builder() -> TuringMachineBuilder {
    let definition: TuringMachineDefinition = include_str!("succ_builder.txt").parse_tc().unwrap();
    let mut builder =
        TuringMachineBuilder::new("succ_adder", definition.init_state().clone()).unwrap();
    builder.accepted_state = definition.accepted_state().clone();
    builder.code = definition
        .code()
        .clone()
        .into_iter()
        .map(Into::into)
        .collect();
    builder
}
*/
