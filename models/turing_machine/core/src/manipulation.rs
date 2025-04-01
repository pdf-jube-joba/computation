use crate::machine::*;
use std::collections::HashSet;

pub mod code {
    use crate::machine::CodeEntry;

    pub fn parse_one_code_entry(code: &str) -> Result<CodeEntry, String> {
        let v: Vec<_> = code.split(',').collect();
        if v.len() < 5 {
            return Err("Code entry is too short".to_string());
        }
        Ok((
            (v[0].try_into()?, v[1].try_into()?),
            (v[2].try_into()?, v[3].try_into()?, v[4].try_into()?),
        ))
    }

    pub fn parse_code(code: &str) -> Result<Vec<CodeEntry>, (usize, String)> {
        let vec: Vec<CodeEntry> = code
            .lines()
            .enumerate()
            .filter(|(_, line)| line.contains(','))
            .map(|(index, line)| match parse_one_code_entry(line) {
                Ok(entry) => Ok(entry),
                Err(err) => Err((index, err)),
            })
            .collect::<Result<Vec<CodeEntry>, (usize, String)>>()?;
        Ok(vec)
    }
}

pub mod tape {
    use super::*;
    pub fn from_vec_and_position(v: Vec<Sign>, position: usize) -> Tape {
        let (left, right) = v.split_at(position);
        Tape {
            left: left.to_owned(),
            head: right[0].clone(),
            right: right[1..].iter().rev().cloned().collect(),
        }
    }
}

pub mod builder {
    use crate::machine::*;

    use super::code::parse_code;

    #[derive(Debug, Clone)]
    pub enum TuringMachineBuilderError {
        NameIsEmpty,
        InitialStateIsEmpty,
        InputTapeIsEmpty,
        OnParseInitialState,
        OnParseAcceptedStates,
        OnParseCode(usize, String),
        CodeContainsAcceptedState,
    }

    impl From<(usize, String)> for TuringMachineBuilderError {
        fn from((index, err): (usize, String)) -> Self {
            TuringMachineBuilderError::OnParseCode(index, err)
        }
    }

    #[derive(Clone, PartialEq)]
    pub struct TuringMachineBuilder {
        name: String,
        init_state: Option<State>,
        accepted_state: Vec<State>,
        code: Vec<CodeEntry>,
        input: Option<Tape>,
    }

    impl TuringMachineBuilder {
        pub fn new(name: &str) -> Result<TuringMachineBuilder, TuringMachineBuilderError> {
            if name.is_empty() {
                return Err(TuringMachineBuilderError::NameIsEmpty);
            }
            let builder = TuringMachineBuilder {
                name: name.to_string(),
                init_state: None,
                accepted_state: Vec::new(),
                code: Vec::new(),
                input: None,
            };
            Ok(builder)
        }
        pub fn build(&self) -> Result<TuringMachineSet, TuringMachineBuilderError> {
            let init_state = if let Some(state) = self.init_state.clone() {
                state
            } else {
                return Err(TuringMachineBuilderError::InitialStateIsEmpty);
            };
            let code = self.code.clone();
            let machine: TuringMachineDefinition =
                match TuringMachineDefinition::new(init_state, self.accepted_state.clone(), code) {
                    Ok(machine) => machine,
                    Err(err) => {
                        if err == "Code contains accepted state" {
                            return Err(TuringMachineBuilderError::CodeContainsAcceptedState);
                        } else {
                            unreachable!();
                        }
                    }
                };
            let input_tape = self
                .input
                .clone()
                .ok_or(TuringMachineBuilderError::InputTapeIsEmpty)?;
            let machine = TuringMachineSet::new(machine, input_tape);
            Ok(machine)
        }

        pub fn input(&mut self, input: Tape) -> &mut Self {
            self.input = Some(input);
            self
        }

        pub fn init_state(&mut self, state: State) -> &mut Self {
            self.init_state = Some(state);
            self
        }

        pub fn get_init_state(&self) -> Option<State> {
            self.init_state.to_owned()
        }

        pub fn accepted_state(&mut self, states: impl IntoIterator<Item = State>) -> &mut Self {
            self.accepted_state = states.into_iter().collect();
            self
        }

        pub fn get_accepted_state(&self) -> Vec<State> {
            self.accepted_state.clone()
        }

        pub fn code_from_entries(
            &mut self,
            entries: impl IntoIterator<Item = CodeEntry>,
        ) -> &mut Self {
            self.code = entries.into_iter().collect();
            self
        }

        pub fn code_new(&mut self, vec: Vec<CodeEntry>) -> &mut Self {
            self.code = vec;
            self
        }

        pub fn code_push(&mut self, entry: CodeEntry) -> &mut Self {
            self.code.push(entry);
            self
        }

        pub fn code_refresh(&mut self) -> &mut Self {
            self.code = Vec::new();
            self
        }

        pub fn get_code(&self) -> Vec<CodeEntry> {
            self.code.to_owned()
        }

        pub fn get_name(&self) -> String {
            self.name.to_owned()
        }

        pub fn get_signs(&self) -> Vec<Sign> {
            self.get_code()
                .iter()
                .flat_map(|(key, value)| vec![key.0.clone(), value.0.clone()])
                .collect()
        }

        pub fn from_source(&mut self, str: &str) -> Result<&mut Self, TuringMachineBuilderError> {
            let mut lines = str.lines();
            if let Some(str) = lines.next() {
                if let Ok(state) = State::try_from(str) {
                    self.init_state(state);
                } else {
                    return Err(TuringMachineBuilderError::OnParseInitialState);
                }
            } else {
                return Err(TuringMachineBuilderError::OnParseInitialState);
            }
            if let Some(str) = lines.next() {
                let res: Vec<State> = str
                    .split_whitespace()
                    .map(State::try_from)
                    .collect::<Result<_, _>>()
                    .map_err(|_| TuringMachineBuilderError::OnParseAcceptedStates)?;
                self.accepted_state(res);
            }
            let code = parse_code(str)?;
            self.code = code;
            Ok(self)
        }
    }
}

pub mod graph_compose {
    use super::{
        builder::{TuringMachineBuilder, TuringMachineBuilderError},
        *,
    };

    pub struct GraphOfBuilder {
        pub name: String,
        pub init_state: State,
        pub assign_vertex_to_builder: Vec<TuringMachineBuilder>,
        pub assign_edge_to_state: Vec<((usize, usize), State)>,
        pub acceptable: Vec<Vec<State>>,
    }

    #[derive(Debug, Clone)]
    pub enum GraphOfBuilderError {
        BuilderError(TuringMachineBuilderError),
        InitialStateIsNotSetted(usize),
        LengthOfVertexAndAcceptableIsDifferent,
        EdgeIndexOut(usize, usize, usize, State),
    }

    impl From<TuringMachineBuilderError> for GraphOfBuilderError {
        fn from(value: TuringMachineBuilderError) -> Self {
            GraphOfBuilderError::BuilderError(value)
        }
    }

    pub fn builder_composition(
        graph: GraphOfBuilder,
    ) -> Result<TuringMachineBuilder, GraphOfBuilderError> {
        let GraphOfBuilder {
            name,
            init_state,
            assign_vertex_to_builder,
            assign_edge_to_state,
            acceptable,
        } = graph;
        let mut builder = TuringMachineBuilder::new(&name)?;

        for (index, builder) in assign_vertex_to_builder.iter().enumerate() {
            if builder.get_init_state().is_none() {
                return Err(GraphOfBuilderError::InitialStateIsNotSetted(index));
            }
        }

        if assign_vertex_to_builder.len() != acceptable.len() {
            return Err(GraphOfBuilderError::LengthOfVertexAndAcceptableIsDifferent);
        }

        for ((i1, i2), state) in &assign_edge_to_state {
            let num_vertex = assign_vertex_to_builder.len();
            if num_vertex <= *i1 || num_vertex <= *i2 {
                return Err(GraphOfBuilderError::EdgeIndexOut(
                    num_vertex,
                    *i1,
                    *i2,
                    state.clone(),
                ));
            }
        }

        let format_name = |index: usize, state: State| {
            State::try_from(
                format!(
                    "{index}-{}-{state}",
                    assign_vertex_to_builder[index].get_name()
                )
                .as_ref(),
            )
            .map_err(|_| ())
            .unwrap()
        };

        builder.init_state(init_state.clone());

        let all_sign: HashSet<Sign> = assign_vertex_to_builder
            .iter()
            .flat_map(|builder| builder.get_signs())
            .collect();

        let code = {
            let mut code: Vec<CodeEntry> = all_sign
                .iter()
                .map(|sign| {
                    (
                        (sign.clone(), init_state.clone()),
                        (
                            sign.clone(),
                            format_name(0, assign_vertex_to_builder[0].get_init_state().unwrap()),
                            Direction::Constant,
                        ),
                    )
                })
                .collect();

            let iter = assign_vertex_to_builder
                .iter()
                .enumerate()
                .flat_map(|(index, builder)| {
                    builder.get_code().into_iter().map(move |(key, value)| {
                        let new_key_state: State = format_name(index, key.1);
                        let new_value_state: State = format_name(index, value.1);
                        ((key.0, new_key_state), (value.0, new_value_state, value.2))
                    })
                });
            code.extend(iter);

            let iter = assign_edge_to_state
                .iter()
                .flat_map(|((index1, index2), state)| {
                    let init_state2 = assign_vertex_to_builder[*index2].get_init_state().unwrap();
                    all_sign
                        .iter()
                        .map(|sign| {
                            (
                                (sign.clone(), format_name(*index1, state.clone())),
                                (
                                    sign.clone(),
                                    format_name(*index2, init_state2.clone()),
                                    Direction::Constant,
                                ),
                            )
                        })
                        .collect::<Vec<_>>()
                });
            code.extend(iter);

            let iter = acceptable.iter().enumerate().flat_map(|(index, v)| {
                all_sign.iter().flat_map(move |sign| {
                    v.iter().map(move |state| {
                        (
                            (sign.clone(), format_name(index, state.clone())),
                            (sign.clone(), state.clone(), Direction::Constant),
                        )
                    })
                })
            });
            code.extend(iter);

            code
        };

        builder
            .accepted_state(acceptable.into_iter().flatten())
            .code_new(code);
        Ok(builder)
    }
}

#[cfg(test)]
mod tests {
    use crate::machine::State;

    use super::{builder, code};

    static CODE_STR: &str = "
        -, start, -, end, C
        ";

    #[test]
    fn parse_code() {
        let code = code::parse_code(CODE_STR);
        assert!(code.is_ok());
    }
    #[test]
    fn builder() {
        let mut builder = builder::TuringMachineBuilder::new("test").unwrap();
        assert!(builder.build().is_err());

        let start_state = State::try_from("start").unwrap();
        builder.init_state(start_state.clone());
        assert_eq!(builder.get_init_state(), Some(start_state));

        let accepted_state = vec![State::try_from("end").unwrap()];
        builder.accepted_state(accepted_state);
    }
}
