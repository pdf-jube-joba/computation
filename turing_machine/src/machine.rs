use std::collections::{HashMap, HashSet};
use std::fmt::{Display};
use yew::{Properties};

pub mod view;
pub mod manipulation;

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
        match value {
            "R" => Ok(Direction::Right),
            "L" => Ok(Direction::Left),
            "C" => Ok(Direction::Constant),
            _ => Err("direction: fail".to_string()),
        }
    }
}

// テープで扱う記号の定義
// 空白記号（スペース）の含まれない文字列を記号として扱う
// 空の文字列で記号としての空白記号を表す
#[derive(Debug, Default, Clone, PartialEq, Hash, Eq)]
pub struct Sign(String);

impl Sign {
    fn blank() -> Sign {
        Sign::from("")
    }
}

impl Display for Sign {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<&str> for Sign {
    fn from(value: &str) -> Self {
        Sign(value.to_string())
    }
}

// 左右無限のテープ
#[derive(Debug, Default, Clone, PartialEq, Properties, Hash, Eq)]
struct Tape {
    left: Vec<Sign>,
    head: Sign,
    right: Vec<Sign>
}

impl Tape {
    fn head(&mut self) -> &mut Sign {
        &mut self.head
    }
    fn move_to(&mut self, m: &Direction) {
        match m {
            Direction::Left => {
                let next_head = self.left.pop().unwrap_or_default();
                let old_head = std::mem::replace(&mut self.head, next_head);
                self.right.push(old_head);
            }
            Direction::Right => {
                let next_head = self.right.pop().unwrap_or_default();
                let old_head = std::mem::replace(&mut self.head, next_head);
                self.left.push(old_head);
            }
            Direction::Constant => {
            }
        }
    }
}

impl TryFrom<&str> for Tape {
    type Error = String;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let v: Vec<&str> = value.lines().collect();
        if v.len() < 3 {return Err("tape: argument is too few".to_owned());}
        let left: Vec<Sign> = v[0].rsplit('|').map(|s| s.into()).collect();
        let head: Sign = v[1].into();
        let right: Vec<Sign> = v[2].rsplit('|').map(|s| s.into()).collect();
        Ok(Self {left, head, right})
    }
}

#[derive(Debug, Clone, PartialEq, Hash, Eq)]
pub struct State(String);
impl Display for State {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
impl From<&str> for State {
    fn from(value: &str) -> Self {
        State(value.to_string())
    }
}

#[derive(Debug, Clone, PartialEq, Hash, Eq)]
pub struct CodeKey(Sign, State);
#[derive(Debug, Clone, PartialEq)]
pub struct CodeValue(Sign, State, Direction);

#[derive(Debug, Default, Clone, PartialEq, Properties)]
struct Code {
    hash: HashMap<CodeKey, CodeValue>,
}

impl Code {
    fn code(&self) -> &HashMap<CodeKey, CodeValue> {
        &self.hash
    }
}

impl TryFrom<&str> for Code {
    type Error = String;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        fn try_parse_one_entry(s: &str) -> Result<(CodeKey, CodeValue), String> {
            let v: Vec<&str> = s.split(',').collect();
            if v.len() < 5 {return Err("code-entry: argument is too few".to_string());}
            let code_key: CodeKey = CodeKey(v[0].into(), v[1].into());
            let code_value: CodeValue = CodeValue(v[2].into(), v[3].into(), v[4].try_into()?);
            Ok((code_key, code_value))
        }
        let mut hash = HashMap::new();
        for (index, str) in value.lines().enumerate() {
            match try_parse_one_entry(str) {
                Ok((key, value)) => {
                    hash.insert(key, value);
                },
                Err(err) => {
                    return Err(format!("{} at line {}", err, index));
                }
            }
        }
        Ok(Code{ hash })
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
    pub fn is_terminate(&mut self) -> bool {
        self.machine_code.accepted_state.contains(&self.machine_state.state) || {
            let hash = &self.machine_code.code.code();
            let key = CodeKey(self.machine_state.tape.head().clone(), self.machine_state.state.clone());
            hash.contains_key(&key)
        }
    }
    pub fn step(&mut self){
        if !self.is_terminate() {
            let hash = &self.machine_code.code.code();
            let key = CodeKey(self.machine_state.tape.head().clone(), self.machine_state.state.clone());
            let CodeValue(sign, state, direction) = hash.get(&key).unwrap();
            *self.machine_state.tape.head() = sign.clone();
            self.machine_state.tape.move_to(direction);
            self.machine_state.state = state.clone();
        }
    }
}

#[derive(Debug, Default, Clone, PartialEq)]
pub struct TuringMachineBuilder {
    init_state: Option<State>,
    accepted_state: Option<HashSet<State>>,
    code: Option<Code>,
    initial_tape: Option<Tape>,
}

impl TuringMachineBuilder {
    pub fn build(self) -> Result<TuringMachineSet, String> {
        let machine_code = {
            let init_state = if let Some(state) = self.init_state.clone() {state} else {
                return Err("fail on initial state".to_string())
            };
            let accepted_state = if let Some(state) = self.accepted_state.clone() {state} else {
                return Err("fail on accepted state".to_string())
            };
            let code = if let Some(state) = self.code.clone() {state} else {
                return Err("fail on initial state".to_string())
            };
            TuringMachine { init_state, accepted_state, code }
        };
        let machine_state = {
            let state = if let Some(state) = self.init_state.clone() {state} else {
                return Err("fail on initial state".to_string())
            };
            let tape = if let Some(state) = self.initial_tape.clone() {state} else {
                return Err("fail on initial state".to_string())
            };
            TuringMachineState { state, tape }
        };
        Ok(TuringMachineSet { machine_code, machine_state })
    }
    
    pub fn init_state(&mut self, str: &str) -> Result<&mut Self, String> {
        self.init_state = Some(State::from(str));
        Ok(self)
    }

    pub fn accepted_state(&mut self, str: &str) -> Result<&mut Self, String> {
        self.accepted_state = str.split_ascii_whitespace()
            .map(|str| State::from(str) )
            .collect::<HashSet<State>>().into();
        Ok(self)
    }

    pub fn code(&mut self, str: &str) -> Result<&mut Self, String> {
        self.code = Some(Code::try_from(str)?);
        Ok(self)
    }

    pub fn initial_tape_left(&mut self, str: &str) -> Result<&mut Self, String> {
        todo!()
    }

    pub fn initial_tape_head(&mut self, str: &str) -> Result<&mut Self, String> {
        todo!()
    }

    pub fn initial_tape_right(&mut self, str: &str) -> Result<&mut Self, String> {
        todo!()
    }
    pub fn initial_tape(&mut self, str: &str) -> Result<&mut Self, String> {
        self.initial_tape = Tape::try_from(str)?.into();
        Ok(self)
    }


}