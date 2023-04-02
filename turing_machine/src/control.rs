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
                if let Some(ref scope) = self.machine {
                    self.event_log.push("control: load".to_owned());
                    let builder = TuringMachineBuilder {
                        init_state: self.initial_state.clone(),
                        accepted_state: self.accepted_state.clone(),
                        code: self.code.clone(),
                        initial_tape: self.tape.clone(),
                    };
                    scope.send_message(TuringMachineMsg::LoadFromBuilder(builder));
                    self.event_log.push("sended".to_string());
                } else {
                    self.event_log.push("control: machine not found".to_string());
                }
            }
        }
        true
    }
}