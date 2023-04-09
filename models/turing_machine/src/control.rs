use web_sys::HtmlInputElement;
use yew::html::Scope;
use yew::prelude::*;
use yew::Properties;

use crate::machine::State;
use crate::machine::TapeAsVec;
use crate::machine::TuringMachineSet;
use crate::manipulation::StandardIntepretation;

use super::view::*;
use super::manipulation::TuringMachineBuilder;

#[derive(Default)]
pub struct ControlView {
    code: String,
    tape: String,
    initial_state: String,
    accepted_state: String,
    machine: Option<Scope<TuringMachineView>>,
    event_log: Vec<String>,
}

impl ControlView {
    fn send_this_log<T>(&mut self, str: T)
    where
        T: AsRef<str>,
    {
        self.event_log.push(format!("control: {}", str.as_ref()));
    }
}

pub enum ControlMsg {
    SetTargetMachineView(Scope<TuringMachineView>),
    EventLog(String),
    OnInputCode(String),
    OnInputTape(String),
    OnInputInitialState(String),
    OnInputAcceptedState(String),
    Load,
}

#[derive(Clone, PartialEq, Properties)]
pub struct ControlProp {}

impl Component for ControlView where {
    type Message = ControlMsg;
    type Properties = ();
    fn create(_ctx: &Context<Self>) -> Self {
        Self::default()
    }
    fn view(&self, ctx: &Context<Self>) -> Html {
        fn to(e: InputEvent) -> String {
            let value: HtmlInputElement = e.target_unchecked_into();
            value.value()
        }
        let oninput_code = ctx.link().callback(|e| ControlMsg::OnInputCode(to(e)));
        let oninput_tape = ctx.link().callback(|e| ControlMsg::OnInputTape(to(e)));
        let oninput_initial_state = ctx
            .link()
            .callback(|e| ControlMsg::OnInputInitialState(to(e)));
        let oninput_accepted_state = ctx
            .link()
            .callback(|e| ControlMsg::OnInputAcceptedState(to(e)));

        html! {
            <div class="control">
            {"control"} <br/>
            <>
                <div class="box">
                    {"initial state"}
                    <textarea oninput={oninput_initial_state}/>
                </div>
                <div class="box">
                    {"accepted state"}
                    <textarea oninput={oninput_accepted_state}/>
                </div>
                <div class="box">
                {"tape"}
                <textarea rows="3" oninput={oninput_tape}/>
                </div>
                <div class="box">
                    {"code"}
                    <textarea oninput={oninput_code}/>
                </div>
                <div class="box">
                    {"event"} <br/>
                    <pre>
                    {for self.event_log.iter().rev().take(10).map(|str| html!{<> {str} <br/> </>})}
                    </pre>
                </div>
                <button onclick={ctx.link().callback(|_| ControlMsg::Load)}> {"load"} </button>
            </>
            </div>
        }
    }
    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            ControlMsg::SetTargetMachineView(scope) => {
                let callback = ctx
                    .link()
                    .callback(|log| ControlMsg::EventLog(format!("machine: {}", log)));
                scope.send_message(TuringMachineMsg::SetEventLog(callback));
                self.machine = Some(scope);
            }
            ControlMsg::EventLog(str) => {
                self.event_log.push(str);
            }
            ControlMsg::OnInputCode(code) => {
                self.code = code;
            }
            ControlMsg::OnInputTape(tape) => {
                self.tape = tape;
            }
            ControlMsg::OnInputInitialState(state) => {
                self.initial_state = state;
            }
            ControlMsg::OnInputAcceptedState(state) => {
                self.accepted_state = state;
            }
            ControlMsg::Load => {
                if let Some(ref mut scope) = self.machine {
                    let handle = || {
                        let mut builder = TuringMachineBuilder::<StandardIntepretation, TapeAsVec, TapeAsVec>::new("user").unwrap();
                        builder
                            .init_state(State::try_from(self.initial_state.as_ref())?)
                            .accepted_state({
                                let vec: Vec<State> = self.accepted_state
                                    .split_whitespace().map(|s| {
                                            State::try_from(s)
                                    }).collect::<Result<_, _>>()?;
                                vec
                            })
                            .code_from_str(&self.code)?
                            .initial_tape_from_str(&self.tape)?;
                        let machine = builder.build()?;
                        Ok::<TuringMachineSet, String>(machine)
                    };
                    match handle() {
                        Ok(machine) => {
                            scope.send_message(TuringMachineMsg::LoadFromMachine(machine));
                            self.send_this_log(&format!("success"));
                        }
                        Err(err) => {
                            self.send_this_log(&format!("failed on {err}"));
                        }
                    }
                } else {
                    self.send_this_log("no machine found");
                }
            }
        }
        true
    }
}
