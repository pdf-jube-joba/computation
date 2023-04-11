use crate::machine::*;
use std::{collections::HashSet};

#[derive(Clone)]
pub struct  Interpretation<Input, Output> where
    Input: Clone,
    Output: Clone
{
    write: fn(Input) -> Result<TapeAsVec, String>,
    read: fn(TapeAsVec) -> Result<Output, String>,
}

impl<Input, Output> Interpretation<Input, Output> where
    Input: Clone,
    Output: Clone,
{
    pub fn new(
        write: fn(Input) -> Result<TapeAsVec, String>,
        read: fn(TapeAsVec) -> Result<Output, String>
    ) -> Interpretation<Input, Output> {
        Interpretation { write, read }
    }
    pub fn write(&self) -> fn(Input) -> Result<TapeAsVec, String> {
        self.write
    }
    pub fn read(&self) -> fn(TapeAsVec) -> Result<Output, String> {
        self.read
    }
}

// pub trait Interpretation {
//     type Input: Clone;
//     type Output: Clone;
//     fn write(&self, input: Self::Input) -> Result<TapeAsVec, String>;
//     fn read(&self, tape: TapeAsVec) -> Result<Self::Output, String>;
// }

type StandardIntepretation = Interpretation<TapeAsVec, TapeAsVec>;

pub fn standard_interpretation() -> Interpretation<TapeAsVec, TapeAsVec> {
    fn write_std(tape: TapeAsVec) -> Result<TapeAsVec, String> {
        Ok(tape)
    }
    fn read_std(tape: TapeAsVec) -> Result<TapeAsVec, String> {
        Ok(tape)
    }
    StandardIntepretation {
        write: write_std,
        read: read_std, 
    }
}

// impl StandardIntepretation {
//     type Input = TapeAsVec;
//     type Output = TapeAsVec;
//     fn write(&self, input: Self::Input) -> Result<TapeAsVec, String> {
//         Ok(input)
//     }
//     fn read(&self, tape: TapeAsVec) -> Result<Self::Output, String> {
//         Ok(tape)
//     }
// }

pub fn write_str(input: String) -> Result<TapeAsVec, String> {
    let lines: Vec<&str> = input.lines().collect();
    if lines.len() < 3 {return Err("error on str to tape".to_string())}
    let left: Vec<Sign> = lines[0].split_whitespace().map(|s| Sign::try_from(s)).collect::<Result<Vec<Sign>, String>>()?;
    let head = Sign::try_from(lines[1])?;
    let right: Vec<Sign> = lines[2].split_whitespace().map(|s| Sign::try_from(s)).collect::<Result<Vec<Sign>, String>>()?;
    Ok(TapeAsVec {
        left,
        head,
        right, 
    })
}

// fn composition<A,B,C>(f: fn(A) -> B, g: fn(B) -> C) -> fn(A) -> C {
//     todo!()
// }

// fn stringfy<Input, Output>(i: Interpretation<Input, Output>) -> Interpretation<String, String> where
//     Input: Clone + TryFrom<String, Error = String>,
//     Output: Clone,
// {
//     let Interpretation { write, read } = i;
//     todo!()
// }

// struct Stringfy<I, In, Out> where
//     I: Interpretation<Input = In, Output = Out>,
//     In: Clone + TryFrom<String, Error = String>,
//     Out: Clone + Into<String>,
// {
//     content: I
// }

// impl<I, In, Out> Interpretation for Stringfy<I, In, Out> where
//     I: Interpretation<Input = In, Output = Out>,
//     In: Clone + TryFrom<String, Error =  String>,
//     Out: Clone + Into<String>,
// {
//     type Input = String;
//     type Output = String;
//     fn write(&self, input: Self::Input) -> Result<TapeAsVec, String> {
//         self.content.write(input.try_into()?)
//     }
//     fn read(&self, tape: TapeAsVec) -> Result<Self::Output, String> {
//         Ok(self.content.read(tape)?.into())
//     }
// }

// pub struct CompositionInterpretation<In, Mid, Out>
// {
//     first: Box<dyn Interpretation<Input = In, Output = Mid>>,
//     second: Box<dyn Interpretation<Input = Mid, Output = Out>>,
// }

// impl<In, Mid, Out> Interpretation for CompositionInterpretation<In, Mid, Out> where
//     In : Clone, Mid: Clone, Out: Clone,
// {
//     type Input = In;
//     type Output = Out;
//     fn write(&self, input: Self::Input) -> Result<TapeAsVec, String> {
//         self.first.write(input)
//     }
//     fn read(&self, tape: TapeAsVec) -> Result<Self::Output, String> {
//         self.second.read(tape)
//     }
// }

// impl<In, Out> Interpretation for Box<dyn Interpretation<Input = In, Output = Out>> where
//     In: Clone, Out: Clone,
// {
//     type Input = In;
//     type Output = Out;
//     fn write(&self, input: Self::Input) -> Result<TapeAsVec, String> {
//         self.as_ref().write(input)
//     }
//     fn read(&self, tape: TapeAsVec) -> Result<Self::Output, String> {
//         self.as_ref().read(tape)
//     }
// }

pub struct TuringMachineBuilder<Input, Output> where
    Input: Clone,
    Output: Clone,
{
    name: String,
    init_state: Option<State>,
    accepted_state: Option<Vec<State>>,
    code: Vec<CodeEntry>,
    interpretation: Interpretation<Input, Output>,
    input: Option<Input>,
}

impl<Input, Output> TuringMachineBuilder<Input, Output> where
    Input: Clone ,
    Output: Clone ,
{
    pub fn new(name: &str, interpretation: Interpretation<Input, Output>) -> Result<TuringMachineBuilder<Input, Output>, String> {
        if name.is_empty() {return Err("empty string".to_string())}
        let builder = TuringMachineBuilder {
            name: name.to_string(),
            init_state: None,
            accepted_state: None,
            code: Vec::new(),
            interpretation: interpretation,
            input: None,
        };
        Ok(builder)
    }
    pub fn build(&self) -> Result<TuringMachineSet, String> where
        Input: Clone + 'static,
        Output: Clone + 'static,
    {
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
        let input = self.input.clone().ok_or("input not found".to_string())?;
        let Interpretation { write, read } = self.interpretation.clone();
        let machine = TuringMachineSet::new(
            init_state,
            accepted_state,
            self.code.clone(),
            write(input.clone())?
        );
        // let run = RunningTuringMachine {
        //     machine,
        //     input,
        //     on_terminate: read ,
        // };
        Ok(machine)
    }

    pub fn input(&mut self, input: Input) -> &mut Self {
        self.input = Some(input);
        self
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

}

impl<In, Out> TuringMachineBuilder<In, Out> where
    In: Clone + TryFrom<String, Error =  String> + Into<String> + 'static,
    Out: Clone + Into<String> + 'static,
{
    pub fn stringfy(self) -> TuringMachineBuilder<String, String> {
        let TuringMachineBuilder { name, init_state, accepted_state, code, interpretation, input } = self;
        // let new_interpretation = Stringfy {
        //     content: interpretation,
        // };
        let input = match input {
            Some(input) => Some(input.into()),
            None => None,
        };
        TuringMachineBuilder {
            name,
            init_state,
            accepted_state,
            code,
            interpretation: todo!(),
            input,
        }
    }
}

// pub struct RunningTuringMachine<Input, Output> where
//     Input: Clone ,
//     Output: Clone ,
// {
//     machine: TuringMachineSet,
//     input: Input,
//     on_terminate: fn(TapeAsVec) -> Result<Output, String>,
// }

// impl<Input, Output> RunningTuringMachine<Input, Output> where
//     Input: Clone ,
//     Output: Clone ,
// {
//     pub fn now_state(&self) -> State {
//         self.machine.now_state().clone()
//     }
//     pub fn now_tape(&self) -> TapeAsVec {
//         self.machine.now_tape()
//     }
//     pub fn code_as_vec(&self) -> Vec<CodeEntry> {
//         self.machine.code_as_vec()
//     }
//     pub fn step(&mut self, n: usize) -> Result<(), usize> {
//         for i in 0..n {
//             if self.machine.is_terminate() {return Err(i)}
//             self.machine.step()
//         }
//         Ok(())
//     }
//     pub fn result(&self) -> Result<Output, String> {
//         if !self.machine.is_terminate() {return Err("not terminated".to_string());}
//         let tape = self.machine.now_tape();
//         let read = self.on_terminate;
//         read(tape)
//     }
//     pub fn first_input(&self) -> &Input {
//         &self.input
//     }
// }

pub fn compose_builder<In, Mid, Out>(first: TuringMachineBuilder<In, Mid>, specified_state: State, second: TuringMachineBuilder<Mid, Out>)
    -> Result<TuringMachineBuilder<In, Out>, String> where
    In: 'static + Clone,
    Mid: 'static + Clone,
    Out: 'static + Clone,
{
    let TuringMachineBuilder {
        name: first_name,
        init_state: first_init_state,
        accepted_state: first_accepted_state,
        code: first_code,
        interpretation: first_interpretation,
        input: first_input,
    } = first;
    let TuringMachineBuilder {
        name: second_name,
        init_state: second_init_state,
        accepted_state: second_accepted_state,
        code: second_code,
        interpretation: second_interpretation,
        input: second_input,
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

    let composition_interpretation = Interpretation {
        write: first_interpretation.write,
        read: second_interpretation.read,
    };

    let mut builder = TuringMachineBuilder::new(&name, composition_interpretation).unwrap();
        builder
            .init_state(init_state)
            .accepted_state(accepted_state)
            .code_from_entries(code);

    match first_input {
        Some(input) => {
            builder.input(input);
        }
        None => {},
    }

    Ok::<TuringMachineBuilder<In, Out>, String>(builder)
}
