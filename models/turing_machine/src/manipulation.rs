use crate::machine::*;
use std::collections::HashSet;

pub mod code {
    use crate::machine::{CodeEntry, State, TuringMachineDefinition};
    use anyhow::Result;

    pub fn parse_one_code_entry(code: &str) -> Result<CodeEntry> {
        let v: Vec<_> = code.split(',').collect();
        if v.len() < 5 {
            anyhow::bail!("Invalid code entry: {}", code);
        }
        // .trim() で parse 用に成形する
        Ok((
            (v[0].trim().parse()?, v[1].trim().parse()?),
            (
                v[2].trim().parse()?,
                v[3].trim().parse()?,
                v[4].trim().try_into()?,
            ),
        ))
    }

    pub fn parse_code(code: &str) -> Result<Vec<CodeEntry>> {
        let vec: Vec<CodeEntry> = code
            .lines()
            .enumerate()
            .filter(|(_, line)| !line.is_empty() && !line.starts_with('#') && line.contains(","))
            .map(|(index, line)| match parse_one_code_entry(line) {
                Ok(entry) => Ok(entry),
                Err(err) => {
                    anyhow::bail!("Error parsing code entry at line {}: {}", index + 1, err)
                }
            })
            .collect::<Result<Vec<CodeEntry>>>()?;
        Ok(vec)
    }

    pub fn parse_definition(code: &str) -> Result<TuringMachineDefinition> {
        // get init state from first line
        let mut lines = code.lines();

        let Some(init_state_line) = lines.next() else {
            anyhow::bail!("Missing initial state line")
        };

        let init_state: State = init_state_line.trim().parse()?;

        let Some(accepted_state_line) = lines.next() else {
            anyhow::bail!("Missing accepted states line")
        };

        let accepted_state: Vec<State> = accepted_state_line
            .split(',')
            .map(|s| s.trim().parse())
            .collect::<Result<_>>()?;

        let code: Vec<_> = lines
            .enumerate()
            .filter(|(_, line)| !line.is_empty() && !line.starts_with('#') && line.contains(","))
            .map(|(index, line)| {
                parse_one_code_entry(line).map_err(|err| {
                    anyhow::anyhow!("Error parsing code entry at line {}: {}", index + 1, err)
                })
            })
            .collect::<Result<_>>()?;

        TuringMachineDefinition::new(init_state, accepted_state, code)
    }
}

pub mod builder {
    use crate::machine::*;
    use anyhow::{anyhow, Result};

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
            let definition: TuringMachineDefinition = super::code::parse_definition(str)?;
            self.init_state(definition.init_state().clone());
            self.accepted_state(definition.accepted_state().clone());
            self.code = definition.code().clone();
            Ok(self)
        }
    }
}

pub mod graph_compose {
    use super::{builder::TuringMachineBuilder, *};
    use anyhow::{anyhow, Result};

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
                "{index}-{}-{state}",
                assign_vertex_to_builder[index].get_name()
            );
            str.as_str().parse::<State>().map_err(|_| ()).unwrap()
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
    use super::{builder, code};
    use crate::machine::State;

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

        let start_state: State = "start".parse().unwrap();
        builder.init_state(start_state.clone());
        assert_eq!(builder.get_init_state(), Some(start_state));

        let accepted_state: Vec<State> = vec!["end".parse().unwrap()];
        builder.accepted_state(accepted_state);
    }
}
