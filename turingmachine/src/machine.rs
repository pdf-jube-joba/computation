use std::collections::HashMap;
use std::fmt::Display;
use yew::prelude::*;
use yew::{Properties};

#[derive(Debug, Clone, PartialEq)]
pub enum MoveTo {
    Right,
    Left
}

impl TryFrom<&str> for MoveTo {
    type Error = ();
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "R" => Ok(MoveTo::Right),
            "L" => Ok(MoveTo::Left),
            _ => Err(()),
        }
    }
}

#[derive(Debug, Default, Clone, PartialEq, Hash, Eq)]
pub struct Sign(Option<String>);
impl Display for Sign {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = if let Sign(Some(str)) = self {str} else {""};
        write!(f, "{}", str)
    }
}

#[derive(Debug, Default, Clone, PartialEq, Properties, Hash, Eq)]
struct Tape {
    left: Vec<Sign>,
    head: Sign,
    right: Vec<Sign>
}

impl Tape {
    fn move_to(&mut self, m: &MoveTo) {
        match m {
            MoveTo::Left => {
                let next_head = self.left.pop().unwrap_or_default();
                let old_head = std::mem::replace(&mut self.head, next_head);
                self.right.push(old_head);
            }
            MoveTo::Right => {
                let next_head = self.right.pop().unwrap_or_default();
                let old_head = std::mem::replace(&mut self.head, next_head);
                self.left.push(old_head);
            }
        }
    }
}

impl TryFrom<String> for Tape {
    type Error = ();
    fn try_from(value: String) -> Result<Self, Self::Error> {
        let v: Vec<&str> = value.lines().collect();
        if v.len() < 3 {return Err(());}
        let left: Vec<Sign> = v[0].rsplit("|").map(|s| Sign(Some(s.to_string()))).collect();
        let head: Sign = Sign(Some(v[1].trim().to_string()));
        let right: Vec<Sign> = v[2].rsplit("|").map(|s| Sign(Some(s.to_string()))).collect();
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


#[derive(Debug, Clone, PartialEq, Hash, Eq)]
pub struct CodeKey(Sign, State);
#[derive(Debug, Clone, PartialEq)]
pub struct CodeValue(Sign, State, MoveTo);

pub fn try_parse_one_entry(s: &str) -> Result<(CodeKey, CodeValue), ()> {
    let v: Vec<&str> = s.split(",").collect();
    if v.len() < 5 {return Err(());}
    let move_to: MoveTo = if let Ok(move_to) = (v[4]).try_into() {move_to} else {return Err(())};
    let code_key: CodeKey = CodeKey(Sign(Some(v[0].to_string())), State(v[1].to_string()));
    let code_value: CodeValue = CodeValue(Sign(Some(v[2].to_string())), State(v[3].to_string()), move_to);
    Ok((code_key, code_value))
}

#[derive(Debug, Default, Clone, PartialEq)]
struct Code(HashMap<CodeKey, CodeValue>);

impl TryFrom<String> for Code {
    type Error = ();
    fn try_from(value: String) -> Result<Self, Self::Error> {
        let mut hash = HashMap::new();
        for str in value.lines() {
            let (key, value) = try_parse_one_entry(str)?;
            hash.insert(key, value);
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
            let head_sign = self.tape.head.clone();
            let Code(ref code) = &self.code;
            code.get(&CodeKey(head_sign, self.state.clone())).is_none()
        }
    }
    pub fn step(&mut self){
        let head_sign = self.tape.head.clone();
        let maybe_next = {
            let Code(code) = &self.code;
            code.get(&CodeKey(head_sign, self.state.clone()))
        };
        if let Some(CodeValue(write_sign, next_state, move_to)) = maybe_next {
            self.state = next_state.clone();
            self.tape.head = write_sign.clone();
            self.tape.move_to(move_to);
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
                        machine.tape.left.iter().rev().map(|sign| html!{<> {sign} {"|"} </>}).collect::<Html>()
                    } {"..."} <br/> </>
                    <> {"h:"} {
                        machine.tape.head.clone()
                    } <br/> </>
                    <> {"r:"} {
                        machine.tape.right.iter().rev().map(|sign| html!{<> {sign} {"|"} </>}).collect::<Html>()
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
                        self.send_log(format!("machine terminated at {num}"));
                    } else {
                        self.send_log(format!("machine step {num}"));
                    }
                } else {
                    self.send_log("no machine found but step pushed".to_owned());
                }
            }
            TuringMachineMsg::SetEventLog(callback) => {
                callback.emit("callback setted".to_owned());
                self.callback_onlog = Some(callback);
            }
            TuringMachineMsg::LoadFromString(state, tape, code) => {
                let state: State = State(state);
                let tape: Tape = if let Ok(tape) = Tape::try_from(tape) {tape} else {
                    self.send_log("failed parse tape".to_owned());
                    return true;
                };
                let code: Code = if let Ok(code) = Code::try_from(code) {code} else {
                    self.send_log("failed parse code".to_owned());
                    return true;
                };
                self.send_log("succed to parse".to_owned());
                let machine: TuringMachine = TuringMachine { state, tape, code} ;
                self.machine = Some(machine);
            }
        }
        true
    }
}