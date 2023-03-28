use crate::machine::*;
use yew::prelude::*;

#[derive(Default)]
pub struct TuringMachineView {
    machine: Option<TuringMachine>,
    callback_onlog: Option<Callback<String>>,
}

impl TuringMachineView {
    fn send_log(&mut self, str: String) {
        if let Some(ref callback) = self.callback_onlog {
            callback.emit(str);
        }
    }
}

#[derive(Clone, PartialEq)]
pub enum TuringMachineMsg {
    LoadFromString(String, String, String),
    #[allow(dead_code)]
    LoadFromMachine(TuringMachine),
    Step(usize),
    SetEventLog(Callback<String>),
}

#[derive(Default, Clone, PartialEq, Properties)]
pub struct TuringMachineProp {
}

impl Component for TuringMachineView {
    type Message = TuringMachineMsg;
    type Properties = TuringMachineProp;
    fn create(_ctx: &Context<Self>) -> Self {
        Self::default()
    }
    fn view(&self, ctx: &Context<Self>) -> Html {
        let machine_html: Html =
        match &self.machine {
            Some(ref machine) => html! {
                <>
                <div class="box">
                    <> {"state:"} {machine.state.clone()} {""} <br/> </>
                    <> {"l:"} {
                        for machine.tape.left.iter().rev().take(10).map(|sign| html!{<> {sign} {"|"} </>})
                    } {"..."} <br/> </>
                    <> {"h:"} {
                        machine.tape.head.clone()
                    } <br/> </>
                    <> {"r:"} {
                        for machine.tape.right.iter().rev().take(10).map(|sign| html!{<> {sign} {"|"} </>})
                    } {"..."} <br/> </>
                </div>
                <div class="box">
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
                        machine.code.0.iter().map(|(CodeKey(key_sign, key_state), CodeValue(value_sign, value_state, value_move))|{
                            html! {
                                <tr>
                                    <td> {key_sign} </td>
                                    <td> {key_state} </td>
                                    <td> {value_sign} </td>
                                    <td> {value_state} </td>
                                    <td> {format!("{:?}", value_move)} </td>
                                </tr>
                            }
                        }).collect::<Html>()
                    }
                    </tbody>
                    </table>
                </div>
                </>
            },
            None => html! {
                <>
                    {"no machine found"}
                </>
            }
        };
        let controls_html: Html = html! {
            <>
            <button onclick={ctx.link().callback(|_| TuringMachineMsg::Step(1)) }> {"step"} </button>
            <button onclick={ctx.link().callback(|_| TuringMachineMsg::Step(10)) }> {"step 10"} </button>
            <button onclick={ctx.link().callback(|_| TuringMachineMsg::Step(100)) }> {"step 100"} </button>
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
                        } else {machine.step()}
                    }
                    if let Some(num) = result {
                        self.send_log(format!("machine terminated at step {num}"));
                    } else {
                        self.send_log(format!("machine step {num}"));
                    }
                } else {
                    unreachable!()
                }
            }
            TuringMachineMsg::SetEventLog(callback) => {
                callback.emit("callback setted".to_owned());
                self.callback_onlog = Some(callback);
            }
            TuringMachineMsg::LoadFromString(state, tape, code) => {
                self.send_log("parsing...".to_string());
                let state: State = State(state);
                let tape: Tape = match Tape::try_from(tape) {
                    Ok(tape) => {tape}
                    Err(err) => {
                        self.send_log(format!("error! {}", err));
                        return false;
                    }
                };
                let code: Code = match Code::try_from(code) {
                    Ok(code) => {code}
                    Err(err) => {
                        self.send_log(format!("error! {}", err));
                        return false;
                    }
                };
                self.send_log("succeed!".to_owned());
                let machine: TuringMachine = TuringMachine { state, tape, code } ;
                self.machine = Some(machine);
            }
            TuringMachineMsg::LoadFromMachine(machine) => {
                self.machine = Some(machine);
            }
        }
        true
    }
}