use turing_machine::{machine::*, manipulation::builder::TuringMachineBuilder};

pub mod num_tape {
    use recursive_function::machine::{Number, NumberTuple};
    use turing_machine::machine::{Sign, TapeAsVec};

    fn partition() -> Sign {
        Sign::try_from("-").unwrap()
    }

    fn one() -> Sign {
        Sign::try_from("1").unwrap()
    }

    fn num_sings(num: Number) -> Vec<Sign> {
        (0..num.into()).map(|_| one()).collect()
    }

    pub fn write(tuple: NumberTuple) -> TapeAsVec {
        let vec: Vec<Number> = tuple.into();
        let mut signs: Vec<Sign> = vec
            .into_iter()
            .flat_map(|num: Number| {
                let mut vec = vec![Sign::blank()];
                vec.extend_from_slice(&num_sings(num));
                vec
            })
            .collect();
        signs.extend_from_slice(&vec![partition()]);
        TapeAsVec {
            left: vec![],
            head: partition(),
            right: signs,
        }
    }

    fn read_one(signs: Vec<Sign>) -> Result<NumberTuple, ()> {
        let v = signs
            .split(|char| *char == Sign::blank())
            .map(|vec| vec.len())
            .skip(1);
        Ok(v.collect::<Vec<_>>().into())
    }

    pub fn read_right_one(tape: TapeAsVec) -> Result<NumberTuple, ()> {
        if tape.head != partition() {
            return Err(());
        }
        let iter = tape
            .right
            .iter()
            .take_while(|sign| **sign == Sign::blank() || **sign == one())
            .cloned();
        read_one(iter.collect())
    }
}

fn state(str: &str) -> State {
    State::try_from(str).unwrap()
}

// 最後の edge の番号 = n
fn accept_end_only(n: usize) -> Vec<Vec<State>> {
    let mut v = vec![vec![]; n];
    v.push(vec![state("end")]);
    v
}

// 最後の edge の番号 = n
fn series_edge_end_only(n: usize) -> Vec<((usize, usize), State)> {
    (0..n).map(|i| ((i, i + 1), state("end"))).collect()
}

pub mod basic;
use basic::*;

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

pub mod composition;
pub mod primitive_recursion;
pub mod mu_recursion;

#[cfg(test)]
fn sign(str: &str) -> Sign {
    Sign::try_from(str).unwrap()
}

#[cfg(test)]
fn vec_sign(vec: Vec<&str>) -> Vec<Sign> {
    vec.into_iter().map(|str| sign(str)).collect()
}

#[cfg(test)]
fn builder_test(
    builder: &mut TuringMachineBuilder,
    step: usize,
    tests: Vec<(TapeAsVec, TapeAsVec)>,
) {
    eprintln!("test start");
    for (input, result) in tests {
        let mut machine = builder.input(input).build().unwrap();
        eprintln!("{:?}\n    {}", machine.now_state(), machine.now_tape());
        for _ in 0..step {
            let _ = machine.step(1);
            eprintln!("{:?}\n    {}", machine.now_state(), machine.now_tape());
            if machine.is_terminate() {
                break;
            }
        }
        assert!(machine.is_accepted());
        assert!(machine.now_tape().eq(&result));
    }
}

#[cfg(test)]
fn builder_test_predicate(
    builder: &mut TuringMachineBuilder,
    step: usize,
    tests: Vec<(TapeAsVec, State)>,   
) {
    eprintln!("test start");
    for (input, result) in tests {
        let mut machine = builder.input(input).build().unwrap();
        eprintln!("{:?}\n    {}", machine.now_state(), machine.now_tape());
        for _ in 0..step {
            let _ = machine.step(1);
            eprintln!("{:?}\n    {}", machine.now_state(), machine.now_tape());
            if machine.is_terminate() {
                break;
            }
        }
        assert!(machine.is_accepted());
        assert_eq!(*machine.now_state(), result);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use recursive_function::machine::NumberTuple;

    #[test]
    fn tuple_read_write() {
        fn assert_equal(tuple: NumberTuple) {
            let tape = num_tape::write(tuple.clone());
            let result = num_tape::read_right_one(tape);
            assert_eq!(Ok(tuple), result)
        }

        assert_equal(vec![].into());
        assert_equal(vec![0].into());
        assert_equal(vec![1].into());
        assert_equal(vec![2].into());
        assert_equal(vec![1, 1].into());
        assert_equal(vec![1, 2, 3].into());
    }
    #[test]

    fn test_zero() {
        let mut zero_builder = zero_builder();
        zero_builder.input(num_tape::write(vec![].into()));
        let mut machine = zero_builder.build().unwrap();
        loop {
            let _ = machine.step(1);
            if machine.is_terminate() {
                break;
            }
        }
        let result = num_tape::read_right_one(machine.now_tape());
        assert_eq!(result, Ok(vec![0].into()));
    }
    #[test]
    fn succ_zero() {
        let mut succ_builder = succ_builder();

        for i in 0..5 {
            succ_builder.input(num_tape::write(vec![i].into()));
            let mut machine = succ_builder.build().unwrap();
            loop {
                let _ = machine.step(1);
                if machine.is_terminate() {
                    break;
                }
            }
            let result = num_tape::read_right_one(machine.now_tape());
            assert_eq!(result, Ok(vec![i + 1].into()))
        }
    }
    #[test]
    fn composition_test() {
        let mut builder = composition::composition(vec![zero_builder()], succ_builder());
        let tests = vec![(
            TapeAsVec {
                left: vec![],
                head: sign("-"),
                right: vec_sign(vec!["-"]),
            },
            TapeAsVec {
                left: vec![],
                head: sign("-"),
                right: vec_sign(vec!["", "1", "-"]),
            },
        )];
        builder_test(&mut builder, 2000, tests);
    }
}
