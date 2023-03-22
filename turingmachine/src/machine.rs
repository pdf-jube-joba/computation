use std::collections::HashMap;
use std::fmt::{Display};
use yew::prelude::*;
use yew::{Properties};

// テープの動く方向を表す。
#[derive(Debug, Clone, PartialEq)]
pub enum Direction {
    Right,
    Left,
}

impl TryFrom<&str> for Direction {
    type Error = String;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "R" => Ok(Direction::Right),
            "L" => Ok(Direction::Left),
            _ => Err("direction: fail".to_string()),
        }
    }
}

// テープで用いる記号について
// 一般の（空白を含む）文字列が一つの記号を表す。
// None が空白記号を表す。
#[derive(Debug, Default, Clone, PartialEq, Hash, Eq)]
pub struct Sign(String);

impl Display for Sign {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

// 空白記号
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

#[derive(Default)]
pub struct TuringMachineView {
    machine: Option<TuringMachine>,
    callback_onlog: Option<Callback<String>>,
}

impl TuringMachineView {
    fn send_log(&mut self, str: String) {
        if let Some(ref callback) = self.callback_onlog {
            callback.emit(str);
        }
    }
}

#[derive(Clone, PartialEq)]
pub enum TuringMachineMsg {
    LoadFromString(String, String, String),
    #[allow(dead_code)]
    LoadFromMachine(TuringMachine),
    Step(usize),
    SetEventLog(Callback<String>),
}

#[derive(Default, Clone, PartialEq, Properties)]
pub struct TuringMachineProp {
}

impl Component for TuringMachineView {
    type Message = TuringMachineMsg;
    type Properties = TuringMachineProp;
    fn create(_ctx: &Context<Self>) -> Self {
        Self::default()
    }
    fn view(&self, ctx: &Context<Self>) -> Html {
        let machine_html: Html =
        match &self.machine {
            Some(ref machine) => html! {
                <>
                <div class="box">
                    <> {"state:"} {machine.state.clone()} {""} <br/> </>
                    <> {"l:"} {
                        for machine.tape.left.iter().rev().take(10).map(|sign| html!{<> {sign} {"|"} </>})
                    } {"..."} <br/> </>
                    <> {"h:"} {
                        machine.tape.head.clone()
                    } <br/> </>
                    <> {"r:"} {
                        for machine.tape.right.iter().rev().take(10).map(|sign| html!{<> {sign} {"|"} </>})
                    } {"..."} <br/> </>
                </div>
                <div class="box">
                    <table>
                    <thead> <tr>
                        <td> {"key_sign"} </td>
                        <td> {"key_state"} </td>
                        <td> {"value_sign"} </td>
                        <td> {"value_state"} </td>
                        <td> {"value_move"} </td>
                    </tr> </thead>
                    <tbody>
                    {
                        machine.code.0.iter().map(|(CodeKey(key_sign, key_state), CodeValue(value_sign, value_state, value_move))|{
                            html! {
                                <tr>
                                    <td> {key_sign} </td>
                                    <td> {key_state} </td>
                                    <td> {value_sign} </td>
                                    <td> {value_state} </td>
                                    <td> {format!("{:?}", value_move)} </td>
                                </tr>
                            }
                        }).collect::<Html>()
                    }
                    </tbody>
                    </table>
                </div>
                </>
            },
            None => html! {
                <>
                    {"no machine found"}
                </>
            }
        };
        let controls_html: Html = html! {
            <>
            <button onclick={ctx.link().callback(|_| TuringMachineMsg::Step(1)) }> {"step"} </button>
            <button onclick={ctx.link().callback(|_| TuringMachineMsg::Step(10)) }> {"step 10"} </button>
            <button onclick={ctx.link().callback(|_| TuringMachineMsg::Step(100)) }> {"step 100"} </button>
            </>
        };
        html! {
            <div class="machine">
            {"machine"} <br/>
            {machine_html}
            {controls_html}
            </div>
        }
    }
    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            TuringMachineMsg::Step(num) => {
                if let Some(ref mut machine) = self.machine {
                    let mut result = None;
                    for index in 0..num {
                        if machine.is_terminate() {
                            result = Some(index);
                            break;
                        } else {machine.step()}
                    }
                    if let Some(num) = result {
                        self.send_log(format!("machine terminated at step {num}"));
                    } else {
                        self.send_log(format!("machine step {num}"));
                    }
                } else {
                    unreachable!()
                }
            }
            TuringMachineMsg::SetEventLog(callback) => {
                callback.emit("callback setted".to_owned());
                self.callback_onlog = Some(callback);
            }
            TuringMachineMsg::LoadFromString(state, tape, code) => {
                self.send_log("parsing...".to_string());
                let state: State = State(state);
                let tape: Tape = match Tape::try_from(tape) {
                    Ok(tape) => {tape}
                    Err(err) => {
                        self.send_log(format!("error! {}", err));
                        return false;
                    }
                };
                let code: Code = match Code::try_from(code) {
                    Ok(code) => {code}
                    Err(err) => {
                        self.send_log(format!("error! {}", err));
                        return false;
                    }
                };
                self.send_log("succeed!".to_owned());
                let machine: TuringMachine = TuringMachine { state, tape, code } ;
                self.machine = Some(machine);
            }
            TuringMachineMsg::LoadFromMachine(machine) => {
                self.machine = Some(machine);
            }
        }
        true
    }
}