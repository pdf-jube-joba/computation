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
        html! {<div class="box">
            {"input edit"}
            {
                for self.inputs.iter().enumerate().map(|(i, (n, b))| {
                    let callback = ctx.link().callback(move |_| InputSetMsg::Rev(i));
                    html! {
                        <button onclick={callback}>
                            <StateView state={*b} rep={n.to_string()}/>
                        </button>
                    }
                })
            }
        </div>}
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

pub struct MachineView {
    machine: Option<LoC>,
    callback_on_log: Option<Callback<String>>,
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
}

#[derive(Debug, Default, Clone, PartialEq, Properties)]
pub struct MachineProps {}

impl Component for MachineView {
    type Message = MachineMsg;
    type Properties = MachineProps;
    fn create(ctx: &Context<Self>) -> Self {
        Self {
            machine: None,
            callback_on_log: None,
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
        let callback_step = ctx.link().callback(MachineMsg::Step);
        let on_set_inputs = ctx.link().callback(|v| MachineMsg::SetInput(v));
        let all_input_name = machine.get_all_input_name();
        html! {
            <div class ="machine"> <br/>
                <utils::view::ControlStepView on_step={callback_step}/>
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
                if let Some(log) = &self.callback_on_log {
                    log.emit(format!("machine: {n} step"))
                }
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
