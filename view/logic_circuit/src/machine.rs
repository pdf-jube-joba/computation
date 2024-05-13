use gloo::timers::callback::Interval;
use std::{collections::HashMap, fmt::Display, iter::repeat, ops::Neg};
use web_sys::HtmlInputElement;
use yew::prelude::*;

use logic_circuit::machine::{Bool, FinGraph, InPin, LoC, Name};

#[derive(Debug, Clone, PartialEq, Eq, Properties)]
pub struct StateProps {
    state: Bool,
    rep: String,
}

#[function_component(StateView)]
pub fn state_view(StateProps { state, rep }: &StateProps) -> Html {
    let state_class = match state {
        Bool::T => "stateT",
        Bool::F => "stateF",
    };
    html! {<>
        <div class={state_class}>
            {rep}
        </div>
    </>}
}

#[derive(Debug, Clone, PartialEq, Eq, Properties)]
pub struct LoCProps {
    pub lc: LoC,
    // pub open_close_name: HashMap<Name, bool>,
}

#[function_component(LoCView)]
pub fn loc_view(
    LoCProps {
        lc,
        // open_close_name,
    }: &LoCProps,
) -> Html {
    match lc {
        LoC::Gate(gate) => {
            html! {
                <div class="gate">
                    {gate.name()}
                    <StateView state = {*gate.state()} rep = {"state"}/>
                </div>
            }
        }
        LoC::FinGraph(fingraph) => {
            let FinGraph {
                name,
                lcs,
                edges,
                input,
                output,
            } = fingraph;
            // let Some(b) = open_close_name.get(name) else {
            //     return html! {"not found"};
            // };
            html! {
                <div class="graph">
                    {name}
                    {for input.iter().map(|(i, (name, i0))|{
                        let s = format!("{i}={name}.{i0}");
                        html!{
                            <> <StateView state = {*fingraph.get_input(i).unwrap()} rep = {s}/> <br/> </>
                        }})
                    }
                    {for output.iter().map(|(o, (name, o0))|{
                        let s = format!("{o}={name}.{o0}");
                        html!{
                            <> <StateView state = {*fingraph.get_output(o).unwrap()} rep = {s}/> <br/> </>
                        }})
                    }
                    {if true {
                        html!{
                            {for lcs.iter().map(|(name, lc)|{
                                let (lc, inout) = fingraph.get_lc_inouts(name).unwrap();
                                html!{
                                    <>
                                        {name}
                                        {lc.name()}
                                        {for inout.iter().map(|(i, (n, o), s)| {
                                            let name = format!("{i}={name}.{o}");
                                            html!{
                                                <StateView state = {*s} rep = {name}/>
                                            }
                                        })}
                                    </>
                                }
                            })}
                        }
                    } else {
                        html!{}
                    }}
                </div>
            }
        }
        LoC::Iter(iter) => {
            todo!()
        }
    }
}

pub struct InputSetView {
    inputs: Vec<(InPin, Bool)>,
}

#[derive(Debug)]
pub enum InputSetMsg {
    Rev(usize),
}

#[derive(Clone, PartialEq, Properties)]
pub struct InputSetProps {
    input_anames: Vec<InPin>,
    on_set: Callback<Vec<(InPin, Bool)>>,
}

impl Component for InputSetView {
    type Message = InputSetMsg;
    type Properties = InputSetProps;
    fn create(ctx: &Context<Self>) -> Self {
        let InputSetProps {
            input_anames,
            on_set,
        } = ctx.props();
        let inputs = input_anames
            .iter()
            .map(|n| (n.clone(), Bool::F))
            .collect::<Vec<_>>();
        Self { inputs }
    }
    fn view(&self, ctx: &Context<Self>) -> Html {
        html! {{for self.inputs.iter().enumerate().map(|(i, (n, b))| {
            let callback = ctx.link().callback(move |_| InputSetMsg::Rev(i));
            html! {
                <>
                <div onclick={callback}>
                    <StateView state={*b} rep={n.to_string()}/>
                </div>
                </>
            }
        })}}
    }
    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        let InputSetMsg::Rev(i) = msg;
        let (_, b) = self.inputs.get_mut(i).unwrap();
        *b = b.neg();
        let InputSetProps {
            input_anames,
            on_set,
        } = ctx.props();
        on_set.emit(self.inputs.clone());
        true
    }
}

#[derive(Clone, PartialEq, Properties)]
pub struct ControlStepView {
    now_input_step: Result<usize, ()>,
}

#[derive(Clone, PartialEq, Properties)]
pub struct ControlStepProps {
    callback_step_usr: Callback<usize>,
    callback_toggle_autostep: Callback<()>,
    now_toggle_state: bool,
}

pub enum ControlStepMsg {
    ChangeStep(String),
}

impl Component for ControlStepView {
    type Message = ControlStepMsg;
    type Properties = ControlStepProps;
    fn create(_ctx: &Context<Self>) -> Self {
        Self {
            now_input_step: Ok(0),
        }
    }
    fn view(&self, ctx: &Context<Self>) -> Html {
        let props = ctx.props().clone();
        let onchange_input = ctx.link().callback(|e: Event| {
            let value: HtmlInputElement = e.target_unchecked_into();
            let str = value.value();
            ControlStepMsg::ChangeStep(str)
        });
        let onclick_input = {
            let step_number = if let Ok(u) = self.now_input_step {
                u
            } else {
                0
            };
            move |_| props.callback_step_usr.clone().emit(step_number)
        };
        let onclick_toggle = props.callback_toggle_autostep.clone();
        let now_parse_result = if let Ok(u) = self.now_input_step {
            html! {u}
        } else {
            html! {"parse error"}
        };
        html! {
            <>
                <input onchange={onchange_input}/> {now_parse_result} <br/>
                <button onclick={onclick_input}> {"step"} </button>
                <button onclick={move |_| onclick_toggle.emit(())}> {"toggle auto step"} </button>
                <> {if props.now_toggle_state {"on"} else {"off"}} </>
            </>
        }
    }
    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            ControlStepMsg::ChangeStep(index) => {
                self.now_input_step = index.parse().map_err(|_| ());
                true
            }
        }
    }
}

pub struct MachineView {
    machine: Option<LoC>,
    callback_on_log: Option<Callback<String>>,
    tick_active: bool,
    #[allow(dead_code)]
    tick_interval: Interval,
}

impl MachineView {
    fn send_log(&mut self, str: String) {
        if let Some(ref callback) = self.callback_on_log {
            callback.emit(str);
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum MachineMsg {
    LoadFromMachine(Box<LoC>),
    Step(usize),
    SetEventLog(Callback<String>),
    SetInput(Vec<(InPin, Bool)>),
    TickToggle,
    Tick,
}

#[derive(Debug, Default, Clone, PartialEq, Properties)]
pub struct MachineProps {}

impl Component for MachineView {
    type Message = MachineMsg;
    type Properties = MachineProps;
    fn create(ctx: &Context<Self>) -> Self {
        let callback = ctx.link().callback(|_| MachineMsg::Tick);
        let interval = Interval::new(1000, move || callback.emit(()));
        Self {
            machine: None,
            callback_on_log: None,
            tick_active: false,
            tick_interval: interval,
        }
    }
    fn view(&self, ctx: &Context<Self>) -> Html {
        let Some(machine) = &self.machine else {
            return html! {
                <>
                {"machine"} <br/>
                {"not found"}
                </>
            };
        };
        let callback_step_usr = ctx.link().callback(MachineMsg::Step);
        let callback_toggle_autostep = ctx.link().callback(|_| MachineMsg::TickToggle);
        let now_toggle_state = self.tick_active;
        let on_set_inputs = ctx.link().callback(|v| MachineMsg::SetInput(v));
        let all_input_name = machine.get_all_input_name();
        html! {
            <div class ="machine"> <br/>
                <ControlStepView callback_step_usr={callback_step_usr} callback_toggle_autostep={callback_toggle_autostep} now_toggle_state={now_toggle_state}/>
                <InputSetView input_anames={all_input_name} on_set={on_set_inputs}/>
                <LoCView lc = {machine.clone()}/>
            </div>
        }
    }
    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            MachineMsg::LoadFromMachine(loc) => {
                self.machine = Some(*loc);
            }
            MachineMsg::Step(n) => {
                let Some(machine) = &mut self.machine else {
                    return false;
                };
                for _ in 0..n {
                    machine.next()
                }
            }
            MachineMsg::Tick => {
                let Some(machine) = &mut self.machine else {
                    return false;
                };
                if self.tick_active {
                    machine.next()
                }
            }
            MachineMsg::TickToggle => {
                self.tick_active = !self.tick_active;
            }
            MachineMsg::SetEventLog(callback) => {
                self.callback_on_log = Some(callback);
            }
            MachineMsg::SetInput(v) => {
                let Some(machine) = &mut self.machine else {
                    return false;
                };
                for (i, b) in v {
                    let i = machine.getmut_input(&i).unwrap();
                    *i = b;
                }
            }
        }
        true
    }
}
