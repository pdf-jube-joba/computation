use yew::html::Scope;
use yew::prelude::*;
use yew::{Properties};
use web_sys::{HtmlInputElement};

use super::machine::view::*;
use super::machine::TuringMachineBuilder;

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
    where T: AsRef<str>
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
    let oninput_code = ctx.link().callback(
        |e| ControlMsg::OnInputCode(to(e))
    );
    let oninput_tape = ctx.link().callback(
        |e| ControlMsg::OnInputTape(to(e))
    );
    let oninput_initial_state = ctx.link().callback(
        |e| ControlMsg::OnInputInitialState(to(e))
    );
    let oninput_accepted_state = ctx.link().callback(
        |e| ControlMsg::OnInputAcceptedState(to(e))
    );
    
    html!{
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
                let callback = ctx.link().callback(|log| ControlMsg::EventLog(format!("machine: {}", log)));
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
                    let handle  = ||{
                        let mut builder = TuringMachineBuilder::default();
                            builder
                            .init_state(&self.initial_state)?
                            .accepted_state(&self.accepted_state)?
                            .code(&self.code)?
                            .initial_tape(&self.tape)?;
                        Ok::<TuringMachineBuilder, String>(builder)
                    };
                    let builder = handle();
                    match builder {
                        Ok(builder) => {
                            scope.send_message(TuringMachineMsg::LoadFromBuilder(builder));
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