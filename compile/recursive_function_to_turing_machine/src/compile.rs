use recursive_function::machine::RecursiveFunctions;
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

fn sign(str: &str) -> Sign {
    Sign::try_from(str).unwrap()
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

pub mod projection;

pub mod composition;
pub mod mu_recursion;
pub mod primitive_recursion;

pub fn compile(recursive_function: RecursiveFunctions) -> TuringMachineBuilder {
    match recursive_function {
        RecursiveFunctions::ZeroConstant => zero_builder(),
        RecursiveFunctions::Successor => succ_builder(),
        RecursiveFunctions::Projection(proj) => {
            projection::projection(proj.parameter_length(), proj.projection_num())
        }
        RecursiveFunctions::Composition(composition) => {
            let recursive_function::machine::Composition {
                parameter_length: _,
                outer_func,
                inner_func,
            } = composition;
            let outer_builder = compile(*outer_func.to_owned());
            let inner_builders: Vec<TuringMachineBuilder> = inner_func
                .to_owned()
                .into_iter()
                .map(|func| compile(func))
                .collect();
            composition::composition(inner_builders, outer_builder)
        }
        RecursiveFunctions::PrimitiveRecursion(prim) => {
            let recursive_function::machine::PrimitiveRecursion {
                zero_func,
                succ_func,
            } = prim;
            primitive_recursion::primitive_recursion(
                compile(*zero_func.to_owned()),
                compile(*succ_func.to_owned()),
            )
        }
        RecursiveFunctions::MuOperator(muop) => {
            let recursive_function::machine::MuOperator { mu_func } = muop;
            mu_recursion::mu_recursion(compile(*mu_func.to_owned()))
        }
    }
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
    use crate::compile::projection::projection;

    use super::*;
    use recursive_function::machine::NumberTuple;

    fn print_process(machine: &TuringMachineSet) {
        let state_str = machine.now_state().to_string();
        if state_str.contains("start") || state_str.contains("end") {
            eprintln!("{}\n   {}", machine.now_state(), machine.now_tape());
        }
    }

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
            print_process(&machine);
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
                print_process(&machine);
                if machine.is_terminate() {
                    break;
                }
            }
            let result = num_tape::read_right_one(machine.now_tape());
            assert_eq!(result, Ok(vec![i + 1].into()))
        }
    }
    #[test]
    fn projection_test() {
        let mut builder = projection::projection(2, 0);
        let input: TapeAsVec = num_tape::write(vec![1,2].into());
        builder.input(input);

        let mut machine = builder.build().unwrap();

        loop {
            let _ = machine.step(1);
            print_process(&machine);
            if machine.is_terminate() {
                break;
            }
        }
        
        let result = num_tape::read_right_one(machine.now_tape());
        assert_eq!(result, Ok(vec![1].into()));

        let mut builder = projection::projection(3, 0);
        let input: TapeAsVec = num_tape::write(vec![1,2,3].into());
        builder.input(input);

        let mut machine = builder.build().unwrap();

        loop {
            let _ = machine.step(1);
            print_process(&machine);
            if machine.is_terminate() {
                break;
            }
        }
        
        let result = num_tape::read_right_one(machine.now_tape());
        assert_eq!(result, Ok(vec![1].into()));

        let mut builder = projection::projection(3, 1);
        let input: TapeAsVec = num_tape::write(vec![1,2,3].into());
        builder.input(input);

        let mut machine = builder.build().unwrap();

        loop {
            let _ = machine.step(1);
            print_process(&machine);
            if machine.is_terminate() {
                break;
            }
        }
        
        let result = num_tape::read_right_one(machine.now_tape());
        assert_eq!(result, Ok(vec![2].into()));

        let mut builder = projection::projection(3, 2);
        let input: TapeAsVec = num_tape::write(vec![1,2,3].into());
        builder.input(input);

        let mut machine = builder.build().unwrap();

        loop {
            let _ = machine.step(1);
            print_process(&machine);
            if machine.is_terminate() {
                break;
            }
        }
        
        let result = num_tape::read_right_one(machine.now_tape());
        assert_eq!(result, Ok(vec![3].into()));
    }
    #[test]
    fn composition_test() {
        let mut builder = composition::composition(vec![zero_builder()], succ_builder());
        let input: TapeAsVec = num_tape::write(vec![].into());
        builder.input(input);

        let mut machine = builder.build().unwrap();

        loop {
            let _ = machine.step(1);
            if machine.is_terminate() {
                print_process(&machine);
                break;
            }
        }
        let result = num_tape::read_right_one(machine.now_tape());
        assert_eq!(result, Ok(vec![1].into()));

        let mut builder = composition::composition(
            vec![
                projection::projection(3, 2),
                projection::projection(3, 1),
                projection::projection(3, 0),
            ],
            projection(3, 0)
        );
        let input: TapeAsVec = num_tape::write(vec![1,2,3].into());
        builder.input(input);

        let mut machine = builder.build().unwrap();
        print_process(&machine);

        loop {
            let _ = machine.step(1);
            print_process(&machine);
            if machine.is_terminate() {
                break;
            }
        }
        let result = num_tape::read_right_one(machine.now_tape());
        assert_eq!(result, Ok(vec![3].into()));
    }
    #[test]
    fn primitive_recursion_test() {
        let mut builder = primitive_recursion::primitive_recursion(
            zero_builder(),
            composition::composition(vec![projection::projection(2, 0)], succ_builder()),
        );
        for i in 0..5 {
            let input = num_tape::write(vec![i].into());
            let mut machine = builder.input(input).build().unwrap();

            loop {
                let _ = machine.step(1);
                eprintln!("{}\n    {}", machine.now_state(), machine.now_tape());
                if machine.is_terminate() {
                    break;
                }
            }
            let result = num_tape::read_right_one(machine.now_tape());
            assert_eq!(result, Ok(vec![i].into()));
        }
    }
    #[test]
    fn mu_recursion_test() {
        let mut builder = mu_recursion::mu_recursion(id());
        let input = num_tape::write(vec![].into());
        let mut machine = builder.input(input).build().unwrap();

        loop {
            let _ = machine.step(1);
            eprintln!("{}\n    {}", machine.now_state(), machine.now_tape());
            if machine.is_terminate() {
                break;
            }
        }

        let result = num_tape::read_right_one(machine.now_tape());
        assert_eq!(result, Ok(vec![0].into()));
    }
}
