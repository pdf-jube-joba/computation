use crate::machine::*;
use std::collections::HashSet;

pub mod tape {
    use crate::machine::*;

    // tape の書き込み、読み込みを行うための構造体
    // 入力と出力それぞれの型と読み書きを行うために使う関数を格納することで
    // "テープに対する解釈"をひとまとめに扱う。
    // マシンの不可解な合成を行うことを型レベルで防ぐなどを期待する。
    #[derive(Clone)]
    pub struct Interpretation<Input, Output>
where
    // Input: Clone,
    // Output: Clone,
    {
        pub write: fn(Input) -> Result<TapeAsVec, String>,
        pub read: fn(TapeAsVec) -> Result<Output, String>,
    }

    impl<Input, Output> Interpretation<Input, Output>
    where
        Input: Clone,
        Output: Clone,
    {
        pub fn new(
            write: fn(Input) -> Result<TapeAsVec, String>,
            read: fn(TapeAsVec) -> Result<Output, String>,
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

    // tape それ自身を入力と出力と解釈するための Interpretation
    pub fn standard_interpretation() -> Interpretation<TapeAsVec, TapeAsVec> {
        fn write_std(tape: TapeAsVec) -> Result<TapeAsVec, String> {
            Ok(tape)
        }
        fn read_std(tape: TapeAsVec) -> Result<TapeAsVec, String> {
            Ok(tape)
        }
        Interpretation {
            write: write_std,
            read: read_std,
        }
    }

    // left head right に対応する文字列をもとに tape を作る。
    pub fn parse_tape(left: &str, head: &str, right: &str) -> Result<TapeAsVec, TapeParseError> {
        TapeAsVec::try_from((left, head, right))
    }

    pub fn string_split_by_bar_interpretation() -> Interpretation<String, String> {
        fn write(str: String) -> Result<TapeAsVec, String> {
            let a = str.split('|').collect::<Vec<&str>>();
            if a.len() != 3 {
                return Err("length mismatch".to_string());
            }
            parse_tape(a[0], a[1], a[2]).map_err(|err| format!("{err:?}"))
        }
        fn read(tape: TapeAsVec) -> Result<String, String> {
            Ok(format!("{tape:?}"))
        }

        Interpretation { write, read }
    }

    pub fn string_split_by_line_interpretation() -> Interpretation<String, String> {
        fn write(str: String) -> Result<TapeAsVec, String> {
            let a = str.lines().collect::<Vec<&str>>();
            if a.len() != 3 {
                return Err("length mismatch".to_string());
            }
            parse_tape(a[0], a[1], a[2]).map_err(|err| format!("{err:?}"))
        }
        fn read(tape: TapeAsVec) -> Result<String, String> {
            Ok(format!("{tape:?}"))
        }

        Interpretation { write, read }
    }
}

// コードを書くのは解釈と合わせて使われるべきだがパースを行う部分だけは別とする。
pub mod code {
    use crate::machine::{CodeEntry, ParseCodeEntryError};

    // 空の行、 "," を含んでない行はコメントとみなす。
    pub fn parse_code(code: &str) -> Result<Vec<CodeEntry>, (usize, ParseCodeEntryError)> {
        let vec: Vec<CodeEntry> = code
            .lines()
            .enumerate()
            .filter(|(_, line)| !line.is_empty() && line.contains(','))
            .map(|(index, line)| CodeEntry::try_from(line).map_err(|err| (index, err)))
            .collect::<Result<Vec<CodeEntry>, (usize, ParseCodeEntryError)>>()?;
        Ok(vec)
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
        OnParseCode(usize, ParseCodeEntryError),
        CodeContainsAcceptedState,
    }

    impl From<(usize, ParseCodeEntryError)> for TuringMachineBuilderError {
        fn from((index, err): (usize, ParseCodeEntryError)) -> Self {
            TuringMachineBuilderError::OnParseCode(index, err)
        }
    }

    #[derive(Clone, PartialEq)]
    pub struct TuringMachineBuilder {
        name: String,
        init_state: Option<State>,
        accepted_state: Vec<State>,
        code: Vec<CodeEntry>,
        input: Option<TapeAsVec>,
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
            let machine: TuringMachine =
                match TuringMachine::new(init_state, self.accepted_state.clone(), code) {
                    Ok(machine) => machine,
                    Err(TuringMachineError::CodeContainsAcceptedState) => {
                        return Err(TuringMachineBuilderError::CodeContainsAcceptedState);
                    }
                };
            let input_tape = self
                .input
                .clone()
                .ok_or(TuringMachineBuilderError::InputTapeIsEmpty)?;
            let machine = TuringMachineSet::new(machine, input_tape);
            Ok(machine)
        }

        pub fn input(&mut self, input: TapeAsVec) -> &mut Self {
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
                .flat_map(|entry| vec![entry.key_sign(), entry.value_sign()])
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
                    .map_err(|err| match err {
                        StateParseError::Error => TuringMachineBuilderError::OnParseAcceptedStates,
                    })?;
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

    // to compose builders on graph which has same type of input and output
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

        // builder に initial state があることが前提。
        for (index, builder) in assign_vertex_to_builder.iter().enumerate() {
            if builder.get_init_state().is_none() {
                return Err(GraphOfBuilderError::InitialStateIsNotSetted(index));
            }
        }

        // vertex の数と acceptable の数が一致することが前提
        if assign_vertex_to_builder.len() != acceptable.len() {
            return Err(GraphOfBuilderError::LengthOfVertexAndAcceptableIsDifferent);
        }

        // edge に出てくる数が vertex を超えないことが前提
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
            .unwrap() // should succeed
        };

        builder.init_state(init_state.clone());

        let all_sign: HashSet<Sign> = assign_vertex_to_builder
            .iter()
            .flat_map(|builder| builder.get_signs())
            .collect();

        let code = {
            // initial_state から builder[0] の initial state への移動
            let mut code: Vec<CodeEntry> = all_sign
                .iter()
                .map(|sign| {
                    CodeEntry::from_tuple(
                        sign.clone(),
                        init_state.clone(),
                        sign.clone(),
                        format_name(0, assign_vertex_to_builder[0].get_init_state().unwrap()), // 上で検査してるので安全
                        Direction::Constant,
                    )
                })
                .collect();

            // 各 builder の名前の付け替えを行いつつ新たにentryを作る。
            let iter = assign_vertex_to_builder
                .iter()
                .enumerate()
                .flat_map(|(index, builder)| {
                    builder.get_code().into_iter().map(move |entry| {
                        let new_key_state: State = format_name(index, entry.key_state());
                        let new_value_state: State = format_name(index, entry.value_state());
                        CodeEntry::from_tuple(
                            entry.key_sign(),
                            new_key_state,
                            entry.value_sign(),
                            new_value_state,
                            entry.value_direction(),
                        )
                    })
                });
            code.extend(iter);

            // 各 edge (i,j) ごとに builder[i] の E_{i,j} で指定された state から builder[j] の initial state への移行
            let iter = assign_edge_to_state
                .iter()
                .flat_map(|((index1, index2), state)| {
                    let init_state2 = assign_vertex_to_builder[*index2].get_init_state().unwrap(); // 上で検査してるので安全
                    all_sign
                        .iter()
                        .map(|sign| {
                            CodeEntry::from_tuple(
                                sign.clone(),
                                format_name(*index1, state.clone()),
                                sign.clone(),
                                format_name(*index2, init_state2.clone()),
                                Direction::Constant,
                            )
                        })
                        .collect::<Vec<_>>()
                });
            code.extend(iter);

            // accepted state として認められているものだけ名前を付け替えるように状態を変更する

            let iter = acceptable.iter().enumerate().flat_map(|(index, v)| {
                all_sign.iter().flat_map(move |sign| {
                    v.iter().map(move |state| {
                        CodeEntry::from_tuple(
                            sign.clone(),
                            format_name(index, state.clone()),
                            sign.clone(),
                            state.clone(),
                            Direction::Constant,
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
