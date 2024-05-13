use web_sys::HtmlInputElement;
use yew::html::Scope;
use yew::prelude::*;
use yew::Properties;

use logic_circuit::machine::LoC;
use logic_circuit::manipulation;

use crate::machine::MachineMsg;

use super::machine;

#[derive(Debug, Default)]
pub struct ControlView {
    source_code: String,
    machine: Option<Scope<machine::MachineView>>,
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
    SetTargetMachineView(Scope<machine::MachineView>),
    EventLog(String),
    OnInputCode(String),
    Load,
}

impl Component for ControlView {
    type Message = ControlMsg;
    type Properties = ();
    fn create(ctx: &Context<Self>) -> Self {
        Self::default()
    }
    fn view(&self, ctx: &Context<Self>) -> Html {
        fn to(e: InputEvent) -> String {
            let value: HtmlInputElement = e.target_unchecked_into();
            value.value()
        }
        let oninput_code = ctx.link().callback(|e| ControlMsg::OnInputCode(to(e)));
        html! {
            <div class="control">
            {"control"} <br/>
            <>
            <div class="box">
                {"code"}
                <textarea row="10" oninput={oninput_code}/>
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
                let callback = ctx.link().callback(|log| ControlMsg::EventLog(log));
                scope.send_message(MachineMsg::SetEventLog(callback));
                self.machine = Some(scope);
            }
            ControlMsg::EventLog(log) => {
                self.event_log.push(log);
            }
            ControlMsg::OnInputCode(code) => {
                self.source_code = code;
            }
            ControlMsg::Load => 'comp: {
                let Some(scope) = &mut self.machine else {
                    self.send_this_log("no machine found");
                    break 'comp;
                };
                let lc: MachineMsg = match manipulation::parse(&self.source_code) {
                    Ok(code) => MachineMsg::LoadFromMachine(Box::new(code)),
                    Err(err) => {
                        self.send_this_log(err.to_string());
                        return true;
                    }
                };
                scope.send_message(lc);
                self.send_this_log("success");
            }
        }
        true
    }
}
