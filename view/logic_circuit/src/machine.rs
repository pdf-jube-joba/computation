use std::ops::Neg;
use yew::prelude::*;

use logic_circuit::machine::{Bool, FinGraph, Gate, InPin, LoC, Name, OtPin};

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
    html! {
        <span class={state_class}>
            {rep}
        </span>
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Ori {
    U,
    D,
    L,
    R,
}

type Pos = (usize, usize);

#[derive(Debug, Clone, PartialEq, Properties)]
pub struct GateProps {
    gate: Gate,
    ori: Ori,
    pos: Pos,
}

#[function_component(GateView)]
fn gate_view(GateProps { gate, ori, pos }: &GateProps) -> Html {
    html! {
        <div class="gate" top={pos.0.to_string()} left={pos.1.to_string()}>
            {gate.name()}
        </div>
    }
}

type Path = Vec<(usize, usize)>;

#[derive(Debug, Clone, PartialEq, Properties)]
pub struct FinGraphProps {
    fingraph: FinGraph,
    names: Vec<(Name, Ori, Pos)>,
    edges: Vec<((Name, InPin), (Name, OtPin), Path)>,
}

#[function_component(FinGraphView)]
pub fn fingraph_view(FinGraphProps { fingraph, names, edges }: &FinGraphProps) -> Html {
    
    todo!()
}
//     // let FinGraph {
//     //     name,
//     //     lcs,
//     //     edges: _,
//     //     input,
//     //     output,
//     // } = fingraph;
//     html! {
//         <div class="graph">
//             {name} <br/>
//             {for input.iter().map(|(i, (name, i0))|{
//                 let s = format!("{i}={name}.{i0} ");
//                 html!{
//                     <StateView state = {*fingraph.get_input(i).unwrap()} rep = {s}/>
//                 }})
//             } <br/>
//             {for output.iter().map(|(o, (name, o0))|{
//                 let s = format!("{o}={name}.{o0} ");
//                 html!{
//                     <StateView state = {*fingraph.get_output(o).unwrap()} rep = {s}/>
//                 }})
//             } <br/>
//             {if *detail {
//                 html!{
//                     {for lcs.iter().map(|(name, _)|{
//                         let (lc, inout) = fingraph.get_lc_inouts(name).unwrap();
//                         html!{
//                             <>
//                             <span>
//                                 {name} {" "}
//                                 {lc.name()} {" "}
//                                 {for inout.iter().map(|(i, (_, o), s)| {
//                                     let name = format!("{i}={name}.{o} ");
//                                     html!{
//                                         <StateView state = {*s} rep = {name}/>
//                                     }
//                                 })}
//                             </span>
//                             <br/>
//                             </>
//                         }
//                     })}
//                 }
//             } else {
//                 html!{}
//             }}
//         </div>
//     }
// }

// #[derive(Debug, Clone, PartialEq, Properties)]
// pub struct IteratorProps {
//     iterator: logic_circuit::machine::Iter,
//     detail: bool,
// }

// #[function_component(IteratorView)]
// fn iterator_view(IteratorProps { iterator, detail }: &IteratorProps) -> Html {
//     let logic_circuit::machine::Iter {
//         name,
//         lc_init: _,
//         lc_extended,
//         next_edges: _,
//         prev_edges: _,
//         input,
//         otput,
//     } = iterator;
//     html! {
//         <div class="iterator">
//             {name} <br/>
//             {for input.iter().map(|(i, i0)|{
//                 let s = format!("{i}={i0} ");
//                 html!{
//                     <StateView state = {*iterator.get_input(i).unwrap()} rep = {s}/>
//                 }})
//             } <br/>
//             {for otput.iter().map(|(o, o0)|{
//                 let s = format!("{o}={o0} ");
//                 html!{
//                     <StateView state= {*iterator.get_otput(o).unwrap()} rep = {s}/>
//                 }
//             })} <br/>
//             {if *detail {
//                 html!{for lc_extended.iter().map(|lc| html!{
//                     <LoCView lc={lc.clone()}/>
//                 })}
//             } else {
//                 html!{}
//             }}
//         </div>
//     }
// }

// pub struct LoCView {
//     pub detail: bool,
// }

// pub enum LoCMsg {
//     ToggleDetail,
// }

// #[derive(Debug, Clone, PartialEq, Eq, Properties)]
// pub struct LoCProps {
//     pub lc: LoC,
// }

// impl Component for LoCView {
//     type Message = LoCMsg;
//     type Properties = LoCProps;
//     fn create(ctx: &Context<Self>) -> Self {
//         Self { detail: false }
//     }
//     fn view(&self, ctx: &Context<Self>) -> Html {
//         let detail = self.detail;
//         let LoCProps { lc } = ctx.props();

//         let html = html! {
//             html!{
//                 <>
//                 <button onclick={ctx.link().callback(|_| LoCMsg::ToggleDetail)}> {"detail"} </button>
//                 </>
//             }
//         };
//         match lc {
//             LoC::Gate(gate) => {
//                 html! {}
//             }
//             LoC::FinGraph(fingraph) => {
//                 html! {
//                     <>
//                     {html}
//                     <FinGraphView fingraph={fingraph.as_ref().clone()} detail={detail}/>
//                     </>
//                 }
//             }
//             LoC::Iter(iter) => {
//                 html! {
//                     <>
//                     {html}
//                     <IteratorView iterator={iter.clone()} detail={detail}/>
//                     </>
//                 }
//             }
//         }
//     }
//     fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
//         self.detail = !self.detail;
//         true
//     }
// }

// pub struct InputSetView {
//     inputs: Vec<(InPin, Bool)>,
// }

// #[derive(Debug)]
// pub enum InputSetMsg {
//     Rev(usize),
// }

// #[derive(Clone, PartialEq, Properties)]
// pub struct InputSetProps {
//     input_anames: Vec<InPin>,
//     on_set: Callback<Vec<(InPin, Bool)>>,
// }

// impl Component for InputSetView {
//     type Message = InputSetMsg;
//     type Properties = InputSetProps;
//     fn create(ctx: &Context<Self>) -> Self {
//         let InputSetProps {
//             input_anames,
//             on_set: _,
//         } = ctx.props();
//         let inputs = input_anames
//             .iter()
//             .map(|n| (n.clone(), Bool::F))
//             .collect::<Vec<_>>();
//         Self { inputs }
//     }
//     fn view(&self, ctx: &Context<Self>) -> Html {
//         html! {<div class="box">
//             {"input edit"}
//             {
//                 for self.inputs.iter().enumerate().map(|(i, (n, b))| {
//                     let callback = ctx.link().callback(move |_| InputSetMsg::Rev(i));
//                     html! {
//                         <button onclick={callback}>
//                             <StateView state={*b} rep={n.to_string()}/>
//                         </button>
//                     }
//                 })
//             }
//         </div>}
//     }
//     fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
//         let InputSetMsg::Rev(i) = msg;
//         let (_, b) = self.inputs.get_mut(i).unwrap();
//         *b = b.neg();
//         let InputSetProps {
//             input_anames: _,
//             on_set,
//         } = ctx.props();
//         on_set.emit(self.inputs.clone());
//         true
//     }
// }

// pub struct MachineView {
//     machine: Option<LoC>,
//     callback_on_log: Option<Callback<String>>,
// }

// #[derive(Debug, Clone, PartialEq)]
// pub enum MachineMsg {
//     LoadFromMachine(Box<LoC>),
//     Step(usize),
//     SetEventLog(Callback<String>),
//     SetInput(Vec<(InPin, Bool)>),
// }

// #[derive(Debug, Default, Clone, PartialEq, Properties)]
// pub struct MachineProps {}

// impl Component for MachineView {
//     type Message = MachineMsg;
//     type Properties = MachineProps;
//     fn create(_ctx: &Context<Self>) -> Self {
//         Self {
//             machine: None,
//             callback_on_log: None,
//         }
//     }
//     fn view(&self, ctx: &Context<Self>) -> Html {
//         let html = {
//             match &self.machine {
//                 None => html! {<> {"not found"} </>},
//                 Some(machine) => {
//                     let callback_step = ctx.link().callback(MachineMsg::Step);
//                     let on_set_inputs = ctx.link().callback(MachineMsg::SetInput);
//                     let all_input_name = machine.get_all_input_name();
//                     html! {
//                         <>
//                             <utils::view::ControlStepView on_step={callback_step}/>
//                             <InputSetView input_anames={all_input_name} on_set={on_set_inputs}/>
//                             <LoCView lc = {machine.clone()}/>
//                         </>
//                     }
//                 }
//             }
//         };
//         html! {
//             <div class="machine">
//             {"machine"} <br/>
//             {html}
//             </div>
//         }
//     }
//     fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
//         match msg {
//             MachineMsg::LoadFromMachine(loc) => {
//                 self.machine = Some(*loc);
//             }
//             MachineMsg::Step(n) => {
//                 let Some(machine) = &mut self.machine else {
//                     return false;
//                 };
//                 for _ in 0..n {
//                     machine.next()
//                 }
//                 if let Some(log) = &self.callback_on_log {
//                     log.emit(format!("machine: {n} step"))
//                 }
//             }
//             MachineMsg::SetEventLog(callback) => {
//                 self.callback_on_log = Some(callback);
//             }
//             MachineMsg::SetInput(v) => {
//                 let Some(machine) = &mut self.machine else {
//                     return false;
//                 };
//                 for (i, b) in v {
//                     let i = machine.getmut_input(&i).unwrap();
//                     *i = b;
//                 }
//             }
//         }
//         true
//     }
// }
