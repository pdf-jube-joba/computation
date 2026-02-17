use turing_machine::{machine::*, manipulation::builder::TuringMachineBuilder};
use utils::parse::ParseTextCodec;
#[cfg(test)]
use utils::TextCodec;
use turing_machine::manipulation::graph_compose::{builder_composition, GraphOfBuilder};

// start state: "start"
// accept state: "end"
struct Builder<'a> {
    name: String,
    code: Vec<&'a str>,
}

impl<'a> From<Builder<'a>> for TuringMachineBuilder {
    fn from(builder: Builder) -> Self {
        let mut tm_builder = TuringMachineBuilder::new(&builder.name).unwrap();
        tm_builder.init_state("start".parse_tc().unwrap());
        tm_builder.accepted_state(vec!["end".parse_tc().unwrap()]);
        for entry in builder.code {
            let entry = turing_machine::parse::parse_one_code_entry(entry).unwrap();
            tm_builder.code_push(entry);
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
    let mut builder = TuringMachineBuilder::new("zero_builder").unwrap();
    builder
        .from_source(include_str!("zero_builder.txt"))
        .unwrap();
    builder
}

pub fn succ_builder() -> TuringMachineBuilder {
    let mut builder = TuringMachineBuilder::new("succ_adder").unwrap();
    builder
        .from_source(include_str!("succ_builder.txt"))
        .unwrap();
    builder
}

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

pub mod auxiliary;
pub mod compile;
pub mod symbols;

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
