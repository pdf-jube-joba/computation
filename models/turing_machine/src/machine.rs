use std::collections::{HashMap, HashSet, VecDeque};
use std::fmt::Display;
use yew::Properties;

// テープの動く方向を表す。
#[derive(Debug, Clone, PartialEq)]
pub enum Direction {
    Right,
    Constant,
    Left,
}

impl TryFrom<&str> for Direction {
    type Error = String;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let value = value.trim();
        match value {
            "R" => Ok(Direction::Right),
            "L" => Ok(Direction::Left),
            "C" => Ok(Direction::Constant),
            _ => Err("direction: fail".to_string()),
        }
    }
}

// テープで扱う記号の定義
// 空白記号（スペース）と','と制御記号の含まれない文字列を記号として扱う
// 空の文字列で記号としての空白記号を表す
#[derive(Debug, Default, Clone, PartialEq, Hash, Eq)]
pub struct Sign(String);

impl Sign {
    pub fn blank() -> Sign {
        Sign::try_from("").unwrap()
    }
}

impl Display for Sign {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl TryFrom<&str> for Sign {
    type Error = String;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let value = value.trim();
        if value.contains(|char: char| char.is_whitespace() || char == ',') {
            return Err(format!("error on parse sign: input...\"{value}\""));
        }
        Ok(Sign(value.to_string()))
    }
}

pub fn parse_str_to_signs<T>(str: &str) -> Result<T, String>
where
    T: std::iter::FromIterator<Sign>,
{
    str.split_whitespace().map(Sign::try_from).collect()
}

// 左右無限のテープ
// ヘッド部分の読み書きと左右への移動のみが許される
// これの中身を読みたい場合はコストを払って、TapeAsVec を使う
#[derive(Debug, Default, Clone, PartialEq, Properties, Hash, Eq)]
pub struct Tape {
    left: VecDeque<Sign>,
    head: Sign,
    right: VecDeque<Sign>,
}

// テープを簡単に見たり作ったりするための構造体
#[derive(Debug, Default, Clone, PartialEq)]
pub struct TapeAsVec {
    pub left: Vec<Sign>,
    pub head: Sign,
    pub right: Vec<Sign>,
}

impl TapeAsVec {
    pub fn new(
        left: impl IntoIterator<Item = Sign>,
        head: Sign,
        right: impl IntoIterator<Item = Sign>,
    ) -> Self {
        Self {
            left: left.into_iter().collect(),
            head,
            right: right.into_iter().collect(),
        }
    }
}

impl From<TapeAsVec> for Tape {
    fn from(TapeAsVec { left, head, right }: TapeAsVec) -> Self {
        Tape {
            left: left.into(),
            head,
            right: right.into(),
        }
    }
}

impl TryFrom<&str> for TapeAsVec {
    type Error = String;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let v: Vec<&str> = value.lines().collect();
        if v.len() < 3 {
            return Err("tape: argument is too few".to_owned());
        }
        let left: Vec<Sign> = parse_str_to_signs(v[0])?;
        let head: Sign = v[1].try_into()?;
        let right: Vec<Sign> = parse_str_to_signs(v[2])?;
        Ok(Self { left, head, right })
    }
}

impl TryFrom<String> for TapeAsVec {
    type Error = String;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        let v: Vec<&str> = value.lines().collect();
        if v.len() < 3 {
            return Err("tape: argument is too few".to_owned());
        }
        let left: Vec<Sign> = parse_str_to_signs(v[0])?;
        let head: Sign = v[1].try_into()?;
        let right: Vec<Sign> = parse_str_to_signs(v[2])?;
        Ok(Self { left, head, right })
    }
}

impl From<TapeAsVec> for String {
    fn from(value: TapeAsVec) -> Self {
        let f = |v: Vec<Sign>| {
            v.iter()
                .map(|Sign(sign)| sign.to_owned())
                .collect::<String>()
        };
        format!(
                "l: {} \nh: {}\n r: {}",
                f(value.left),
                value.head,
                f(value.right)
            )
    }
}

impl Tape {
    pub fn new(
        left: impl IntoIterator<Item = Sign>,
        head: Sign,
        right: impl IntoIterator<Item = Sign>,
    ) -> Self {
        Self {
            left: left.into_iter().collect(),
            head,
            right: right.into_iter().collect(),
        }
    }
    pub fn head_read(&self) -> &Sign {
        &self.head
    }
    fn head_write(&mut self, sign: &Sign) {
        self.head = sign.clone();
    }
    fn move_to(&mut self, m: &Direction) {
        match m {
            Direction::Left => {
                let next_head = self.left.pop_front().unwrap_or_default();
                let old_head = std::mem::replace(&mut self.head, next_head);
                self.right.push_front(old_head);
            }
            Direction::Right => {
                let next_head = self.right.pop_front().unwrap_or_default();
                let old_head = std::mem::replace(&mut self.head, next_head);
                self.left.push_front(old_head);
            }
            Direction::Constant => {}
        }
    }
    fn show(&self) -> TapeAsVec {
        TapeAsVec {
            left: self.left.iter().cloned().collect(),
            head: self.head.clone(),
            right: self.right.iter().cloned().collect(),
        }
    }
}

impl Display for Tape {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let left: String = self.left.iter().map(|sign| format!("{sign} ")).collect();
        writeln!(f, "l:{left}")?;
        writeln!(f, "h:{}", self.head)?;
        let right: String = self.right.iter().map(|sign| format!("{sign} ")).collect();
        writeln!(f, "r:{right}")
    }
}

// マシンの持つ状態の定義
// テープの記号と同じ
#[derive(Debug, Clone, PartialEq, Hash, Eq)]
pub struct State(String);
impl Display for State {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
impl TryFrom<&str> for State {
    type Error = String;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let value = value.trim();
        if value.contains(|char: char| char.is_whitespace() || char == ',') {
            return Err(format!("whitespace contained input:{value}"));
        }
        Ok(State(value.to_string()))
    }
}

#[derive(Debug, Clone, PartialEq, Hash, Eq)]
pub struct CodeKey(Sign, State);
#[derive(Debug, Clone, PartialEq)]
pub struct CodeValue(Sign, State, Direction);

#[derive(Debug, Clone, PartialEq)]
pub struct CodeEntry(CodeKey, CodeValue);

impl CodeEntry {
    pub fn key_sign(&self) -> Sign {
        self.0 .0.clone()
    }
    pub fn key_state(&self) -> State {
        self.0 .1.clone()
    }
    pub fn value_sign(&self) -> Sign {
        self.1 .0.clone()
    }
    pub fn value_state(&self) -> State {
        self.1 .1.clone()
    }
    pub fn value_direction(&self) -> Direction {
        self.1 .2.clone()
    }
    pub fn from_tuple(
        key_sign: Sign,
        key_state: State,
        value_sign: Sign,
        value_state: State,
        value_direction: Direction,
    ) -> Self {
        CodeEntry(
            CodeKey(key_sign, key_state),
            CodeValue(value_sign, value_state, value_direction),
        )
    }
}

impl TryFrom<&str> for CodeEntry {
    type Error = String;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        if value.contains(|char: char| char.is_control()) {
            return Err("code-entry: some control char contained".to_string());
        };
        let v: Vec<&str> = value.split(',').collect();
        if v.len() < 5 {
            return Err("code-entry: argument is too few".to_string());
        }
        let code_key: CodeKey = CodeKey(v[0].try_into()?, v[1].try_into()?);
        let code_value: CodeValue = CodeValue(v[2].try_into()?, v[3].try_into()?, v[4].try_into()?);
        Ok(CodeEntry(code_key, code_value))
    }
}

#[derive(Debug, Default, Clone, PartialEq, Properties)]
pub struct Code {
    hash: HashMap<CodeKey, CodeValue>,
}

impl Code {
    fn code(&self) -> &HashMap<CodeKey, CodeValue> {
        &self.hash
    }
    pub fn code_as_vec(&self) -> Vec<CodeEntry> {
        self.hash
            .iter()
            .map(|(k, v)| CodeEntry(k.clone(), v.clone()))
            .collect()
    }
    pub fn add(&mut self, CodeEntry(k, v): CodeEntry) {
        self.hash.insert(k, v);
    }
    pub fn from_iter_entry(iter: impl IntoIterator<Item = CodeEntry>) -> Self {
        Code {
            hash: HashMap::from_iter(iter.into_iter().map(|CodeEntry(k, v)| (k, v))),
        }
    }
}

impl TryFrom<&str> for Code {
    type Error = String;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let mut code = Code::default();
        for (index, str) in value.lines().enumerate() {
            match CodeEntry::try_from(str) {
                Ok(entry) => {
                    code.add(entry);
                }
                Err(err) => {
                    return Err(format!("{} at line {}", err, index));
                }
            }
        }
        Ok(code)
    }
}

// Turing machine は次のものから構成されている。
// Σ:有限集合...テープに用いる記号
// b:Σ...空白記号
// Q:有限集合...マシンの状態
// q_init:Q...マシンの初期状態
// F:subset of Q...マシンの受理状態全体
// δ:(Q,Σ) -> (Q,Σ,{E,R,C})...マシンの遷移関数
// ただし、実装上は次のように固定してしまう
// ΣやQはある無限集合（可能なマシンの用いうる記号や状態の集合）Sign, State の部分集合を（暗黙的に）指しているものとし、
// δを（有限な）HashMap<(Sign, State), (Sign, State, {L,R,C})> により実装することで、
// このHashMapに存在するSignやStateが「実は考えていたQやΣである」とする。
// また、マシンの停止は以下の二つの可能性があるものとする。
// - マシンの状態が accepted_state に含まれる。
// - 部分関数である遷移関数の定義域に含まれない。
#[derive(Debug, Clone, PartialEq)]
pub struct TuringMachine {
    init_state: State,
    accepted_state: HashSet<State>,
    code: Code,
}

impl TuringMachine {
    pub fn new(
        init_state: State,
        accepted_state: impl IntoIterator<Item = State>,
        code: impl IntoIterator<Item = CodeEntry>,
    ) -> Self {
        TuringMachine {
            init_state,
            accepted_state: accepted_state.into_iter().collect(),
            code: Code::from_iter_entry(code.into_iter()),
        }
    }
}

// TuringMachine の計算過程を表す。
//
#[derive(Debug, Clone, PartialEq)]
pub struct TuringMachineState {
    state: State,
    tape: Tape,
}

impl TuringMachineState {
    pub fn new(state: State, tape: Tape) -> Self {
        TuringMachineState { state, tape }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TuringMachineSet {
    machine_code: TuringMachine,
    machine_state: TuringMachineState,
}

impl TuringMachineSet {
    pub fn new(
        init_state: State,
        accepted_state: impl IntoIterator<Item = State>,
        code: impl IntoIterator<Item = CodeEntry>,
        tape: TapeAsVec,
    ) -> Self {
        let machine_code = TuringMachine::new(init_state.clone(), accepted_state.into_iter(), code);
        let machine_state =
            TuringMachineState::new(init_state, Tape::new(tape.left, tape.head, tape.right));
        TuringMachineSet {
            machine_code,
            machine_state,
        }
    }
    fn now_key(&self) -> CodeKey {
        CodeKey(
            self.machine_state.tape.head_read().clone(),
            self.machine_state.state.clone(),
        )
    }
    pub fn now_state(&self) -> &State {
        &self.machine_state.state
    }
    pub fn now_tape(&self) -> TapeAsVec {
        self.machine_state.tape.show()
    }
    pub fn code_as_vec(&self) -> Vec<CodeEntry> {
        self.machine_code.code.code_as_vec()
    }
    pub fn is_terminate(&self) -> bool {
        self.machine_code
            .accepted_state
            .contains(&self.machine_state.state)
            || {
                let hash = &self.machine_code.code.code();
                let key = self.now_key();
                !hash.contains_key(&key)
            }
    }
    fn one_step(&mut self) {
        if !self.is_terminate() {
            let hash = self.machine_code.code.code();
            let key = self.now_key();
            let CodeValue(sign, state, direction) = hash.get(&key).unwrap();
            self.machine_state.tape.head_write(sign);
            self.machine_state.tape.move_to(direction);
            self.machine_state.state = state.clone();
        }
    }
    pub fn step(&mut self, num: usize) -> Result<(), usize> {
        for i in 0..num {
            if self.is_terminate() {
                return Err(i);
            }
            self.one_step();
        }
        Ok(())
    }
    pub fn result(&self) -> Result<TapeAsVec, String> {
        if !self.is_terminate() {
            return Err("not terminated".to_string());
        }
        Ok(self.now_tape())
    }
}

impl Display for TuringMachineSet {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "code:")?;
        for (CodeKey(k1, k2), CodeValue(v1, v2, v3)) in self.machine_code.code.hash.iter() {
            writeln!(f, "{k1}, {k2}, {v1}, {v2}, {v3:?}")?;
        }
        writeln!(f, "state: {}", self.machine_state.state)?;
        writeln!(f, "tape: {:?}", self.machine_state.tape)
    }
}

// mod tests {
//     use super::*;
//     #[test]
//     fn tape_test() {

//     }
// }
