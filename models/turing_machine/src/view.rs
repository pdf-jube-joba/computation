use crate::machine::*;
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

pub struct TuringMachineView {
    machine: Option<TuringMachineSet>,
    callback_on_log: Option<Callback<String>>,
    callback_on_terminate: Option<Callback<TapeAsVec>>,
    tick_active: bool,
    tick_interval: Interval,
}

impl TuringMachineView {
    fn send_log(&mut self, str: String) {
        if let Some(ref callback) = self.callback_on_log {
            callback.emit(str);
        }
    }
}

#[derive(Clone)]
pub enum TuringMachineMsg {
    LoadFromMachine(TuringMachineSet),
    Step(usize),
    SetEventLog(Callback<String>),
    SetTerminateCallback(Callback<TapeAsVec>),
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
        let machine_html: Html = match &self.machine {
            Some(machine) => html! {
                <>
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
        let controls_html: Html = html! {
            <>
            <button onclick={ctx.link().callback(|_| TuringMachineMsg::Step(1)) }> {"step"} </button>
            <button onclick={ctx.link().callback(|_| TuringMachineMsg::Step(10)) }> {"step 10"} </button>
            <button onclick={ctx.link().callback(|_| TuringMachineMsg::Step(100)) }> {"step 100"} </button>
            <button onclick={ctx.link().callback(|_| TuringMachineMsg::TickToggle)}> {"toggle active"} </button>
            </>
        };
        html! {
            <div class="machine">
            {"machine"} <br/>
            {machine_html}
            {controls_html}
            </div>
        }
    }
    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            TuringMachineMsg::Step(num) => {
                if let Some(ref mut machine) = self.machine {
                    let mut result = None;
                    for index in 0..num {
                        if machine.is_terminate() {
                            result = Some(index);
                            break;
                        } else {
                            machine.step()
                        }
                    }
                    if let Some(num) = result {
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
            TuringMachineMsg::SetTerminateCallback(callback) => {
                self.send_log("on terminate callback setted".to_string());
                self.callback_on_terminate = Some(callback);
            }
            TuringMachineMsg::LoadFromMachine(machine) => {
                self.machine = Some(machine);
            }
            TuringMachineMsg::TickToggle => {
                self.tick_active = !self.tick_active;
            }
            TuringMachineMsg::Tick => {
                if self.tick_active {
                    if let Some(ref mut machine) = self.machine {
                        if machine.is_terminate() {
                            self.tick_active = false;
                            self.send_log(format!("machine teminate"));
                        } else {
                            machine.step();
                            self.send_log(format!("machine step"));
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
