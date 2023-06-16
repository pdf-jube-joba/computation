use web_sys::HtmlInputElement;
use yew::html::Scope;
use yew::prelude::*;
use yew::Properties;

use crate::machine::State;
use crate::manipulation;
use crate::manipulation::tape::string_split_by_line_interpretation;

use super::machine::*;
use crate::manipulation::TuringMachineBuilder;

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

impl Component for ControlView {
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
            ControlMsg::Load => 'comp : {
                let Some(ref mut scope) = self.machine else {
                    self.send_this_log("no machine found");
                    break 'comp;
                };
                fn handle(
                    init_state: &str,
                    accepted_state: &str,
                    code: &str,
                    tape: &str
                ) -> Result<TuringMachineBuilder<String, String>, String> {
                    let mut builder =
                    TuringMachineBuilder::new("user", string_split_by_line_interpretation())
                            .unwrap();
                    let code = manipulation::code::parse_code(code)?;
                    builder
                        .init_state(State::try_from(init_state.as_ref())?)
                        .accepted_state({
                            let vec: Vec<State> = accepted_state
                                .split_whitespace()
                                .map(State::try_from)
                                .collect::<Result<_, _>>()?;
                            vec
                        })
                        .code_new(code);
                    builder.input(tape.to_string());
                    Ok::<TuringMachineBuilder<_, _>, String>(builder)
                }
                let builder = {
                    match handle(&self.initial_state, &self.accepted_state, &self.code, &self.tape) {
                        Ok(builder) => builder,
                        Err(err) => {
                            self.send_this_log(format!("failed on build {err}"));
                            break 'comp;
                        }
                    }
                };
                match builder.build() {
                    Ok(machine) => {
                        scope.send_message(TuringMachineMsg::LoadFromMachine(machine));
                        self.send_this_log("success");
                    }
                    Err(err) => {
                        self.send_this_log(format!("failed on {err}"));
                    }
                }
            }
        }
        true
    }
}
