use crate::machine::*;
use std::{collections::HashSet};

pub trait Interpretation {
    type Input;
    type Output;
    fn write(&self, input: &Self::Input) -> TapeAsVec;
    fn read(&self, tape: &TapeAsVec) -> Result<Self::Output, String>;
}

struct CompositionInterpretation<I1, I2, In, Mid, Out> where 
    I1: Interpretation<Input = In, Output = Mid>,
    I2: Interpretation<Input = Mid, Output = Out>,
{
    first: I1,
    second: I2,
}

impl<I1, I2, In, Mid, Out> Interpretation for CompositionInterpretation<I1, I2, In, Mid, Out> where
    I1: Interpretation<Input = In, Output = Mid>,
    I2: Interpretation<Input = Mid, Output = Out>,
{
    type Input = In;
    type Output = Out;
    fn write(&self, input: &Self::Input) -> TapeAsVec {
        self.first.write(input)
    }
    fn read(&self, tape: &TapeAsVec) -> Result<Self::Output, String> {
        self.second.read(tape)
    }
}

pub struct TuringMachineBuilder<Input, Output> {
    name: String,
    init_state: Option<State>,
    accepted_state: Option<Vec<State>>,
    code: Vec<CodeEntry>,
    initial_tape: TapeAsVec,
    interpretation: Option<Box<dyn Interpretation<Input = Input, Output = Output>>>,
}

impl<Input, Output> TuringMachineBuilder<Input, Output> {
    pub fn new(name: &str) -> Result<TuringMachineBuilder<Input, Output>, String> {
        if name.is_empty() {return Err("empty string".to_string())}
        let builder = TuringMachineBuilder {
            name: name.to_string(),
            init_state: None,
            accepted_state: None,
            code: Vec::new(),
            initial_tape: TapeAsVec::default(),
            interpretation: None,
        };
        Ok(builder)
    }
    pub fn build(&self) -> Result<TuringMachineSet, String> {
        let init_state = if let Some(state) = self.init_state.clone() {
            state
        } else {
            return Err("fail on initial state".to_string());
        };
        let accepted_state = if let Some(accepted_state) = self.accepted_state.clone() {
            accepted_state
        } else {
            return Err("fail on accepted state".to_string());
        };
        Ok(TuringMachineSet::new(
            init_state,
            accepted_state,
            self.code.clone(),
            self.initial_tape.left.clone(),
            self.initial_tape.head.clone(),
            self.initial_tape.right.clone(),
        ))
    }

    pub fn init_state(&mut self, state: State) -> &mut Self {
        self.init_state = Some(state);
        self
    }

    pub fn accepted_state(&mut self, states: impl IntoIterator<Item = State>) -> &mut Self {
        self.accepted_state = Some(states.into_iter().collect());
        self
    }

    pub fn code_from_entries(&mut self, entries: impl IntoIterator<Item = CodeEntry>) -> &mut Self {
        self.code = entries.into_iter().collect();
        self
    }

    pub fn code_from_str(&mut self, str: &str) -> Result<&mut Self, String> {
        let mut vec = Vec::new();
        for entry in str.lines().map(CodeEntry::try_from) {
            vec.push(entry?)
        }
        self.code = vec;
        Ok(self)
    }

    pub fn code_push_str(&mut self, str: &str) -> Result<&mut Self, String> {
        let entry = CodeEntry::try_from(str)?;
        self.code.push(entry);
        Ok(self)
    }

    pub fn code_refresh(&mut self) {
        self.code = Vec::new();
    }

    pub fn initial_tape_left_from_str(&mut self, str: &str) -> Result<&mut Self, String> {
        self.initial_tape.left = parse_str_to_signs(str)?;
        Ok(self)
    }

    pub fn initial_tape_head_from_str(&mut self, str: &str) -> Result<&mut Self, String> {
        self.initial_tape.head = Sign::try_from(str)?;
        Ok(self)
    }

    pub fn initial_tape_right_from_str(&mut self, str: &str) -> Result<&mut Self, String> {
        self.initial_tape.right = parse_str_to_signs(str)?;
        Ok(self)
    }
    pub fn initial_tape_from_str(&mut self, str: &str) -> Result<&mut Self, String> {
        // self.initial_tape = Tape::try_from(str)?;
        Ok(self)
    }
    fn initial_tape(&mut self, tape: TapeAsVec) -> Result<&mut Self, String> {
        self.initial_tape = tape;
        Ok(self)
    }

    pub fn set_interpretation(&mut self, interpretation: Box<dyn Interpretation<Input = Input, Output = Output>>) {
        self.interpretation = Some(interpretation);
    }

    pub fn write(&mut self, input: &Input) -> Result<&mut Self, String> {
        let interpretation = if let Some(interpretation) = &self.interpretation {
            interpretation
        } else {
            return Err("no interpretation".to_string());
        };
        let tape = interpretation.write(input);
        self.initial_tape(tape)?;
        Ok(self)
    }
}

fn composition<In, Mid, Out>(first: TuringMachineBuilder<In, Mid>, specified_state: State, second: TuringMachineBuilder<Mid, Out>) -> Result<TuringMachineBuilder<In, Out>, String> {
    let TuringMachineBuilder {
        name: first_name,
        init_state: first_init_state,
        accepted_state: first_accepted_state,
        code: first_code,
        initial_tape: first_initial_tape,
        interpretation: first_interpretation,
    } = first;
    let TuringMachineBuilder {
        name: second_name,
        init_state: second_init_state,
        accepted_state: second_accepted_state,
        code: second_code,
        initial_tape: _,
        interpretation: second_interpretation,
    } = second;

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

    let first_state_conversion = |state: &State| {
        State::try_from(format!("0-{first_name}-{state}").as_ref()).unwrap()
    };
    let second_state_conversion = |state: &State| {
        State::try_from(format!("1-{second_name}-{state}").as_ref()).unwrap()
    };

    let init_state = first_state_conversion(&first_init_state);

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
            .flat_map(|entry| vec![entry.key_sign(), entry.value_sign()])
            .collect::<HashSet<Sign>>();

        let mut code = Vec::new();
        code.extend(
            first_code.into_iter()
            .map(|entry| {
                let new_key_state = first_state_conversion(&entry.key_state());
                let new_value_state = first_state_conversion(&entry.value_state());
                CodeEntry::from_tuple(entry.key_sign(), new_key_state, entry.value_sign(), new_value_state, entry.value_direction())
        }));
        code.extend(
            second_code.into_iter()
            .map(|entry| {
                let new_key_state = second_state_conversion(&entry.key_state());
                let new_value_state = second_state_conversion(&entry.value_state());
                CodeEntry::from_tuple(entry.key_sign(), new_key_state, entry.value_sign(), new_value_state, entry.value_direction())
        })
        );
        code.extend(
            used_sign.into_iter()
            .map(|sign|{
                CodeEntry::from_tuple(
                    sign.clone(),
                    first_state_conversion(&specified_state),
                    sign,
                    second_state_conversion(&second_init_state),
                    Direction::Constant,
                )
            })
        );
        code
    };

    let handle = ||{
        CompositionInterpretation {
            first: first_interpretation?,
            second: second_interpretation?,
        }
    };

    let mut builder = TuringMachineBuilder::new(&name).unwrap();
        builder
            .init_state(init_state)
            .accepted_state(accepted_state)
            .code_from_entries(code)
            .initial_tape(first_initial_tape)?;
    Ok(builder)
}
