use crate::machine::*;
use crate::manipulation::{TuringMachineBuilder, Interpretation};

#[derive(Debug, Clone, PartialEq)]
pub struct Number(usize);

impl Number {
    fn is_zero(self) -> bool {
        self.0 == 0
    }
    fn succ(self) -> Self {
        Number(self.0 + 1)
    }
}
pub struct NatNumInterpretation;

impl NatNumInterpretation {
    fn partition() -> Sign {
        Sign::try_from("-").unwrap()
    }
    fn one() -> Sign {
        Sign::try_from("1").unwrap()
    }
}

impl Interpretation for NatNumInterpretation {
    type Input = Vec<Number>;
    type Output = Vec<Number>;
    fn write(input: Self::Input) -> Tape {
        let right: Vec<Sign> = input
            .into_iter()
            .flat_map(|num| {
                    std::iter::repeat(Sign::try_from("1").unwrap())
                    .take(num.0)
                    .chain(std::iter::once(Sign::try_from("-").unwrap()))
            })
            .collect();

        eprintln!("{right:?}");

        Tape::new(
            Vec::new(),
            Sign::try_from("-").unwrap(),
            right,
        )
    }
    
    fn read(tape: &Tape) -> Result<Self::Output, String> {
        let mut vec = Vec::new();
        let right = tape.right();
        
        let mut num = 0;
        for i in 0..right.len() {
            match right[i] {
                _ if *right[i] == Sign::try_from("-").unwrap() => {
                    vec.push(Number(num))
                }
                _ if *right[i] == Sign::try_from("1").unwrap() => {
                    num += 1;
                }
                _ if *right[i] == Sign::blank() => {
                    break;
                }
                _ => unreachable!()
            }
        }
        // for l in tape.right.(|sign| NatNumInterpretation::partition() == *sign) {
        //     eprintln!("1:{l:?}");
        //     if l.iter().all(|sign| NatNumInterpretation::one() == *sign) {
        //         vec.push(Number(l.len()));
        //     } else {
        //         return Err("fail on interpreting".to_string());
        //     }
        // }
        Ok(vec)
    }
}

fn inc() -> TuringMachineBuilder<Vec<Number>, Vec<Number>> {
    let mut builder = TuringMachineBuilder::new("one").unwrap();
    builder
        .init_state(State::try_from("start").unwrap())
        .accepted_state(vec![
            State::try_from("end").unwrap()
        ])
        // .code_push(" , start_inc , , end_inc , C").unwrap()
        .code_push("-, start, -, read, R").unwrap()
        .code_push("1, read, 1, read, R").unwrap()
        .code_push("-, read, 1, write, R").unwrap()
        .code_push(" , write, -, write_end, L").unwrap()
        .code_push("1, write_end, 1, write_end, L").unwrap()
        .code_push("-, write_end, - , end, C").unwrap()
        ;
    builder
}

pub fn inc_example(i: usize) -> TuringMachineBuilder::<Vec<Number>, Vec<Number>> {
    let mut builder = inc();
    builder
        .initial_tape(NatNumInterpretation::write(vec![Number(i)]));
    builder
}

pub fn inc_composition_example(i: usize) -> TuringMachineBuilder {
    let mut builder = inc().composition(State::try_from("end").unwrap(), inc()).unwrap();
    builder
        .initial_tape(NatNumInterpretation::write(vec![Number(i)]));

    builder
}

mod test {
    use super::*;

    #[test]
    fn inc_test1() {
        let number_pred = Number(10);

        let mut builder = inc();
        builder
            .initial_tape(NatNumInterpretation::write(vec![number_pred.clone()]));
        
        let mut machine = builder.build().unwrap();

        for _ in 0..100 {
            if machine.is_terminate() {break;}
            machine.step();
        }
        let tape = machine.machine_state.tape;
        let number_succ = NatNumInterpretation::read(&tape).unwrap()[0].clone();
        assert_eq!(number_pred.succ(), number_succ);
    }

    #[test]
    fn inc_test2() {
        let number_pred = Number(10);

        let mut builder = inc().composition(State::try_from("end").unwrap(), inc()).unwrap();
        
        builder
            .initial_tape(NatNumInterpretation::write(vec![number_pred.clone()]));
        
        let mut machine = builder.build().unwrap();

        for _ in 0..100 {
            if machine.is_terminate() {break;}
            machine.step();
        }
        let tape = machine.machine_state.tape;
        let number_succ = NatNumInterpretation::read(&tape).unwrap()[0].clone();
        assert_eq!(number_pred.succ().succ(), number_succ);
    }
}