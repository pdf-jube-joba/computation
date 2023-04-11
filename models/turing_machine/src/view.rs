use crate::machine::*;
use crate::manipulation::TuringMachineBuilder;
use crate::manipulation::RunningTuringMachine;
use gloo::timers::callback::Interval;
use yew::prelude::*;

#[derive(Debug, Clone, PartialEq, Properties)]
struct TapeProps {
    tape: TapeAsVec,
}

#[function_component(TapeView)]
fn tape_view(TapeProps { tape }: &TapeProps) -> Html {
    html! {
        <>
        {"tape"} <br/>
        <> {"l:"} {
            for tape.left.iter().take(10).map(|sign| html!{<> {sign} {"|"} </>})
        } {"..."} <br/> </>
        <> {"h:"} {
            tape.head.clone()
        } <br/> </>
        <> {"r:"} {
            for tape.right.iter().take(10).map(|sign| html!{<> {sign} {"|"} </>})
        } {"..."} <br/> </>
        </>
    }
}

#[derive(Debug, Clone, PartialEq, Properties)]
struct CodeProps {
    code: Vec<CodeEntry>,
}

#[function_component(CodeView)]
fn code_view(CodeProps { code }: &CodeProps) -> Html {
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

#[derive(Clone, PartialEq, Properties)]
struct TuringMachineResultProps {
    input: String,
    result: Result<String, String>,
}

#[function_component(TuringMachineResultView)]
fn running_turing_machine_vew(props: &TuringMachineResultProps) -> Html {
    let TuringMachineResultProps { input, result} = props;
    html!{
        <>
           {"input"} {input.clone()}
           { match result {
             Ok(output) => html! {<> {"output"} {output} </>},
             Err(err) => html! {<> {"error"} {err} </>}
           }}
        </>
    }
}

pub struct TuringMachineView {
    machine: Option<RunningTuringMachine<String, String>>,
    callback_on_log: Option<Callback<String>>,
    callback_on_terminate: Option<Callback<TapeAsVec>>,
    tick_active: bool,
    #[allow(dead_code)]
    tick_interval: Interval,
}

impl TuringMachineView {
    fn send_log(&mut self, str: String) {
        if let Some(ref callback) = self.callback_on_log {
            callback.emit(str);
        }
    }
}

pub enum TuringMachineMsg {
    LoadFromBuilder(TuringMachineBuilder<String, String>),
    // LoadFromMachine(TuringMachineSet),
    Step(usize),
    SetEventLog(Callback<String>),
    TickToggle,
    Tick,
}

#[derive(Default, Clone, PartialEq, Properties)]
pub struct TuringMachineProp {}

impl Component for TuringMachineView {
    type Message = TuringMachineMsg;
    type Properties = TuringMachineProp;
    fn create(ctx: &Context<Self>) -> Self {
        let callback = ctx.link().callback(|_| TuringMachineMsg::Tick);
        let interval = Interval::new(1000, move || callback.emit(()));

        Self {
            machine: None,
            callback_on_log: None,
            callback_on_terminate: None,
            tick_active: true,
            tick_interval: interval,
        }
    }
    fn view(&self, ctx: &Context<Self>) -> Html {
        let controls_html: Html = html! {
            <>
                <button onclick={ctx.link().callback(|_| TuringMachineMsg::Step(1)) }> {"step"} </button>
                <button onclick={ctx.link().callback(|_| TuringMachineMsg::Step(10)) }> {"step 10"} </button>
                <button onclick={ctx.link().callback(|_| TuringMachineMsg::Step(100)) }> {"step 100"} </button>
                <button onclick={ctx.link().callback(|_| TuringMachineMsg::TickToggle)}> {"toggle active"} </button>
            </>
        };
        let machine_html: Html = match &self.machine {
            Some(machine) => html! {
                <>
                <div class="box">
                    {"result"}
                    <TuringMachineResultView input={machine.first_input().clone()} result={machine.result()} />
                </div>
                <div class="box">
                    <> {"state:"} {machine.now_state().clone()} {""} <br/> </>
                    <TapeView tape={machine.now_tape().clone()}/>
                </div>
                <div class="box">
                    <CodeView code={machine.code_as_vec().clone()}/>
                </div>
                </>
            },
            None => html! {
                <>
                    {"no machine found"}
                </>
            },
        };
        html! {
            <div class="machine">
            {"machine"} <br/>
            {controls_html}
            {machine_html}
            </div>
        }
    }
    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            TuringMachineMsg::Step(num) => {
                if let Some(ref mut machine) = self.machine {
                    let result = machine.step(num);
                    if let Err(num) = result {
                        self.send_log(format!("machine terminated at step {num}"));
                    } else {
                        self.send_log(format!("machine step {num}"));
                    }
                } else {
                    self.send_log(format!("machine not setted"));
                }
            }
            TuringMachineMsg::SetEventLog(callback) => {
                callback.emit("callback setted".to_owned());
                self.callback_on_log = Some(callback);
            }
            TuringMachineMsg::LoadFromBuilder(builder) => {
                match builder.build() {
                    Ok(machine) => {
                        self.machine = Some(machine);
                        self.send_log("machine setted".to_string());
                    }
                    Err(err) => {
                        self.send_log(err);
                    }
                }
            }
            TuringMachineMsg::TickToggle => {
                self.tick_active = !self.tick_active;
            }
            TuringMachineMsg::Tick => {
                if self.tick_active {
                    if let Some(ref mut machine) = self.machine {
                        match machine.step(1) {
                            Err(_) => {
                                self.tick_active = false;
                                self.send_log(format!("machine teminate"));
                            }
                            Ok(_) => {
                                self.send_log(format!("machine step"));
                            }
                        }
                    } else {
                        self.send_log(format!("machine not setted"))
                    }
                    return true;
                } else {
                    return false;
                }
            }
        }
        true
    }
}
