use crate::machine::*;
use crate::manipulation::{TuringMachineBuilder, Interpretation, compose_builder};

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

impl From<usize> for Number {
    fn from(value: usize) -> Self {
        Number(value)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NumberTuple(Vec<Number>);

impl TryFrom<String> for NumberTuple {
    type Error = String;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        let value = value.trim();
        if !(value.starts_with('(') && value.ends_with(')')) {
            return Err("not tuple".to_string());
        }
        let vec = value.get(1..value.len()-1).unwrap()
            .split(',')
            .map(|str| {
                match str.trim().parse() {
                    Ok(n) => Ok(Number(n)),
                    Err(_) => Err("parse fail".to_string())
                }
            })
            .collect::<Result<_, _>>()?;
        Ok(NumberTuple(vec))
    }
}

impl Into<String> for NumberTuple {
    fn into(self) -> String {
        let mut s = String::new();
        s.push('(');
        for (i, Number(num)) in self.0.iter().enumerate() {
            if i != 0 {
                s.push(',');
            }
            s.push_str(&num.to_string());
        }
        s.push(')');
        s
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
    pub fn interpretation() -> Interpretation<String, String> {
        fn write(input: String) -> Result<TapeAsVec, String> {
            let right: Vec<Sign> = NumberTuple::try_from(input)?.0
                .into_iter()
                .flat_map(|Number(num)| {
                        std::iter::repeat(NatNumInterpretation::one())
                        .take(num)
                        .chain(std::iter::once(NatNumInterpretation::partition()))
                })
                .collect();

            Ok(TapeAsVec::new(
                Vec::new(),
                NatNumInterpretation::partition(),
                right,
            ))
        }
        fn read(tape: TapeAsVec) -> Result<String, String> {
            let mut vec = Vec::new();
            let right = tape.right.clone();
            
            let mut num = 0;
            for i in 0..right.len() {
                match right[i] {
                    _ if right[i] == NatNumInterpretation::partition() => {
                        vec.push(Number(num))
                    }
                    _ if right[i] == NatNumInterpretation::one() => {
                        num += 1;
                    }
                    _ if right[i] == Sign::blank() => {
                        break;
                    }
                    _ => unreachable!()
                }
            }
            Ok(NumberTuple(vec).into())
        }
        Interpretation::new(
            write,
            read
        )
    }
}

pub fn inc() -> TuringMachineBuilder<String, String> {
    let mut builder = TuringMachineBuilder::new("increment", NatNumInterpretation::interpretation()).unwrap();
    builder
        .init_state(State::try_from("start").unwrap())
        .accepted_state(vec![
            State::try_from("end").unwrap()
        ])
        // .code_push(" , start_inc , , end_inc , C").unwrap()
        .code_push_str("-, start, -, read, R").unwrap()
        .code_push_str("1, read, 1, read, R").unwrap()
        .code_push_str("-, read, 1, write, R").unwrap()
        .code_push_str(" , write, -, write_end, L").unwrap()
        .code_push_str("1, write_end, 1, write_end, L").unwrap()
        .code_push_str("-, write_end, - , end, C").unwrap()
        ;
    builder
}

pub fn inc_inc_4_example(i: usize) -> TuringMachineBuilder<String, String> {
    let mut builder = compose_builder(inc(), State::try_from("end").unwrap(), inc()).unwrap();
    builder.input("(4)".to_string());
    builder
}

mod test {
    use super::*;
    #[test]
    fn number_string(){
        let str = "  (1, 0,15) ";
        assert_eq!(NumberTuple::try_from(str.to_string()), Ok(NumberTuple(vec![Number(1), Number(0), Number(15)])))
    }

    #[test]
    fn inc_test1() {
        // let number_pred = Number(10);

        let mut builder = inc();
        let mut machine = builder
            .input("(10)".to_string())
            .build().unwrap();

        machine.step(100);

        let result = machine.result().unwrap();
        let result = NatNumInterpretation::interpretation().read()(result).unwrap();
        assert_eq!("(11)".to_string(), result)
    }

    // #[test]
    // fn inc_test2() {
    //     let number_pred = Number(10);

    //     let mut builder = compose_builder(inc(), State::try_from("end").unwrap(), inc()).unwrap();
    //     let mut machine = builder
    //         .build(NumberTuple(vec![number_pred.clone()])).unwrap();

    //     machine.step(200);
        
    //     let result = machine.result().unwrap();
    //     assert_eq!(NumberTuple(vec![number_pred.succ()]), result)
    // }
}
