use web_sys::HtmlInputElement;
use yew::html::Scope;
use yew::prelude::*;
use yew::Properties;

use turing_machine::manipulation::builder::TuringMachineBuilder;
use turing_machine::manipulation::tape::string_split_by_line_interpretation;

use super::machine::*;

#[derive(Default)]
pub struct ControlView {
    source_code: String,
    tape: String,
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
        html! {
            <div class="control">
            {"control"} <br/>
            <>
                <div class="box">
                    {"code"}
                    <textarea rows="10" oninput={oninput_code}/>
                </div>
                <div class="box">
                {"tape"}
                    <textarea rows="3" oninput={oninput_tape}/>
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
                self.source_code = code;
            }
            ControlMsg::OnInputTape(tape) => {
                self.tape = tape;
            }
            ControlMsg::Load => 'comp: {
                let Some(ref mut scope) = self.machine else {
                    self.send_this_log("no machine found");
                    break 'comp;
                };
                fn handle(code: &str, tape: &str) -> Result<TuringMachineBuilder, String> {
                    let interpretation = string_split_by_line_interpretation();
                    let mut builder = TuringMachineBuilder::new("user").unwrap();
                    builder
                        .from_source(code)
                        .map_err(|_| "builder error".to_string())?;
                    builder.input((interpretation.write())(tape.to_string())?);
                    Ok::<TuringMachineBuilder, String>(builder)
                }
                let builder = {
                    match handle(&self.source_code, &self.tape) {
                        Ok(builder) => builder,
                        Err(err) => {
                            self.send_this_log(format!("failed on build {err}"));
                            break 'comp;
                        }
                    }
                };
                match builder.build() {
                    Ok(machine) => {
                        scope.send_message(TuringMachineMsg::LoadFromMachine(Box::new(machine)));
                        self.send_this_log("success");
                    }
                    Err(err) => {
                        self.send_this_log(format!("failed on {err:?}"));
                    }
                }
            }
        }
        true
    }
}
