use crate::machine::*;
// use crate::manipulation::TuringMachineBuilder;
use std::fmt::Display;
use gloo::timers::callback::Interval;
use yew::prelude::*;

#[derive(Debug, Clone, PartialEq, Properties)]
struct SignBoxProps {
    sign: Sign
}

#[function_component(SignBox)]
fn sign_box_view(SignBoxProps { sign }: &SignBoxProps) -> Html {
    html!{
        <div class={classes!("sign-box")}> {sign} </div>
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
        <div class={classes!("tape")}> {"l:"} {
            for tape.left.iter().take(10).map(|sign| html!{<SignBox sign={sign.clone()}/>})
        } {"..."} </div>
        <div class={classes!("tape")}> {"h:"}
            <SignBox sign={tape.head.clone()}/>
        </div>
        <div class={classes!("tape")}> {"r:"} {
            for tape.right.iter().take(10).map(|sign| html!{<SignBox sign={sign.clone()}/>})
        } {"..."} </div>
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

#[derive(Clone, PartialEq, Properties)]
pub struct TuringMachineResultProps<T1, T2, T3> where
    T1: Clone + PartialEq + Properties + Display,
    T2: Clone + PartialEq + Properties + Display,
    T3: Clone + PartialEq + Properties + Display,
{
    pub input: T1,
    pub result: Option<Result<T2, T3>>,
}

#[function_component(TuringMachineResultView)]
fn running_turing_machine_vew<T1, T2, T3>(props: &TuringMachineResultProps<T1, T2, T3>) -> Html where
    T1: Clone + PartialEq + Properties + Display,
    T2: Clone + PartialEq + Properties + Display,
    T3: Clone + PartialEq + Properties + Display,
{
    let TuringMachineResultProps { input, result } = props;
    html! {
        <>
           {"input"} {input}
           {
            match result {
                None => html!{<> {"not yet"} </>},
                Some(Ok(output)) => html! {<> {"output"} {output} </>},
                Some(Err(err)) => html! {<> {"error"} {err} </>}
            }
            }
        </>
    }
}

#[derive(Clone, PartialEq, Properties)]
pub struct ControlStepProps {
    callback_step_usr: Callback<usize>,
    callback_toggle_autostep: Callback<()>,
    now_toggle_state: bool,
}

#[function_component(ControlStepView)]
fn control_step(props: &ControlStepProps) -> Html {
    let ControlStepProps {
        callback_step_usr,
        callback_toggle_autostep,
        now_toggle_state,
    } = props;
    let callback_step_usr_1 = callback_step_usr.clone();
    let callback_step_usr_2 = callback_step_usr.clone();
    let callback_step_usr_3 = callback_step_usr.clone();
    let callback_toggle_autostep = callback_toggle_autostep.clone();
    html!{
        <>
            <button onclick={move |_| callback_step_usr_1.emit(1)}> {"step 1"} </button>
            <button onclick={move |_| callback_step_usr_2.emit(10)}> {"step 10"} </button>
            <button onclick={move |_| callback_step_usr_3.emit(100)}> {"step 100"} </button>
            <button onclick={move |_| callback_toggle_autostep.emit(())}> {"auto step"} </button>
            <> {if *now_toggle_state {"on"} else {"off"}} </>
        </>
    }
}

pub struct TuringMachineView {
    machine: Option<TuringMachineSet>,
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
    // LoadFromBuilder(TuringMachineBuilder<String, String>),
    LoadFromMachine(TuringMachineSet),
    Step(usize),
    SetEventLog(Callback<String>),
    SetMachineOnTerminate(Callback<TapeAsVec>),
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
            tick_active: false,
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
                        let tape = machine.now_tape();
                        if let Some(callback) = &self.callback_on_terminate {
                            callback.emit(tape);
                        }
                        self.tick_active = false;
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
                self.machine = Some(machine);
                self.send_log("machine setted".to_string());
            }
            TuringMachineMsg::TickToggle => {
                self.tick_active = !self.tick_active;
            }
            TuringMachineMsg::SetMachineOnTerminate(callback) => {
                self.tick_active = true;
                self.callback_on_terminate = Some(callback);
            }
            TuringMachineMsg::Tick => {
                if self.tick_active {
                    if let Some(ref mut machine) = self.machine {
                        let result = machine.step(1);
                        if let Err(num) = result {
                            let tape = machine.now_tape();
                            if let Some(callback) = &self.callback_on_terminate {
                                callback.emit(tape);
                            }
                            self.tick_active = false;
                            self.send_log(format!("machine terminated at step {num}"));
                        } else {
                            self.send_log("machine step 1".to_string());
                        }
                    } else {
                        self.send_log("machine not setted".to_string());
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
