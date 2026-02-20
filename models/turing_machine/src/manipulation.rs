use crate::machine::*;
use std::collections::HashSet;

pub mod builder {
    use crate::machine::*;
    use anyhow::{Result, anyhow};

    #[derive(Clone, PartialEq)]
    pub struct UserCodeEntry<SignT> {
        pub key_sign: SignT,
        pub key_state: State,
        pub value_sign: SignT,
        pub value_state: State,
        pub direction: Direction,
    }

    impl<SignT> From<UserCodeEntry<SignT>> for CodeEntry
    where
        SignT: Into<Sign>,
    {
        fn from(entry: UserCodeEntry<SignT>) -> Self {
            (
                (entry.key_sign.into(), entry.key_state),
                (entry.value_sign.into(), entry.value_state, entry.direction),
            )
        }
    }

    impl<SignT> From<(SignT, State, SignT, State, Direction)> for UserCodeEntry<SignT> {
        fn from(entry: (SignT, State, SignT, State, Direction)) -> Self {
            UserCodeEntry {
                key_sign: entry.0,
                key_state: entry.1,
                value_sign: entry.2,
                value_state: entry.3,
                direction: entry.4,
            }
        }
    }

    impl From<CodeEntry> for UserCodeEntry<Sign> {
        fn from(entry: CodeEntry) -> Self {
            UserCodeEntry {
                key_sign: entry.0.0,
                key_state: entry.0.1,
                value_sign: entry.1.0,
                value_state: entry.1.1,
                direction: entry.1.2,
            }
        }
    }

    #[derive(Clone, PartialEq)]
    pub struct TuringMachineBuilder<SignT = Sign> {
        pub name: String,
        pub init_state: State,
        pub accepted_state: Vec<State>,
        pub code: Vec<UserCodeEntry<SignT>>,
    }

    impl<SignT> TuringMachineBuilder<SignT>
    where
        SignT: Clone + Eq + std::hash::Hash + Into<Sign>,
    {
        pub fn new(name: &str, init_state: State) -> Result<TuringMachineBuilder<SignT>> {
            if name.is_empty() {
                return Err(anyhow!("Name is empty"));
            }
            Ok(TuringMachineBuilder {
                name: name.to_string(),
                init_state,
                accepted_state: Vec::new(),
                code: Vec::new(),
            })
        }

        pub fn build(&self, tape: Tape) -> Result<TuringMachine> {
            let code = self
                .code
                .iter()
                .cloned()
                .map(CodeEntry::from)
                .collect::<Vec<_>>();
            let machine = TuringMachineDefinition::new(
                self.init_state.clone(),
                self.accepted_state.clone(),
                code,
            )?;
            Ok(TuringMachine::new(machine, tape))
        }
    }
}

pub mod graph_compose {
    use super::{builder::TuringMachineBuilder, builder::UserCodeEntry, *};
    use anyhow::{Result, anyhow};
    use utils::{TextCodec, parse::ParseTextCodec};

    pub struct GraphOfBuilder<SignT = Sign> {
        pub name: String,
        pub init_state: State,
        pub assign_vertex_to_builder: Vec<TuringMachineBuilder<SignT>>,
        pub assign_edge_to_state: Vec<((usize, usize), State)>,
        pub acceptable: Vec<Vec<State>>,
    }

    pub fn builder_composition<SignT>(
        graph: GraphOfBuilder<SignT>,
    ) -> Result<TuringMachineBuilder<SignT>>
    where
        SignT: Clone + Eq + std::hash::Hash + Into<Sign>,
    {
        let GraphOfBuilder {
            name,
            init_state,
            assign_vertex_to_builder,
            assign_edge_to_state,
            acceptable,
        } = graph;

        let mut builder = TuringMachineBuilder::new(&name, init_state.clone())?;

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

        let format_name = |index: usize, state: State| -> Result<State> {
            let name = format!(
                "v{index}-{}-{}",
                assign_vertex_to_builder[index].name,
                state.print()
            );
            name.parse_tc().map_err(|err| anyhow!("{err}"))
        };

        let all_sign: HashSet<SignT> = assign_vertex_to_builder
            .iter()
            .flat_map(|builder| {
                builder
                    .code
                    .iter()
                    .flat_map(|entry| vec![entry.key_sign.clone(), entry.value_sign.clone()])
            })
            .collect();

        let make_constant_entries = |from_state: &State, to_state: &State| {
            all_sign
                .iter()
                .cloned()
                .map(|sign| UserCodeEntry {
                    key_sign: sign.clone(),
                    key_state: from_state.clone(),
                    value_sign: sign,
                    value_state: to_state.clone(),
                    direction: Direction::Constant,
                })
                .collect::<Vec<_>>()
        };

        let mut code: Vec<UserCodeEntry<SignT>> = Vec::new();
        let first_state = format_name(0, assign_vertex_to_builder[0].init_state.clone())?;
        code.extend(make_constant_entries(&init_state, &first_state));

        for (index, builder) in assign_vertex_to_builder.iter().enumerate() {
            for entry in builder.code.clone() {
                let new_key_state = format_name(index, entry.key_state)?;
                let new_value_state = format_name(index, entry.value_state)?;
                code.push(UserCodeEntry {
                    key_sign: entry.key_sign,
                    key_state: new_key_state,
                    value_sign: entry.value_sign,
                    value_state: new_value_state,
                    direction: entry.direction,
                });
            }
        }

        for ((index1, index2), state) in &assign_edge_to_state {
            let init_state2 = assign_vertex_to_builder[*index2].init_state.clone();
            let from_state = format_name(*index1, state.clone())?;
            let to_state = format_name(*index2, init_state2)?;
            code.extend(make_constant_entries(&from_state, &to_state));
        }

        for (index, states) in acceptable.iter().enumerate() {
            for state in states {
                let from_state = format_name(index, state.clone())?;
                code.extend(make_constant_entries(&from_state, state));
            }
        }

        builder
            .accepted_state
            .extend(acceptable.into_iter().flatten());
        builder.code = code;
        Ok(builder)
    }
}
