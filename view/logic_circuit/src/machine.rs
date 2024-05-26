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

    #[derive(Debug, Clone, PartialEq, Eq, Properties)]
    struct InPinProps {
        pos: Pos,
        state: Bool,
        inpin: InPin,
    }

    #[function_component(InPinView)]
    fn inpin_view(InPinProps { pos, state, inpin }: &InPinProps) -> Html {
        html! {
            <>
            <CircleView pos={pos.clone()} rad={PIN_RAD} col={if *state == Bool::T {BOOL_T_COL.to_string()} else {BOOL_F_COL.to_string()}} border="black"/>
            <TextView pos={pos.clone()} text={inpin.to_string()}/>
            </>
        }
    }

    #[derive(Debug, Clone, PartialEq, Eq, Properties)]
    struct OtPinProps {
        pos: Pos,
        state: Bool,
        otpin: OtPin,
    }

    #[function_component(OtPinView)]
    fn inpin_view(OtPinProps { pos, state, otpin }: &OtPinProps) -> Html {
        html! {
            <>
            <CircleView pos={pos.clone()} rad={PIN_RAD} col={if *state == Bool::T {BOOL_T_COL.to_string()} else {BOOL_F_COL.to_string()}} border="black"/>
            <TextView pos={pos.clone()} text={otpin.to_string()}/>
            </>
        }
    }

    #[derive(Debug, Clone, PartialEq, Properties)]
    struct LoCProps {
        inputs: Vec<(InPin, Bool)>,
        otputs: Vec<(OtPin, Bool)>,
        ori: Ori,
        pos: Pos, // center of loc
    }

    impl LoCProps {
        fn input_pos(&self, inpin: &InPin) -> Option<Pos> {
            let diff = Diff(WIDTH_LC as isize, (PIN_LEN * self.inputs.len()) as isize);
            let diff_i = Diff(
                0,
                (PIN_LEN * self.inputs.iter().position(|i| i.0 == *inpin)? + PIN_RAD) as isize,
            );
            Some(self.pos - rot(diff / 2, self.ori) + rot(diff_i / 2, self.ori))
        }

        fn otput_pos(&self, otpin: &OtPin) -> Option<Pos> {
            let diff = Diff(WIDTH_LC as isize, (PIN_LEN * self.otputs.len()) as isize);
            let diff_i = Diff(
                0,
                (PIN_LEN * self.otputs.iter().position(|o| o.0 == *otpin)? + PIN_RAD) as isize,
            );
            Some(self.pos - rot(diff / 2, self.ori) + rot(diff_i / 2, self.ori))
        }

        fn rect_lu(&self) -> Pos {
            let m = std::cmp::max(self.inputs.len(), self.otputs.len());
            let diff = Diff(WIDTH_LC as isize, (PIN_LEN * m) as isize);
            self.pos - rot(diff / 2, self.ori)
        }

        fn rect_diff(&self) -> Diff {
            let m = std::cmp::max(self.inputs.len(), self.otputs.len());
            let diff = Diff(WIDTH_LC as isize, (PIN_LEN * m) as isize);
            rot(diff, self.ori)
        }
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

    #[function_component(LoCView)]
    fn lc_view(locprops: &LoCProps) -> Html {
        html! {
            <>
                <RectView pos={locprops.rect_lu()} diff={locprops.rect_diff()} col={"lightgray".to_string()} border={"black".to_string()}/>
                {for locprops.inputs.iter().map(|(inpin, state)|{
                    html!{
                        <InPinView pos={locprops.input_pos(&inpin).unwrap()} state={*state} inpin={inpin.clone()}/>
                    }
                })}
                {for locprops.otputs.iter().map(|(otpin, state)|{
                    html!{
                        <OtPinView pos={locprops.otput_pos(&otpin).unwrap()} state={*state} otpin={otpin.clone()}/>
                    }
                })}
            </>
        }
    }

    #[derive(Debug, Clone, PartialEq, Properties)]
    pub struct ActLoCProps {
        pub fingraph: FinGraph,
        pub poslc: HashMap<Name, (Ori, Pos)>,
        pub on_inpin_clicks: Callback<InPin>,
    }

    impl ActLoCProps {
        fn get_lc_props(&self, name: &Name) -> Option<LoCProps> {
            let loc = self.fingraph.get_lc(name)?;
            let (ori, pos) = self.poslc.get(&name).unwrap();
            Some(LoCProps {
                inputs: loc
                    .get_inpins()
                    .into_iter()
                    .map(|i| (i.clone(), *loc.get_input(&i).unwrap()))
                    .collect(),
                otputs: loc
                    .get_otpins()
                    .into_iter()
                    .map(|o| (o.clone(), *loc.get_output(&o).unwrap()))
                    .collect(),
                ori: *ori,
                pos: *pos,
            })
        }
    }

    #[function_component(ActLoCView)]
    pub fn actlc_view(actlocprops: &ActLoCProps) -> Html {
        let graph_edge: Vec<(Pos, Pos, Bool)> = actlocprops
            .fingraph
            .edges()
            .iter()
            .map(|((no, o), (ni, i))| {
                let loc_o_props = actlocprops.get_lc_props(no).unwrap();
                let pos_o = loc_o_props.otput_pos(o).unwrap();

                let loc_i_props = actlocprops.get_lc_props(ni).unwrap();
                let pos_i = loc_i_props.input_pos(i).unwrap();

                let state = *actlocprops
                    .fingraph
                    .get_lc(ni)
                    .unwrap()
                    .get_input(i)
                    .unwrap();

                (pos_o, pos_i, state)
            })
            .collect();

        let inpin_edge: Vec<(Pos, Pos, Bool)> = actlocprops
            .fingraph
            .get_inpins()
            .into_iter()
            .enumerate()
            .map(|(k, i)| {
                let (n, inpin) = actlocprops
                    .fingraph
                    .get_inpin_to_lc_inpin(&i)
                    .unwrap()
                    .clone();
                let loc_props = actlocprops.get_lc_props(&n).unwrap();
                let state = loc_props
                    .inputs
                    .iter()
                    .find(|(i, _)| *i == inpin)
                    .unwrap()
                    .1;
                (
                    Pos(0, k * PIN_LEN + PIN_RAD),
                    loc_props.input_pos(&inpin).unwrap(),
                    state,
                )
            })
            .collect();

        let otpin_edge: Vec<(Pos, Pos, Bool)> = actlocprops
            .fingraph
            .get_otpins()
            .into_iter()
            .enumerate()
            .map(|(k, o)| {
                let (n, otpin) = actlocprops
                    .fingraph
                    .get_otpin_to_lc_otpin(&o)
                    .unwrap()
                    .clone();
                let loc_props = actlocprops.get_lc_props(&n).unwrap();
                let state = loc_props
                    .otputs
                    .iter()
                    .find(|(o, _)| *o == otpin)
                    .unwrap()
                    .1;
                (
                    Pos(800, k * PIN_LEN + PIN_RAD),
                    loc_props.otput_pos(&otpin).unwrap(),
                    state,
                )
            })
            .collect();
        html! {
            <svg width="800" height="500" viewBox="0 0 800 500">
            {for actlocprops.poslc.iter().map(|(name, (ori, pos))|{
                let loc = actlocprops.fingraph.get_lc(name).unwrap();
                let inputs: Vec<_> = loc.get_inpins().into_iter().map(|i|(i.clone(), *loc.get_input(&i).unwrap())).collect();
                let otputs: Vec<_> = loc.get_otpins().into_iter().map(|o|(o.clone(), *loc.get_output(&o).unwrap())).collect();
                html!{
                    <LoCView {inputs} {otputs} ori={*ori} pos={*pos}/>
                }
            })}
            {for inpin_edge.into_iter().map(|(pos_i, pos_ni, state)|{
                html!{
                    <PolyLineView vec={vec![pos_i, pos_ni]} col={if state == Bool::T {BOOL_T_COL} else {BOOL_F_COL}}/>
                }
            })
            }
            {for otpin_edge.into_iter().map(|(pos_o, pos_no, state)|{
                html!{
                    <PolyLineView vec={vec![pos_no, pos_o]} col={if state == Bool::T {BOOL_T_COL} else {BOOL_F_COL}}/>
                }
            })}
            {for graph_edge.into_iter().map(|(pos_o, pos_i, state)|{

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
        ToggleIn(InPin),
    }

    #[derive(Debug, Clone, PartialEq, Eq, Properties)]
    pub struct FingraphMachineProps {
        pub init_fingraph: FinGraph,
        pub init_pos_lc: Vec<(Name, (Ori, Pos))>,
    }

    impl Component for FingraphMachine {
        type Message = FingraphMachineMsg;
        type Properties = FingraphMachineProps;
        fn create(ctx: &Context<Self>) -> Self {
            let FingraphMachineProps {
                init_fingraph,
                init_pos_lc,
            } = ctx.props();
            Self {
                fingraph: init_fingraph.clone(),
                pos_lc: init_pos_lc.iter().cloned().collect(),
            }
        }
        fn view(&self, ctx: &Context<Self>) -> Html {
            let on_step = ctx.link().callback(FingraphMachineMsg::Step);
            let on_inpin_clicks = ctx.link().callback(FingraphMachineMsg::ToggleIn);
            html! {
                <>
                <ActLoCView fingraph={self.fingraph.clone()} poslc={self.pos_lc.clone()} {on_inpin_clicks}/>
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
                FingraphMachineMsg::ToggleIn(inpin) => {
                    let input = self.fingraph.getmut_input(&inpin).unwrap();
                    *input = input.neg();
                    true
                }
            }
        }
    }
}
