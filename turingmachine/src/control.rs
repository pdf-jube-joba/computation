use yew::html::Scope;
use yew::prelude::*;
use yew::{Properties};
use web_sys::{HtmlInputElement};

use super::machine::*;

#[derive(Default)]
pub struct ControlView {
    code: String,
    tape: String,
    state: String,
    machine: Option<Scope<TuringMachineView>>,
    event_log: Vec<String>,
}

pub enum ControlMsg {
    SetTargetMachineView(Scope<TuringMachineView>),
    EventLog(String),
    OnInputCode(String),
    OnInputTape(String),
    OnInputState(String),
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
    let oninput_code = ctx.link().callback(|e: InputEvent| {
        let value: HtmlInputElement = e.target_unchecked_into();
        let str: String = value.value() ;
        ControlMsg::OnInputCode(str)
    });
    let oninput_tape = ctx.link().callback(|e: InputEvent| {
        let value: HtmlInputElement = e.target_unchecked_into();
        let str: String = value.value() ;
        ControlMsg::OnInputTape(str)
    });
    let oninput_state = ctx.link().callback(|e: InputEvent| {
        let value: HtmlInputElement = e.target_unchecked_into();
        let str: String = value.value() ;
        ControlMsg::OnInputState(str)
    });
    
    html!{
        <div class="control">
        {"control"} <br/>
        <>
            <div class="box">
                {"state"}
                <textarea oninput={oninput_state}/>
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
                {for self.event_log.iter().rev().take(10).map(|str| html!{<> {str} <br/> </>})}
            </div>
            <button onclick={ctx.link().callback(|_| ControlMsg::Load)}> {"load"} </button>
        </>
        </div>
    }
  }
    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            ControlMsg::SetTargetMachineView(scope) => {
                let callback = ctx.link().callback(|log| ControlMsg::EventLog(log));
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
            ControlMsg::OnInputState(state) => {
                self.state = state;
            }
            ControlMsg::Load => {
                if let Some(ref scope) = self.machine {
                    scope.send_message(TuringMachineMsg::LoadFromString(self.state.clone(), self.tape.clone(), self.code.clone()))
                }
                self.event_log.push("load".to_owned());
            }
        }
        true
    }
}