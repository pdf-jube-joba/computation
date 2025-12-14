use turing_machine::{machine::*, manipulation::builder::TuringMachineBuilder};

struct Builder<'a> {
    name: String,
    code: Vec<&'a str>,
}

impl<'a> From<Builder<'a>> for TuringMachineBuilder {
    fn from(builder: Builder) -> Self {
        let mut tm_builder = TuringMachineBuilder::new(&builder.name).unwrap();
        tm_builder.init_state("start".parse().unwrap());
        tm_builder.accepted_state(vec!["end".parse().unwrap()]);
        for entry in builder.code {
            let entry = turing_machine::manipulation::code::parse_one_code_entry(entry).unwrap();
            tm_builder.code_push(entry);
        }
        tm_builder
    }
}

// 最後の edge の番号 = n
fn accept_end_only(n: usize) -> Vec<Vec<State>> {
    let mut v = vec![vec![]; n];
    v.push(vec!["end".parse().unwrap()]);
    v
}

// 最後の edge の番号 = n
fn series_edge_end_only(n: usize) -> Vec<((usize, usize), State)> {
    (0..n)
        .map(|i| ((i, i + 1), "end".parse().unwrap()))
        .collect()
}

#[cfg(test)]
fn vec_sign(vec: Vec<&str>) -> Vec<Sign> {
    vec.into_iter().map(|s| s.parse().unwrap()).collect()
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
fn builder_test(builder: &mut TuringMachineBuilder, step: usize, tests: Vec<(Result<Tape, String>, Result<Tape, String>)>) {
    eprintln!("test start");
    for (input, expect) in tests {
        let input = input.unwrap();
        let expect = expect.unwrap();
        eprintln!("input: {}", input);
        let mut machine = builder.build(input).unwrap();
        eprintln!("{:?}\n    {}", machine.now_state(), machine.now_tape());
        for _ in 0..step {
            let _ = machine.step(1);
            eprintln!("__{:?}\n    {}", machine.now_state(), machine.now_tape());
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
        eprintln!("{:?}\n    {}", machine.now_state(), machine.now_tape());
        for _ in 0..step {
            let _ = machine.step(1);
            eprintln!("__{:?}\n    {}", machine.now_state(), machine.now_tape());
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
