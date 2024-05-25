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

#[derive(Debug, Clone, PartialEq, Properties)]
pub struct FinGraphProps {
    fingraph: FinGraph,
    detail: bool,
}

#[function_component(FinGraphView)]
pub fn fingraph_view(FinGraphProps { fingraph, detail }: &FinGraphProps) -> Html {
    let lcs: Vec<_> = fingraph
        .get_lc_names()
        .into_iter()
        .map(|n| {
            let lc = fingraph.get_lc(&n).unwrap();
            let name = lc.get_name();
            let inout = fingraph.get_lc_inpins(&n);
            (n.clone(), name, inout)
        })
        .collect();
    let input: Vec<_> = fingraph
        .get_inpins()
        .into_iter()
        .map(|i| (i.clone(), *fingraph.get_input(&i).unwrap()))
        .collect();
    let otput: Vec<_> = fingraph
        .get_otpins()
        .into_iter()
        .map(|o| (o.clone(), *fingraph.get_output(&o).unwrap()))
        .collect();
    html! {
        <div class="graph">
            {for input.into_iter().map(|(i, s)|{
                let rep = format!("[{i}]");
                html!{
                    <StateView state = {s} rep = {rep}/>
                }})
            } <br/>
            {for otput.into_iter().map(|(o, s)|{
                let rep = format!("[{o}]");
                html!{
                    <StateView state = {s} rep = {rep}/>
                }})
            } <br/>
            {if *detail {
                html!{
                    {for lcs.iter().map(|(usename, name, inout) |{
                        html!{
                            <>
                            <span>
                                {usename} {" "}
                                {name} {" "}
                                {for inout.iter().map(|(i, n , o, s)| {
                                    let rep = format!("{i}={n}.{o} ");
                                    html!{
                                        <StateView state = {*s} rep = {rep}/>
                                    }
                                })}
                            </span>
                            <br/>
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

#[derive(Debug, Clone, PartialEq, Properties)]
pub struct IteratorProps {
    iterator: logic_circuit::machine::Iter,
    detail: bool,
}

#[function_component(IteratorView)]
fn iterator_view(IteratorProps { iterator, detail }: &IteratorProps) -> Html {
    // let logic_circuit::machine::Iter {
    //     name,
    //     lc_init: _,
    //     lc_extended,
    //     next_edges: _,
    //     prev_edges: _,
    //     input,
    //     otput,
    // } = iterator;
    let input: Vec<_> = iterator
        .get_inpins()
        .into_iter()
        .map(|i| (i.clone(), iterator.get_input(&i).unwrap()))
        .collect();
    let otput: Vec<_> = iterator
        .get_otpins()
        .into_iter()
        .map(|o| (o.clone(), iterator.get_otput(&o).unwrap()))
        .collect();
    html! {
        <div class="iterator">
            {for input.iter().map(|(i, i0)|{
                let s = format!("{i}={i0} ");
                html!{
                    <StateView state = {*iterator.get_input(i).unwrap()} rep = {s}/>
                }})
            } <br/>
            {for otput.iter().map(|(o, o0)|{
                let s = format!("{o}={o0} ");
                html!{
                    <StateView state= {*iterator.get_otput(o).unwrap()} rep = {s}/>
                }
            })} <br/>
            {if *detail {
                html!{for iterator.get_lcs().into_iter().map(|lc| html!{
                    <LoCView lc={lc.clone()}/>
                })}
            } else {
                html!{}
            }}
        </div>
    }
}

pub struct LoCView {
    pub detail: bool,
}

pub enum LoCMsg {
    ToggleDetail,
}

#[derive(Debug, Clone, PartialEq, Eq, Properties)]
pub struct LoCProps {
    pub lc: LoC,
}

impl Component for LoCView {
    type Message = LoCMsg;
    type Properties = LoCProps;
    fn create(ctx: &Context<Self>) -> Self {
        Self { detail: false }
    }
    fn view(&self, ctx: &Context<Self>) -> Html {
        let detail = self.detail;
        let LoCProps { lc } = ctx.props();

        let html = html! {
            html!{
                <>
                <button onclick={ctx.link().callback(|_| LoCMsg::ToggleDetail)}> {"detail"} </button>
                </>
            }
        };
        match lc {
            LoC::Gate(gate) => {
                html! {}
            }
            LoC::FinGraph(name, fingraph) => {
                html! {
                    <>
                    {html}
                    <FinGraphView fingraph={fingraph.as_ref().clone()} detail={detail}/>
                    </>
                }
            }
            LoC::Iter(name, iter) => {
                html! {
                    <>
                    {html}
                    <IteratorView iterator={iter.clone()} detail={detail}/>
                    </>
                }
            }
        }
    }
    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        self.detail = !self.detail;
        true
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
            on_set: _,
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
            input_anames: _,
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
    fn create(_ctx: &Context<Self>) -> Self {
        Self {
            machine: None,
            callback_on_log: None,
        }
    }
    fn view(&self, ctx: &Context<Self>) -> Html {
        let html = {
            match &self.machine {
                None => html! {<> {"not found"} </>},
                Some(machine) => {
                    let callback_step = ctx.link().callback(MachineMsg::Step);
                    let on_set_inputs = ctx.link().callback(MachineMsg::SetInput);
                    let all_input_name = machine.get_inpins();
                    html! {
                        <>
                            <utils::view::ControlStepView on_step={callback_step}/>
                            <InputSetView input_anames={all_input_name} on_set={on_set_inputs}/>
                            <LoCView lc = {machine.clone()}/>
                        </>
                    }
                }
            }
        };
        html! {
            <div class="machine">
            {"machine"} <br/>
            {html}
            </div>
        }
    }
    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
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

pub mod svg_lc {
    use super::*;
    use std::{collections::HashMap, fmt::Display};
    use utils::view::svg::*;
    use web_sys::Element;

    const BOOL_T_COL: &str = "red";
    const BOOL_F_COL: &str = "blue";

    const WIDTH_LC: usize = 50;
    const PIN_LEN: usize = 10;
    const PIN_RAD: usize = 5;

    #[derive(Debug, Clone, PartialEq, Properties)]
    struct LoCProps {
        loc: LoC,
        ori: Ori,
        pos: Pos, // center of loc
    }

    fn rot(mut diff: Diff, ori: Ori) -> Diff {
        match ori {
            Ori::U => diff,
            Ori::D => Diff(-diff.0, -diff.1),
            Ori::L => {
                diff.rot_counterclockwise();
                diff
            }
            Ori::R => {
                diff.rot_clockwise();
                diff
            }
        }
    }

    fn input_pos(pos: Pos, len_of_input: usize, num: usize, ori: Ori) -> Pos {
        let diff = Diff(WIDTH_LC as isize, (PIN_LEN * len_of_input) as isize);
        let diff_i = Diff(0, (PIN_LEN * num + PIN_RAD) as isize);
        pos - rot(diff / 2, ori) + rot(diff_i / 2, ori)
    }

    fn otput_pos(pos: Pos, len_of_otput: usize, num: usize, ori: Ori) -> Pos {
        let diff = Diff(-(WIDTH_LC as isize), (PIN_LEN * len_of_otput) as isize);
        let diff_i = Diff(0, (PIN_LEN * num + PIN_RAD) as isize);
        pos - rot(diff / 2, ori) + rot(diff_i / 2, ori)
    }

    #[function_component(LoCView)]
    fn lc_view(LoCProps { loc, ori, pos }: &LoCProps) -> Html {
        let input: Vec<_> = loc
            .get_inpins()
            .into_iter()
            .map(|i| (i.clone(), loc.get_input(&i).unwrap()))
            .collect();
        let otput: Vec<_> = loc
            .get_otpins()
            .into_iter()
            .map(|o| (o.clone(), loc.get_output(&o).unwrap()))
            .collect();
        let m = std::cmp::max(input.len(), otput.len());
        let diff = Diff(WIDTH_LC as isize, (PIN_LEN * m) as isize);

        html! {
            <>
                <RectView pos={*pos - rot(diff / 2, *ori)} diff={rot(diff, *ori)} col={"lightgray".to_string()} border={"black".to_string()}/>
                {for input.iter().enumerate().map(|(i, (inpin, state))|{
                    html!{
                        <>
                        <CircleView pos={input_pos(*pos, m, i, *ori)} rad={PIN_RAD} col={if **state == Bool::T {BOOL_T_COL.to_string()} else {BOOL_F_COL.to_string()}} border="black"/>
                        <TextView pos={input_pos(*pos, m, i, *ori)} text={inpin.to_string()}/>
                        </>
                    }
                })}
                {for otput.iter().enumerate().map(|(i, (otpin, state))|{
                    html!{
                        <>
                        <CircleView pos={otput_pos(*pos, m, i, *ori)} rad={PIN_RAD} col={if **state == Bool::T {BOOL_T_COL.to_string()} else {BOOL_F_COL.to_string()}} border="black"/>
                        <TextView pos={otput_pos(*pos, m, i, *ori)} text={otpin.to_string()}/>
                        </>
                    }
                })}
            </>
        }
    }

    #[derive(Debug, Clone, PartialEq, Properties)]
    pub struct ActLoCProps {
        pub fingraph: FinGraph,
        pub pos_lc: HashMap<Name, (Ori, Pos)>,
    }

    #[function_component(ActLoCView)]
    pub fn actlc_view(ActLoCProps { fingraph, pos_lc }: &ActLoCProps) -> Html {
        html! {
            <svg width="800" height="500" viewBox="0 0 800 500">
            {for pos_lc.iter().map(|(name, (ori, pos))|{
                let loc = fingraph.get_lc(name).unwrap();
                html!{
                    <LoCView loc={loc.clone()} ori={*ori} pos={*pos}/>
                }
            })}
            {for fingraph.edges().iter().map(|((no, o), (ni, i))|{
                let loc_o = fingraph.get_lc(no).unwrap();
                let (ori_o, pos_o) = pos_lc.get(no).unwrap();
                let otpins = loc_o.get_otpins();
                let num_o = otpins.iter().position(|otpin| o == otpin).unwrap();
                let pos_o = otput_pos(*pos_o, otpins.len(), num_o, *ori_o);

                let loc_i = fingraph.get_lc(ni).unwrap();
                let (ori_i, pos_i) = pos_lc.get(ni).unwrap();
                let inpins = loc_i.get_inpins();
                let num_i = loc_i.get_inpins().iter().position(|inpin| i == inpin).unwrap();
                let pos_i = input_pos(*pos_i, inpins.len(), num_i, *ori_i);

                let state = *loc_i.get_input(i).unwrap();

                html!{
                    <PolyLineView vec={vec![pos_o, pos_i]} col={if state == Bool::T {BOOL_T_COL} else {BOOL_F_COL}}/>
                }
            })}
            </svg>
        }
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    struct FingraphMachine {
        pub fingraph: FinGraph,
        pub pos_lc: HashMap<Name, (Ori, Pos)>,
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    enum FingraphMachineMsg {
        Step(usize),
    }

    #[derive(Debug, Clone, PartialEq, Eq, Properties)]
    struct FingraphMachineProps {}

    impl Component for FingraphMachine {
        type Message = FingraphMachineMsg;
        type Properties = FingraphMachineProps;
        fn create(ctx: &Context<Self>) -> Self {
            todo!()
        }
        fn view(&self, ctx: &Context<Self>) -> Html {
            let on_step = ctx.link().callback(FingraphMachineMsg::Step);
            html! {
                <>
                <ActLoCView fingraph={self.fingraph.clone()} pos_lc={self.pos_lc.clone()}/>
                <utils::view::ControlStepView {on_step}/>
                </>
            }
        }
        fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
            match msg {
                FingraphMachineMsg::Step(step) => {
                    for _ in 0..step {
                        self.fingraph.next();
                    }
                    true
                }
            }
        }
    }
}
