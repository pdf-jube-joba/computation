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

    pub fn init_state(&mut self, str: &str) -> Result<&mut Self, String> {
        self.init_state = Some(State::try_from(str)?);
        Ok(self)
    }

    pub fn accepted_state(&mut self, str: &str) -> Result<&mut Self, String> {
        self.accepted_state = str
            .split_whitespace()
            .map(|str| State::try_from(str).unwrap())
            .collect::<Vec<State>>()
            .into();
        Ok(self)
    }

    pub fn code(&mut self, str: &str) -> Result<&mut Self, String> {
        let mut vec = Vec::new();
        for entry in str.lines().map(|str| CodeEntry::try_from(str)) {
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
    pub fn initial_tape(&mut self, str: &str) -> Result<&mut Self, String> {
        self.initial_tape = Tape::try_from(str)?.into();
        Ok(self)
    }
    fn initial_tape_from_tape(&mut self, tape: Tape) {
        self.initial_tape = tape;
    }

    fn composition(self, accepted_state: State, other: Self) -> Result<Self, String> {
        let composition_name = {
            let mut first_name = (&self.name).to_string();
            first_name.push_str(&format!("-{}", (&other.name)));
            first_name
        };
        let code = ;

        let builder = TuringMachineBuilder::new(&composition_name).unwrap();
        Ok(builder)
    }
}

pub mod example {
    use crate::machine::*;
    use crate::machine::manipulation::TuringMachineBuilder;

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
        let mut builder = TuringMachineBuilder::new("one").unwrap();
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
