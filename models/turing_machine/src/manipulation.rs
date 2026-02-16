use crate::machine::*;
use std::collections::HashSet;

pub mod builder {
    use crate::machine::*;
    use anyhow::{Result, anyhow};
    use utils::parse::ParseTextCodec;

    #[derive(Clone, PartialEq)]
    pub struct TuringMachineBuilder {
        name: String,
        init_state: Option<State>,
        accepted_state: Vec<State>,
        code: Vec<CodeEntry>,
    }

    impl TuringMachineBuilder {
        pub fn new(name: &str) -> Result<TuringMachineBuilder> {
            if name.is_empty() {
                return Err(anyhow!("Name is empty"));
            }
            Ok(TuringMachineBuilder {
                name: name.to_string(),
                init_state: None,
                accepted_state: Vec::new(),
                code: Vec::new(),
            })
        }

        pub fn build(&self, tape: Tape) -> Result<TuringMachineSet> {
            let init_state = self
                .init_state
                .clone()
                .ok_or_else(|| anyhow!("Initial state is empty"))?;
            let code = self.code.clone();
            let machine =
                TuringMachineDefinition::new(init_state, self.accepted_state.clone(), code)?;
            Ok(TuringMachineSet::new(machine, tape))
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

        pub fn from_source(&mut self, str: &str) -> Result<&mut Self> {
            let definition: TuringMachineDefinition = str.parse_tc().map_err(|e| anyhow!(e))?;
            self.init_state(definition.init_state().clone());
            self.accepted_state(definition.accepted_state().clone());
            self.code = definition.code().clone();
            Ok(self)
        }
    }
}

pub mod graph_compose {
    use super::{builder::TuringMachineBuilder, *};
    use anyhow::{Result, anyhow};
    use utils::{TextCodec, parse::ParseTextCodec};

    pub struct GraphOfBuilder {
        pub name: String,
        pub init_state: State,
        pub assign_vertex_to_builder: Vec<TuringMachineBuilder>,
        pub assign_edge_to_state: Vec<((usize, usize), State)>,
        pub acceptable: Vec<Vec<State>>,
    }

    pub fn builder_composition(graph: GraphOfBuilder) -> Result<TuringMachineBuilder> {
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
                return Err(anyhow!("Initial state is not set for vertex {index}"));
            }
        }

        if assign_vertex_to_builder.len() != acceptable.len() {
            return Err(anyhow!("Length of vertices and acceptable states differ"));
        }

        for ((i1, i2), state) in &assign_edge_to_state {
            let num_vertex = assign_vertex_to_builder.len();
            if num_vertex <= *i1 || num_vertex <= *i2 {
                return Err(anyhow!(
                    "Edge index out of bounds: {num_vertex}, {i1}, {i2}, {state:?}"
                ));
            }
        }

        let format_name = |index: usize, state: State| {
            let str = format!(
                "v{index}-{}-{}",
                assign_vertex_to_builder[index].get_name(),
                state.print()
            );
            str.parse_tc().unwrap()
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
