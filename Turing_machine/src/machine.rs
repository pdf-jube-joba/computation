use std::collections::HashMap;
use std::fmt::{Display};
use yew::{Properties};

pub mod app;
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

#[derive(Debug, Default, Clone, PartialEq, Hash, Eq)]
pub struct Sign(String);

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

impl TryFrom<String> for Tape {
    type Error = String;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        let v: Vec<&str> = value.lines().collect();
        if v.len() < 3 {return Err("tape: argument is too few".to_owned());}
        let left: Vec<Sign> = v[0].rsplit("|").map(|s| s.into()).collect();
        let head: Sign = v[1].into();
        let right: Vec<Sign> = v[2].rsplit("|").map(|s| s.into()).collect();
        Ok(Self {left, head, right})
    }
}

// マシンの状態について
// マシンの状態も文字列を用いて表す。
// ただし、空白記号であらわされる状態を停止状態とする。
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

pub fn try_parse_one_entry(s: &str) -> Result<(CodeKey, CodeValue), String> {
    let v: Vec<&str> = s.split(",").collect();
    if v.len() < 5 {return Err("code-entry: argument is too few".to_string());}
    let code_key: CodeKey = CodeKey(v[0].into(), v[1].into());
    let code_value: CodeValue = CodeValue(v[2].into(), v[3].into(), v[4].try_into()?);
    Ok((code_key, code_value))
}

#[derive(Debug, Default, Clone, PartialEq)]
struct Code(HashMap<CodeKey, CodeValue>);

impl TryFrom<String> for Code {
    type Error = String;
    fn try_from(value: String) -> Result<Self, Self::Error> {
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
        Ok(Code(hash))
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TuringMachine {
    state: State,
    tape: Tape,
    code: Code,
}

impl TuringMachine {
    pub fn is_terminate(&mut self) -> bool {
        let State(ref state) = self.state;
        state == "" || {
            let Code(ref code) = &self.code;
            // todo clone しないやり方はある？
            code.get(&CodeKey(self.tape.head.clone(), self.state.clone())).is_none()
        }
    }
    pub fn step(&mut self){
        if !self.is_terminate() {
            let maybe_next = &self.code.0.get(&CodeKey(self.tape.head.clone(), self.state.clone()));
            if let Some(CodeValue(write_sign, next_state, direction)) = maybe_next {
                self.state = next_state.clone();
                self.tape.head = write_sign.clone();
                self.tape.move_to(direction);
            }
        }
    }
}