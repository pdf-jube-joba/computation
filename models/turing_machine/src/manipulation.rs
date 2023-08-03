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
    pub fn parse_tape(left: &str, head: &str, right: &str) -> Result<TapeAsVec, String> {
        let mut left = left
            .split_whitespace()
            .map(Sign::try_from)
            .collect::<Result<Vec<_>, _>>()?;
        left.reverse();
        let head = Sign::try_from(head)?;
        let right = right
            .split_whitespace()
            .map(Sign::try_from)
            .collect::<Result<_, _>>()?;
        Ok(TapeAsVec { left, head, right })
    }

    pub fn string_split_by_bar_interpretation() -> Interpretation<String, String> {
        fn write(str: String) -> Result<TapeAsVec, String> {
            let a = str.split('|').collect::<Vec<&str>>();
            if a.len() != 3 {
                return Err("length mismatch".to_string());
            }
            parse_tape(a[0], a[1], a[2])
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
            parse_tape(a[0], a[1], a[2])
        }
        fn read(tape: TapeAsVec) -> Result<String, String> {
            Ok(format!("{tape:?}"))
        }

        Interpretation { write, read }
    }
}

// コードを書くのは解釈と合わせて使われるべきだがパースを行う部分だけは別とする。
pub mod code {
    use crate::machine::CodeEntry;

    // 空の行、 "," を含んでない行はコメントとみなす。
    pub fn parse_code(code: &str) -> Result<Vec<CodeEntry>, String> {
        let mut vec = Vec::new();
        for entry in code
            .lines()
            .filter(|line| !line.is_empty() && line.contains(","))
            .map(CodeEntry::try_from)
        {
            vec.push(entry?)
        }
        Ok(vec)
    }
}

pub mod builder {
    use crate::machine::*;

    use super::code::parse_code;

    #[derive(Clone, PartialEq)]
    pub struct TuringMachineBuilder {
        name: String,
        init_state: Option<State>,
        accepted_state: Vec<State>,
        code: Vec<CodeEntry>,
        input: Option<TapeAsVec>,
    }

    impl TuringMachineBuilder {
        pub fn new(name: &str) -> Result<TuringMachineBuilder, String> {
            if name.is_empty() {
                return Err("empty string".to_string());
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
        pub fn build(&self) -> Result<TuringMachineSet, String> {
            let init_state = if let Some(state) = self.init_state.clone() {
                state
            } else {
                return Err("fail on initial state".to_string());
            };
            let code = self.code.clone();
            let machine: TuringMachine = if let Ok(machine) =
                TuringMachine::new(init_state, self.accepted_state.clone(), code)
            {
                machine
            } else {
                return Err("machine is not well-defined".to_string());
            };
            let input_tape = self.input.clone().ok_or("input not found".to_string())?;
            let machine = TuringMachineSet::new(machine, input_tape);
            // let run = RunningTuringMachine {
            //     machine,
            //     input,
            //     on_terminate: read ,
            // };
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

        pub fn from_source(&mut self, str: &str) -> Result<&mut Self, ()> {
            let mut lines = str.lines();
            if let Some(str) = lines.next() {
                if let Ok(state) = State::try_from(str) {
                    self.init_state(state);
                } else {
                    return Err(());
                }
            } else {
                return Err(());
            }
            if let Some(str) = lines.next() {
                let res: Vec<State> = str.split_whitespace()
                    .map(|str| State::try_from(str).map_err(|_| ()))
                    .collect::<Result<_, _>>()?;
                self.accepted_state(res);
            }
            let code = parse_code(str).map_err(|_| ())?;
            self.code = code;
            Ok(self)
        }

    }
}

pub mod graph_compose {
    use super::{builder::TuringMachineBuilder, *};
    use std::collections::HashMap;
    pub struct GraphOfMachine {
        // number_of_vertex: usize,
        // edge: Vec<(usize, usize)>,
        pub assign_vertex_to_machine: Vec<crate::machine::TuringMachine>,
        pub assign_edge_to_state: HashMap<(usize, usize), State>,
    }

    pub fn naive_composition(graph: GraphOfMachine) -> Result<TuringMachine, ()> {
        let GraphOfMachine {
            // edge,
            assign_vertex_to_machine,
            assign_edge_to_state,
        } = graph;
        let init_state: State = assign_vertex_to_machine[0].init_state().clone();
        let remove_state: HashSet<State> =
            assign_edge_to_state.values().into_iter().cloned().collect();
        let accepted_state: Vec<State> = assign_vertex_to_machine
            .iter()
            .flat_map(|machine| {
                machine
                    .accepted_state()
                    .iter()
                    .filter(|state| !remove_state.contains(*state))
            })
            .cloned()
            .collect();
        let mut code: Vec<CodeEntry> = assign_vertex_to_machine
            .iter()
            .flat_map(|machine| machine.code())
            .cloned()
            .collect();
        let connect_code: Vec<CodeEntry> = assign_edge_to_state
            .into_iter()
            .flat_map(|((start, end), state)| {
                let connect_start_state = state;
                let connect_end_state = assign_vertex_to_machine[end].init_state().clone();
                assign_vertex_to_machine[start]
                    .signs()
                    .iter()
                    .map(|sign| {
                        let entry = CodeEntry::from_tuple(
                            sign.clone(),
                            connect_start_state.clone(),
                            sign.clone(),
                            connect_end_state.clone(),
                            Direction::Constant,
                        );
                        entry
                    })
                    .collect::<Vec<_>>()
            })
            .collect();
        code.extend(connect_code);
        TuringMachine::new(init_state, accepted_state, code)
    }

    pub fn checked_composition(graph: GraphOfMachine) -> Result<TuringMachine, ()> {
        let GraphOfMachine {
            // edge,
            assign_vertex_to_machine,
            assign_edge_to_state: _,
        } = &graph;
        let mut states: HashSet<State> = HashSet::new();
        for machine in assign_vertex_to_machine {
            let v_st = machine.states();
            for state in &v_st {
                if states.contains(&state) {
                    return Err(());
                }
            }
            states.extend(v_st);
        }
        naive_composition(graph)
    }

    // to compose builders on graph which has same type of input and output
    pub struct GraphOfBuilder {
        pub name: String,
        pub init_state: State,
        pub assign_vertex_to_builder: Vec<TuringMachineBuilder>,
        pub assign_edge_to_state: HashMap<(usize, usize), State>,
        pub acceptable: Vec<Vec<State>>,
    }
    // TODO 終了状態の名前を指定できた方がよい。
    pub fn naive_builder_composition(
        graph: GraphOfBuilder,
    ) -> Result<TuringMachineBuilder, String> {
        let GraphOfBuilder {
            name,
            init_state,
            assign_vertex_to_builder,
            assign_edge_to_state,
            acceptable,
        } = graph;
        let mut builder = TuringMachineBuilder::new(&name).unwrap();

        // builder に initial state があることが前提。
        for (index, builder) in assign_vertex_to_builder.iter().enumerate() {
            if builder.get_init_state().is_none() {
                return Err(format!("no initial state in builder {index}"));
            }
        }

        // vertex の数と acceptable の数が一致することが前提
        if assign_vertex_to_builder.len() != acceptable.len() {
            return Err(format!("length of vertex and accpetable is different"));
        }

        // edge に出てくる数が vertex を超えないことが前提
        for ((i1, i2), state) in &assign_edge_to_state {
            let num_vertex = assign_vertex_to_builder.len();
            if num_vertex <= *i1 || num_vertex <= *i2 {
                return Err(format!("edge's index out...num: {num_vertex}, edge: {i1} {i2} {state}"));
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
            .unwrap()
        };

        // builder.init_state(assign_vertex_to_builder[0].get_init_state().unwrap());
        builder.init_state(init_state.clone());

        let all_sign: Vec<Sign> = assign_vertex_to_builder
            .iter()
            .flat_map(|builder| builder.get_signs())
            .collect();

        let code = {
            // let mut code = vec![];

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

            let iter = acceptable.iter().enumerate().flat_map(|(index, v)|{
                all_sign
                    .iter()
                    .flat_map(move |sign|{
                        v.iter().map(move |state|{
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
        // let accepted_state: Vec<State> = {
        //     let mut vec: HashSet<State> = HashSet::new();
        //     for builder in assign_vertex_to_builder {
        //         vec.extend(builder.get_accepted_state())
        //     }
        //     for edge in assign_edge_to_state {
        //         vec.remove(&edge.1);
        //     }
        //     vec.into_iter().collect()
        // };
        builder.accepted_state(acceptable.into_iter().flatten()).code_new(code);
        Ok(builder)
    }
}

pub mod compose_diff_type {
    use super::{builder::TuringMachineBuilder, *};
    pub fn compose_builder(
        first: TuringMachineBuilder,
        specified_state: &State,
        second: TuringMachineBuilder,
    ) -> Result<TuringMachineBuilder, String> {
        let first_name = first.get_name();
        let second_name = second.get_name();
        let name = format!("{first_name}-{second_name}");

        let first_state_conversion =
            |state: &State| State::try_from(format!("0-{first_name}-{state}").as_ref()).unwrap();
        let second_state_conversion =
            |state: &State| State::try_from(format!("1-{second_name}-{state}").as_ref()).unwrap();

        let accepted_state = {
            if !first.get_accepted_state().contains(&specified_state) {
                return Err("state is not in terminate states".to_string());
            }
            let mut accepted_state = Vec::new();
            accepted_state.extend(
                first
                    .get_accepted_state()
                    .into_iter()
                    .filter(|state| state != specified_state),
            );
            accepted_state.extend(second.get_accepted_state());
            accepted_state
        };

        let code = {
            let mut code = Vec::new();
            code.extend(first.get_code().into_iter().map(|entry| {
                let new_key_state = first_state_conversion(&entry.key_state());
                let new_value_state = first_state_conversion(&entry.value_state());
                CodeEntry::from_tuple(
                    entry.key_sign(),
                    new_key_state,
                    entry.value_sign(),
                    new_value_state,
                    entry.value_direction(),
                )
            }));
            code.extend(second.get_code().into_iter().map(|entry| {
                let new_key_state = second_state_conversion(&entry.key_state());
                let new_value_state = second_state_conversion(&entry.value_state());
                CodeEntry::from_tuple(
                    entry.key_sign(),
                    new_key_state,
                    entry.value_sign(),
                    new_value_state,
                    entry.value_direction(),
                )
            }));

            let mut used_sign = Vec::new();
            used_sign.extend(first.get_signs());
            used_sign.extend(second.get_signs());

            let second_init_state = if let Some(state) = second.get_init_state() {
                state
            } else {
                return Err("second builder does not have init state".to_string());
            };

            code.extend(used_sign.into_iter().map(|sign| {
                CodeEntry::from_tuple(
                    sign.clone(),
                    first_state_conversion(&specified_state),
                    sign,
                    second_state_conversion(&second_init_state),
                    Direction::Constant,
                )
            }));
            code
        };

        let mut builder = TuringMachineBuilder::new(&name).unwrap();
        if let Some(init_state) = first.get_init_state() {
            builder.init_state(init_state);
        };

        builder
            .accepted_state(accepted_state)
            .code_from_entries(code);

        Ok::<TuringMachineBuilder, String>(builder)
    }
}

pub mod machine_as_function {
    // pub struct
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
