use std::collections::HashMap;
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

pub type Sign = Option<String>;
pub fn sign_to_str(sign: &Sign) -> &str {
    if let Some(ref str) = sign {str} else {" "}
}

#[derive(Debug, Default, Clone, PartialEq, Properties)]
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

pub type State = String;

pub type CodeKey = (Sign, State);
pub type CodeValue = (Sign, State, MoveTo);

type Code = HashMap<CodeKey, CodeValue>;

pub fn try_parse(s: String) -> Option<(CodeKey, CodeValue)> {
    let v: Vec<&str> = s.split_ascii_whitespace().collect();
    if v.len() < 5 {return None;}
    let move_to: MoveTo = if let Ok(move_to) = (v[4]).try_into() {move_to} else {return None};
    let code_key: CodeKey = (Some(v[0].to_string()), v[1].to_string());
    let code_value: CodeValue = (Some(v[2].to_string()), v[3].to_string(), move_to);
    Some((code_key, code_value))
}

#[derive(Debug, Default, Clone, PartialEq)]
pub struct TuringMachine {
    state: State,
    tape: Tape,
    code: Code
}

impl TuringMachine {
    pub fn step(&mut self) -> bool {
        let now = (self.tape.head.clone(), self.state.clone());
        let next = self.code.get(&now);
        if let Some((write_sign, next_state, move_to)) = next {
            self.state = next_state.clone();
            self.tape.head = write_sign.clone();
            self.tape.move_to(move_to);
            true
        } else {false}
    }
    pub fn try_parse(str: &str) -> Result<TuringMachine, ()> {
        todo!()
    }
}

#[derive(Default)]
pub struct TuringMachineView {
    machine: TuringMachine,
    // machine_event: Vec<String>,
    callback_onlog: Option<Callback<String>>,
}

#[derive(Clone, PartialEq)]
pub enum TuringMachineMsg {
    Load(TuringMachine),
    LoadFromString(String, String, String),
    Step,
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
        let machine = &self.machine;
        html! {
            <div class="turing-machine-view">
                <> {"state:"} {machine.state.clone()} {""} </>
                <> {"l:"} {
                    machine.tape.left.iter().rev().map(|sign| html!{<> {sign_to_str(sign)} {"|"} </>}).collect::<Html>()
                } {"..."} </>
                <> {"h:"} {
                    machine.tape.head.clone()
                } </>
                <> {"r:"} {
                    machine.tape.left.iter().rev().map(|sign| html!{<> {sign_to_str(sign)} {"|"} </>}).collect::<Html>()
                } {"..."} </>
                <div class="code-view-entry">
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
                        machine.code.iter().map(|((key_sign, key_state), (value_sign, value_state, value_move))|{
                            html! {
                                <tr>
                                    <td> {sign_to_str(&key_sign)} </td>
                                    <td> {key_state} </td>
                                    <td> {sign_to_str(&value_sign)} </td>
                                    <td> {value_state} </td>
                                    <td> {format!("{:?}", value_move)} </td>
                                </tr>
                            }
                        }).collect::<Html>()
                    }
                    </tbody>
                    </table>
                </div>
                <button onclick={ctx.link().callback(|_| TuringMachineMsg::Step) }> {"step"} </button>
            </div>
        }
    }
    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            TuringMachineMsg::Step => {
                if let Some(ref on_step) = self.callback_onlog {
                    on_step.emit("machine step".to_owned())
                }
                self.machine.step();
            }
            TuringMachineMsg::Load(machine) => {
                self.machine = machine;
            }
            TuringMachineMsg::SetEventLog(callback) => {
                self.callback_onlog = Some(callback);
            }
            TuringMachineMsg::LoadFromString(str1, st2, str3) => {
                // todo!()
            }
        }
        true
    }
}