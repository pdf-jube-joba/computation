use std::process::Output;

use super::*;

#[derive(Debug, Clone, PartialEq)]
pub struct TuringMachineBuilder {
    name: String,
    init_state: Option<State>,
    accepted_state: Option<Vec<State>>,
    code: Vec<CodeEntry>,
    initial_tape: Tape,
}

impl TuringMachineBuilder {
    pub fn new(name: &str) -> Result<TuringMachineBuilder, String> {
        if name.is_empty() {return Err("empty string".to_string())}
        let builder = TuringMachineBuilder {
            name: name.to_string(),
            init_state: None,
            accepted_state: None,
            code: Vec::new(),
            initial_tape: Tape::default(),
        };
        Ok(builder)
    }
    pub fn build(self) -> Result<TuringMachineSet, String> {
        let machine_code = {
            let init_state = if let Some(state) = self.init_state.clone() {
                state
            } else {
                return Err("fail on initial state".to_string());
            };
            let accepted_state = if let Some(state) = self.accepted_state.clone() {
                HashSet::from_iter(state.into_iter())
            } else {
                return Err("fail on accepted state".to_string());
            };
            let code = Code::from_iter_entry(self.code);
            TuringMachine {
                init_state,
                accepted_state,
                code,
            }
        };
        let machine_state = {
            let state = self.init_state.unwrap();
            let tape = self.initial_tape;
            TuringMachineState { state, tape }
        };
        Ok(TuringMachineSet {
            machine_code,
            machine_state,
        })
    }

    fn init_state(&mut self, state: State) -> &mut Self {
        self.init_state = Some(state);
        self
    }

    pub fn init_state_from_str(&mut self, str: &str) -> Result<&mut Self, String> {
        self.init_state = Some(State::try_from(str)?);
        Ok(self)
    }

    fn accepted_state(&mut self, states: impl IntoIterator<Item = State>) -> &mut Self {
        self.accepted_state = Some(states.into_iter().collect());
        self
    }

    pub fn accepted_state_from_str(&mut self, str: &str) -> Result<&mut Self, String> {
        self.accepted_state = str
            .split_whitespace()
            .map(|str| State::try_from(str).unwrap())
            .collect::<Vec<State>>()
            .into();
        Ok(self)
    }

    fn code_from_entries(&mut self, entries: impl IntoIterator<Item = CodeEntry>) -> &mut Self {
        self.code = entries.into_iter().collect();
        self
    }

    pub fn code(&mut self, str: &str) -> Result<&mut Self, String> {
        let mut vec = Vec::new();
        for entry in str.lines().map(CodeEntry::try_from) {
            vec.push(entry?)
        }
        self.code = vec;
        Ok(self)
    }

    pub fn code_push(&mut self, str: &str) -> Result<&mut Self, String> {
        let entry = CodeEntry::try_from(str)?;
        self.code.push(entry);
        Ok(self)
    }

    pub fn code_refresh(&mut self) {
        self.code = Vec::new();
    }

    pub fn initial_tape_left(&mut self, str: &str) -> Result<&mut Self, String> {
        self.initial_tape.left = to_vec_sign(str);
        Ok(self)
    }

    pub fn initial_tape_head(&mut self, str: &str) -> Result<&mut Self, String> {
        self.initial_tape.head = Sign::try_from(str)?;
        Ok(self)
    }

    pub fn initial_tape_right(&mut self, str: &str) -> Result<&mut Self, String> {
        self.initial_tape.right = to_vec_sign(str);
        Ok(self)
    }
    pub fn initial_tape_from_str(&mut self, str: &str) -> Result<&mut Self, String> {
        self.initial_tape = Tape::try_from(str)?;
        Ok(self)
    }
    fn initial_tape(&mut self, tape: Tape) {
        self.initial_tape = tape;
    }

    fn composition(self, specified_state: State, other: Self) -> Result<Self, String> {
        let TuringMachineBuilder {
            name: first_name,
            init_state: first_init_state,
            accepted_state: first_accepted_state,
            code: first_code,
            initial_tape: first_initial_tape,
        } = self;
        let TuringMachineBuilder {
            name: second_name,
            init_state: second_init_state,
            accepted_state: second_accepted_state,
            code: second_code,
            initial_tape: _,
        } = other;

        let first_init_state = if let Some(state) = first_init_state {state} else {
            return Err("first arg's init_state not setted".to_string());
        };
        let first_accepted_state = if let Some(states) = first_accepted_state {states} else {
            return Err("first arg's accepted state not setted".to_string());
        };
        let second_init_state = if let Some(state) = second_init_state {state} else {
            return Err("second arg's init_state not setted".to_string());
        };
        let second_accepted_state = if let Some(states) = second_accepted_state {states} else {
            return Err("second arg's accepted state not setted".to_string());
        };

        let name = format!("{first_name}-{second_name}");

        let init_state = first_init_state;

        let accepted_state = {
            if !first_accepted_state.contains(&specified_state) {
                return Err("".to_string());
            }
            let mut accepted_state = Vec::new();
            accepted_state
                .extend(first_accepted_state.into_iter().filter(|state|{
                    *state != specified_state
                }));
            accepted_state.extend(second_accepted_state);
            accepted_state
        };

        let code = {
            let used_sign = first_code.iter()
                .chain((second_code).iter())
                .flat_map(|CodeEntry(CodeKey(s1, _), CodeValue(s2, _, _))| vec![s1.clone(), s2.clone()])
                .collect::<HashSet<Sign>>();

            let mut code = Vec::new();
            code.extend(
                first_code.into_iter()
                .map(|CodeEntry(CodeKey(s_1, q_1), CodeValue(s_2, q_2, m))| {
                    let new_q_1 = State::try_from(format!("{first_name}-{q_1}").as_ref()).unwrap();
                    let new_q_2 = State::try_from(format!("{first_name}-{q_2}").as_ref()).unwrap();
                    CodeEntry(CodeKey(s_1, new_q_1), CodeValue(s_2, new_q_2, m))
            }));
            code.extend(
                second_code.into_iter()
                .map(|CodeEntry(CodeKey(s_1, q_1), CodeValue(s_2, q_2, m))| {
                let new_q_1 = State::try_from(format!("{second_name}-{q_1}").as_ref()).unwrap();
                let new_q_2 = State::try_from(format!("{second_name}-{q_2}").as_ref()).unwrap();
                CodeEntry(CodeKey(s_1, new_q_1), CodeValue(s_2, new_q_2, m))
            })
            );
            code.extend(
                used_sign.into_iter()
                .map(|sign|{
                    CodeEntry(CodeKey(sign.clone(), specified_state.clone()), CodeValue(sign, second_init_state.clone(), Direction::Constant))
                })
            );
            code
        };

        let mut builder = TuringMachineBuilder::new(&name).unwrap();
            builder
                .init_state(init_state)
                .accepted_state(accepted_state)
                .code_from_entries(code)
                .initial_tape(first_initial_tape);
        Ok(builder)
    }
}

trait Interpretation {
    type Input;
    type Output;
    fn write(input: Self::Input) -> Tape;
    fn read(tape: &Tape) -> Result<Self::Output, String>;
}

pub mod example {
    use std::iter::Once;

    use crate::machine::*;
    use crate::machine::manipulation::TuringMachineBuilder;

    use super::Interpretation;

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
            Tape {
                left: vec![],
                head: Sign::try_from("-").unwrap(),
                right: input
                    .into_iter()
                    .flat_map(|num| std::iter::repeat(Sign::try_from("1").unwrap())
                        .take(num.0).chain(std::iter::once(Sign::try_from("-").unwrap())))
                    .collect(),
            }
        }
        fn read(tape: &Tape) -> Result<Self::Output, String> {
            let mut vec = Vec::new();
            for l in tape.right.split(|sign| Sign::try_from("-").unwrap() == *sign) {
                if l.iter().all(|sign| NatNumInterpretation::partition() == *sign) {
                    vec.push(Number(l.len()));
                } else {
                    return Err("fail on interpreting".to_string());
                }
            }
            Ok(vec)
        }
    }

    fn inc() -> TuringMachineBuilder {
        let mut builder = TuringMachineBuilder::new("one").unwrap();
        builder
            .init_state(State::try_from("start").unwrap())
            .accepted_state(vec![
                State::try_from("end").unwrap()
            ])
            // .code_push(" , start_inc , , end_inc , C").unwrap()
            .code_push("1, start, 1, read, C").unwrap()
            .code_push("1, read, 1, read, R").unwrap()
            .code_push(" , read, 1, read_end, L").unwrap()
            .code_push("1, read_end, 1, read_end, L").unwrap()
            .code_push(" , read_end,  , end, R").unwrap()
            ;
        builder
    }

    pub fn inc_example(i: usize) -> TuringMachineBuilder {
        let mut builder = inc();
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
            eprintln!("{machine}");

            for i in 0..100 {
                if machine.is_terminate() {break;}
                machine.step();
                eprintln!("{i} step {machine:?}");
            }
            let tape = machine.machine_state.tape;
            let number_succ = NatNumInterpretation::read(&tape).unwrap()[0].clone();
            assert_eq!(number_pred.succ(), number_succ);
        }
    }
}
