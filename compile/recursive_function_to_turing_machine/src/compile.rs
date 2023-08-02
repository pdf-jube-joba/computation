use std::collections::{HashSet, HashMap};

use turing_machine::{machine::*, manipulation::{builder::{self, TuringMachineBuilder}, graph_compose::{GraphOfBuilder, checked_composition, naive_builder_composition}}, manipulation::code};

mod num_tape {
    use recursive_function::machine::{NumberTuple, Number};
    use turing_machine::machine::{TapeAsVec, Sign};

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
        let mut signs: Vec<Sign> = vec.into_iter().flat_map(|num: Number| {
            let mut vec = vec![Sign::blank()];
            vec.extend_from_slice(&num_sings(num));
            vec
        }).collect();
        signs.extend_from_slice(&vec![partition()]);
        TapeAsVec { left: vec![], head: partition(), right: signs }
    }

    fn read_one(signs: Vec<Sign>) -> Result<NumberTuple, ()> {
        let v = signs.split(|char| *char == Sign::blank()).map(|vec|{
            vec.len()
        }).skip(1);
        Ok(v.collect::<Vec<_>>().into())
    }

    pub fn read_right_one(tape: TapeAsVec) -> Result<NumberTuple, ()> {
        if tape.head != partition() {
            return Err(());
        }
        let iter = tape.right.iter().take_while(|sign| {
            **sign == Sign::blank() || **sign == one()
        }).cloned();
        read_one(iter.collect())
    }

}

fn zero_builder() -> TuringMachineBuilder {
    let code =  code::parse_code(include_str!("zero_builder.txt")).unwrap();
    let mut builder = TuringMachineBuilder::new("zero_builder").unwrap();
    builder
        .code_new(code)
        .init_state(State::try_from("start").unwrap())
        .accepted_state(vec![State::try_from("end").unwrap()]);
    builder
}

fn succ_builder() -> TuringMachineBuilder {
    let code =  code::parse_code(include_str!("succ_builder.txt")).unwrap();
    let mut builder = TuringMachineBuilder::new("succ_adder").unwrap();
    builder
        .code_new(code)
        .init_state(State::try_from("start").unwrap())
        .accepted_state(vec![State::try_from("end").unwrap()]);
    builder
}

fn id() -> TuringMachineBuilder {
    let code =  code::parse_code(include_str!("id.txt")).unwrap();
    let mut builder = TuringMachineBuilder::new("id").unwrap();
    builder
        .code_new(code)
        .init_state(State::try_from("start").unwrap())
        .accepted_state(vec![State::try_from("end").unwrap()]);
    builder
}

fn copy() -> TuringMachineBuilder {
    let code =  code::parse_code(include_str!("copy.txt")).unwrap();
    let mut builder = TuringMachineBuilder::new("copy").unwrap();
    builder
        .code_new(code)
        .init_state(State::try_from("start").unwrap())
        .accepted_state(vec![State::try_from("end").unwrap()]);
    builder
}

fn move_right() -> TuringMachineBuilder {
    let code =  code::parse_code(include_str!("move_right.txt")).unwrap();
    let mut builder = TuringMachineBuilder::new("move_right").unwrap();
    builder
        .code_new(code)
        .init_state(State::try_from("start").unwrap())
        .accepted_state(vec![State::try_from("end").unwrap()]);
    builder
}

fn move_left() -> TuringMachineBuilder {
    let code =  code::parse_code(include_str!("move_left.txt")).unwrap();
    let mut builder = TuringMachineBuilder::new("move_left").unwrap();
    builder
        .code_new(code)
        .init_state(State::try_from("start").unwrap())
        .accepted_state(vec![State::try_from("end").unwrap()]);
    builder
}

fn copy_n(n: usize) -> TuringMachineBuilder {
    if n == 0 { id() } else {
        let graph = GraphOfBuilder {
            name: "a".to_string(),
            assign_vertex_to_builder: vec![copy(), move_right(), copy_n(n-1), move_left()],
            assign_edge_to_state: HashMap::new(),
        };
        naive_builder_composition(graph).unwrap()
    }
}

fn rotate() -> TuringMachineBuilder {
    let code =  code::parse_code(include_str!("rotate.txt")).unwrap();
    let mut builder = TuringMachineBuilder::new("bin_adder").unwrap();
    builder
        .code_new(code)
        .init_state(State::try_from("start").unwrap())
        .accepted_state(vec![State::try_from("end").unwrap()]);
    builder
}

fn composition(inner_builder: Vec<TuringMachineBuilder>, outer_builder: TuringMachineBuilder) -> TuringMachineBuilder {
    unimplemented!()
}

#[cfg(test)]
mod test {
    use recursive_function::machine::NumberTuple;

    use super::{num_tape, zero_builder, succ_builder};

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
        assert_equal(vec![1,1].into());
        assert_equal(vec![1,2,3].into());
    }
    #[test]
    fn test_zero() {
        let mut zero_builder = zero_builder();
        zero_builder.input(num_tape::write(vec![].into()));
        let mut machine = zero_builder.build().unwrap();
        loop {
            let _ = machine.step(1);
            if machine.is_terminate() { break;}
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
            eprintln!("start: {} {:?}", machine.now_state(), machine.now_tape());
            loop {
                let _ = machine.step(1);
                eprintln!("next: {} {:?}", machine.now_state(), machine.now_tape());
                if machine.is_terminate() { break;}
            }
            let result = num_tape::read_right_one(machine.now_tape());
            assert_eq!(result, Ok(vec![i+1].into()))
        }
    }
}
