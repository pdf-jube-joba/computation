use turing_machine::machine::*;
// use crate::manipulation::TuringMachineBuilder;
use std::{fmt::Display, iter::repeat};
use utils::view::ControlStepView;
use yew::prelude::*;

#[derive(Debug, Clone, PartialEq, Properties)]
struct SignBoxProps {
    sign: Sign,
}

#[function_component(SignBox)]
fn sign_box_view(SignBoxProps { sign }: &SignBoxProps) -> Html {
    if *sign == Sign::blank() {
        html! {
            <span class={classes!("sign-box")}> {"_"} </span>
        }
    } else {
        html! {
            <span class={classes!("sign-box")}> {sign} </span>
        }
    }
}

#[derive(Clone, PartialEq, Properties)]
pub struct StateProps {
    state: State,
}

#[function_component(StateView)]
pub fn state_view(StateProps { state }: &StateProps) -> Html {
    html! {
        <div> {state} </div>
    }
}

#[derive(Debug, Clone, PartialEq, Properties)]
pub struct TapeProps {
    pub tape: TapeAsVec,
}

#[function_component(TapeView)]
pub fn tape_view(TapeProps { tape }: &TapeProps) -> Html {
    html! {
        <>
        {"tape"} <br/>
            <div class={classes!("tape")}>
            <div class={classes!("tape-left")}> {
                for tape.left.iter().chain(repeat(&Sign::blank())).take(10).map(|sign| html!{<SignBox sign={sign.clone()}/>})
            } </div>
            <div class={classes!("tape-head")}>
                <SignBox sign={tape.head.clone()}/>
            </div>
            <div class={classes!("tape-right")}> {
                for tape.right.iter().chain(repeat(&Sign::blank())).take(10).map(|sign| html!{<SignBox sign={sign.clone()}/>})
            } </div>
            </div>
        </>
    }
}

#[derive(Debug, Clone, PartialEq, Properties)]
pub struct CodeProps {
    pub code: Vec<CodeEntry>,
}

#[function_component(CodeView)]
pub fn code_view(CodeProps { code }: &CodeProps) -> Html {
    html! {
        <>
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
            code.iter().map(|entry|{
                html! {
                    <tr>
                        <td> {entry.key_sign()} </td>
                        <td> {entry.key_state()} </td>
                        <td> {entry.value_sign()} </td>
                        <td> {entry.value_state()} </td>
                        <td> {format!("{:?}", entry.value_direction())} </td>
                    </tr>
                }
            }).collect::<Html>()
        }
        </tbody>
        </table>
        </>
    }
}

pub struct TuringMachineView {
    machine: Option<TuringMachineSet>,
    callback_on_log: Option<Callback<String>>,
    callback_on_terminate: Option<Callback<TapeAsVec>>,
}

impl TuringMachineView {
    fn send_log(&mut self, str: String) {
        if let Some(ref callback) = self.callback_on_log {
            callback.emit(str);
        }
    }
}

pub enum TuringMachineMsg {
    LoadFromMachine(Box<TuringMachineSet>),
    Step(usize),
    SetEventLog(Callback<String>),
    SetMachineOnTerminate(Callback<TapeAsVec>),
}

#[derive(Default, Clone, PartialEq, Properties)]
pub struct TuringMachineProp {
    pub code_visible: bool,
}

impl Component for TuringMachineView {
    type Message = TuringMachineMsg;
    type Properties = TuringMachineProp;
    fn create(ctx: &Context<Self>) -> Self {
        Self {
            machine: None,
            callback_on_log: None,
            callback_on_terminate: None,
        }
    }
    fn view(&self, ctx: &Context<Self>) -> Html {
        match &self.machine {
            None => html! {
                <> {"no machine found"} </>
            },
            Some(machine) => {
                let TuringMachineProp { code_visible } = ctx.props();
                let on_step: Callback<usize> =
                    ctx.link().callback(|step| TuringMachineMsg::Step(step));
                html! {
                    <div class="machine">
                        {"machine"} <br/>
                            <StateView state={machine.now_state().clone()} />
                            <TapeView tape={machine.now_tape()} />
                            {
                                if *code_visible {html!{ <CodeView code={machine.code_as_vec().clone()} /> }} else {html!{}}
                            }
                            <ControlStepView on_step={on_step}/>
                    </div>
                }
            }
        }
    }
    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            TuringMachineMsg::Step(num) => {
                if let Some(ref mut machine) = self.machine {
                    let result = machine.step(num);
                    if let Err(num) = result {
                        let tape = machine.now_tape();
                        if let Some(callback) = &self.callback_on_terminate {
                            callback.emit(tape);
                        }
                        self.send_log(format!("machine terminated at step {num}"));
                    } else {
                        self.send_log(format!("machine step {num}"));
                    }
                } else {
                    self.send_log("machine not setted".to_string());
                }
            }
            TuringMachineMsg::SetEventLog(callback) => {
                callback.emit("callback setted".to_owned());
                self.callback_on_log = Some(callback);
            }
            TuringMachineMsg::LoadFromMachine(machine) => {
                self.machine = Some(*machine);
                self.send_log("machine setted".to_string());
            }
            TuringMachineMsg::SetMachineOnTerminate(callback) => {
                self.callback_on_terminate = Some(callback);
            }
        }
        true
    }
}
