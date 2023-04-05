pub mod example {
    use crate::machine::*;

    fn one() -> Sign {
        Sign::try_from("1").unwrap()
    }

    #[derive(Debug, Clone, PartialEq)]
    pub struct Number(usize);

    impl Number {
        fn succ(self) -> Self {
            Number(self.0 + 1)
        }
        fn to_signs(self) -> Vec<Sign> {
            let Number(num) = self;
            (0..num).map(|_| one()).collect()
        }
    }

    fn write_natural_numbers(vec: Vec<Number>) -> Tape {
        let mut tape = Tape {
            left: vec![],
            head: Sign::blank(),
            right: vec
                .into_iter()
                .flat_map(|num| num.to_signs().into_iter())
                .collect(),
        };
        tape.move_to(&Direction::Right);
        tape
    }

    fn read_natural_numbers(mut tape: Tape) -> Result<Vec<Number>, ()> {
        let mut vec = Vec::new();
        tape.move_to(&Direction::Left);
        for l in tape.right.split(|sign| Sign::blank() == *sign) {
            if l.iter().all(|sign| one() == *sign) {
                vec.push(Number(l.len()));
            } else {
                return Err(());
            }
        }
        Ok(vec)
    }

    fn inc() -> TuringMachineBuilder {
        let mut builder = TuringMachineBuilder::default();
        builder
            .init_state("start_inc").unwrap()
            .accepted_state("end_inc").unwrap()
            // .code_push(" , start_inc , , end_inc , C").unwrap()
            .code_push("1, start_inc, 1, read, C").unwrap()
            .code_push("1, read, 1, read, R").unwrap()
            .code_push(" , read, 1, read_end, L").unwrap()
            .code_push("1, read_end, 1, read_end, L").unwrap()
            .code_push(" , read_end,  , end_inc, R").unwrap()
            ;
        builder
    }

    pub fn inc_example(i: usize) -> TuringMachineBuilder {
        let mut builder = inc();
        builder
            .initial_tape_from_tape(write_natural_numbers(vec![Number(i)]));
        builder
    }

    mod test {
        use super::*;
        use crate::{machine::TuringMachine};

        #[test]
        fn inc_test1() {
            let number_pred = Number(10);

            let mut builder = inc();
            builder
                .initial_tape_from_tape(write_natural_numbers(vec![number_pred.clone()]));
            let mut machine = builder.build().unwrap();
            eprintln!("{machine}");

            for i in 0..100 {
                if machine.is_terminate() {break;}
                machine.step();
                eprintln!("{i} step {machine:?}");
            }
            let tape = machine.machine_state.tape;
            let number_succ = read_natural_numbers(tape).unwrap()[0].clone();
            assert_eq!(number_pred.succ(), number_succ);
        }
    }
}
