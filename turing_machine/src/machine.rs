use std::collections::{HashMap, HashSet, VecDeque};
use std::fmt::Display;
use yew::Properties;

pub mod manipulation;
pub mod view;

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
    fn blank() -> Sign {
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
            return Err(format!("whitespace contained input: {value}"));
        }
        Ok(Sign(value.to_string()))
    }
}

fn to_vec_sign(str: &str) -> VecDeque<Sign> {
    str.split_whitespace()
        .map(|s| Sign::try_from(s).unwrap())
        .collect()
}

// 左右無限のテープ
#[derive(Debug, Default, Clone, PartialEq, Properties, Hash, Eq)]
struct Tape {
    left: VecDeque<Sign>,
    head: Sign,
    right: VecDeque<Sign>,
}

impl Tape {
    fn head(&self) -> &Sign {
        &self.head
    }
    fn head_mut(&mut self) -> &mut Sign {
        &mut self.head
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
}

impl TryFrom<&str> for Tape {
    type Error = String;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let v: Vec<&str> = value.lines().collect();
        if v.len() < 3 {
            return Err("tape: argument is too few".to_owned());
        }
        let left: VecDeque<Sign> = to_vec_sign(v[0]);
        let head: Sign = v[1].try_into()?;
        let right: VecDeque<Sign> = to_vec_sign(v[2]);
        Ok(Self { left, head, right })
    }
}

impl Display for Tape {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let left: String = self.left.iter()
            .map(|sign|{ format!("{sign} ") })
            .collect();
        writeln!(f, "l:{left}")?;
        writeln!(f, "h:{}", self.head)?;
        let right: String = self.right.iter()
            .map(|sign|{ format!("{sign} ") })
            .collect();
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
        if value.contains(|char: char| char.is_whitespace() || char ==',') {
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
struct CodeEntry(CodeKey, CodeValue);

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
        let code_value: CodeValue =
            CodeValue(v[2].try_into()?, v[3].try_into()?, v[4].try_into()?);
        Ok(CodeEntry(code_key, code_value))
    }
}

#[derive(Debug, Default, Clone, PartialEq, Properties)]
struct Code {
    hash: HashMap<CodeKey, CodeValue>,
}

impl Code {
    fn code(&self) -> &HashMap<CodeKey, CodeValue> {
        &self.hash
    }
    fn add(&mut self, CodeEntry(k, v): CodeEntry) {
        self.hash.insert(k, v);
    }
    fn from_iter_entry(iter: impl IntoIterator<Item = CodeEntry>) -> Self {
        Code {
            hash: HashMap::from_iter(iter.into_iter().map(|CodeEntry(k, v)|{
                (k, v)
            }))
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
struct TuringMachine {
    init_state: State,
    accepted_state: HashSet<State>,
    code: Code,
}

// TuringMachine の計算過程を表す。
//
#[derive(Debug, Clone, PartialEq)]
pub struct TuringMachineState {
    state: State,
    tape: Tape,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TuringMachineSet {
    machine_code: TuringMachine,
    machine_state: TuringMachineState,
}

impl TuringMachineSet {
    fn now_key(&self) -> CodeKey {
        CodeKey(
            self.machine_state.tape.head().clone(),
            self.machine_state.state.clone(),
        )
    }
    pub fn is_terminate(&mut self) -> bool {
        self.machine_code
            .accepted_state
            .contains(&self.machine_state.state)
            || {
                let hash = &self.machine_code.code.code();
                let key = self.now_key();
                !hash.contains_key(&key)
            }
    }
    pub fn step(&mut self) {
        if !self.is_terminate() {
            let hash = &self.machine_code.code.code();
            let key = self.now_key();
            let CodeValue(sign, state, direction) = hash.get(&key).unwrap();
            *self.machine_state.tape.head_mut() = sign.clone();
            self.machine_state.tape.move_to(direction);
            self.machine_state.state = state.clone();
        }
    }
}

impl Display for TuringMachineSet {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "code:")?;
        for (CodeKey(k1, k2), CodeValue(v1, v2, v3)) in (&self.machine_code.code.hash).iter() {
            writeln!(f, "{k1}, {k2}, {v1}, {v2}, {v3:?}")?;
        }
        writeln!(f, "state: {}", self.machine_state.state)?;
        writeln!(f, "tape: {:?}", self.machine_state.tape)
    }
}
