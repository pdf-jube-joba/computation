mod example {
    use crate::machine::*;

    fn one() -> Sign {
        Sign::from("1")
    }

    struct Number(usize);
    impl Number {
        fn to_signs(self) -> Vec<Sign> {
            let Number(num) = self;
            (0..num).map(|_| one()).collect()
        }
    }

    fn write_natural_numbers(vec: Vec<Number>) -> Tape {
        let mut tape =
        Tape {
            left: vec![],
            head: Sign::blank(),
            right: vec.into_iter().flat_map(|num|{
                num.to_signs().into_iter()
            }).collect(),
        };
        tape.move_to(&Direction::Right);
        tape
    }

    fn read_natural_numbers(tape: Tape) -> Result<Vec<Number>, ()> {
        let mut vec = Vec::new();
        for l in tape.right.split(|sign| Sign::blank() == *sign) {
            if l.iter().all(|sign| one() == *sign) {
                vec.push(Number(l.len()));
            } else {
                return Err(());
            }
        }
        Ok(vec)
    }

    fn inc() -> TuringMachine {
        TuringMachine {
            init_state: State(String::from("start_inc")),
            accepted_state: HashSet::from_iter(vec![
                State("".to_string()),
            ]),
            code: Code::try_from(
                ",start_inc,,read,R
                ").unwrap(),
        }
    }

    mod test {
        use crate::{machine::TuringMachine, builder};
        use super::*;

        #[test]
        fn inc_test1() {
            let mut builder = TuringMachineBuilder::default();
                builder
                .init_state("start_add").unwrap();
            // let builder = TuringMachineBuilder {
            //     init_state: String::from("start_add"),
            //     accepted_state: String::from("end"),
            //     code: String::from(",start_add,,read,R"),
            //     initial_tape: String::from(""),
            // };
            let mut machine = match builder.build() {
                Ok(machine) => machine,
                Err(err) => panic!("error! {err}"),
            };
            machine.step();
        }
    }
}