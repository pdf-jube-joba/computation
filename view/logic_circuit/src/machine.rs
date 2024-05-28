use std::ops::Neg;
use yew::prelude::*;

use logic_circuit::machine::{Bool, FinGraph, Gate, InPin, LoC, Name, OtPin};

// #[derive(Debug, Clone, PartialEq, Eq, Properties)]
// pub struct StateProps {
//     state: Bool,
//     rep: String,
// }

// #[function_component(StateView)]
// pub fn state_view(StateProps { state, rep }: &StateProps) -> Html {
//     let state_class = match state {
//         Bool::T => "stateT",
//         Bool::F => "stateF",
//     };
//     html! {
//         <span class={state_class}>
//             {rep}
//         </span>
//     }
// }

// #[derive(Debug, Clone, PartialEq, Properties)]
// pub struct FinGraphProps {
//     fingraph: FinGraph,
//     detail: bool,
// }

// #[function_component(FinGraphView)]
// pub fn fingraph_view(FinGraphProps { fingraph, detail }: &FinGraphProps) -> Html {
//     let lcs: Vec<_> = fingraph
//         .get_lc_names()
//         .into_iter()
//         .map(|n| {
//             let lc = fingraph.get_lc(&n).unwrap();
//             let name = lc.get_name();
//             let inout = fingraph.get_lc_inpins(&n);
//             (n.clone(), name, inout)
//         })
//         .collect();
//     let input: Vec<_> = fingraph.get_inpins();
//     let otput: Vec<_> = fingraph.get_otpins();
//     html! {
//         <div class="graph">
//             {for input.into_iter().map(|(i, s)|{
//                 let rep = format!("[{i}]");
//                 html!{
//                     <StateView state = {s} rep = {rep}/>
//                 }})
//             } <br/>
//             {for otput.into_iter().map(|(o, s)|{
//                 let rep = format!("[{o}]");
//                 html!{
//                     <StateView state = {s} rep = {rep}/>
//                 }})
//             } <br/>
//             {if *detail {
//                 html!{
//                     {for lcs.iter().map(|(usename, name, inout) |{
//                         html!{
//                             <>
//                             <span>
//                                 {usename} {" "}
//                                 {name} {" "}
//                                 {for inout.iter().map(|(i, n , o, s)| {
//                                     let rep = format!("{i}={n}.{o} ");
//                                     html!{
//                                         <StateView state = {*s} rep = {rep}/>
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
//     let input: Vec<_> = iterator.get_inpins();
//     let otput: Vec<_> = iterator.get_otpins();
//     html! {
//         <div class="iterator">
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
//                 html!{for iterator.get_lcs().into_iter().map(|lc| html!{
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
//             LoC::FinGraph(name, fingraph) => {
//                 html! {
//                     <>
//                     {html}
//                     <FinGraphView fingraph={fingraph.as_ref().clone()} detail={detail}/>
//                     </>
//                 }
//             }
//             LoC::Iter(name, iter) => {
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
//                     let all_input_name = machine
//                         .get_inpins()
//                         .into_iter()
//                         .map(|v| v.0)
//                         .collect::<Vec<_>>();
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

pub mod svg_lc {
    use super::*;
    use anyhow::{bail, Result};
    use either::Either;
    use std::{collections::HashMap, fmt::Display};
    use utils::view::{log, svg::*};
    use web_sys::Element;

    const BOOL_T_COL: &str = "red";
    const BOOL_F_COL: &str = "blue";

    const WIDTH_LC: usize = 50;
    const PIN_LEN: usize = 25;
    const PIN_RAD: usize = 10;

    const IN_LINE: usize = 50;
    const OT_LINE: usize = 750;
    const UP_LINE: usize = 100;

    const COMP_LINE: usize = 400;
    const COMP_LEN: usize = 100;

    #[derive(Debug, Clone, PartialEq, Properties)]
    struct InPinProps {
        pos: Pos,
        state: Bool,
        inpin: InPin,
        onmousedown: Callback<()>,
    }

    #[function_component(InPinView)]
    fn inpin_view(
        InPinProps {
            pos,
            state,
            inpin,
            onmousedown,
        }: &InPinProps,
    ) -> Html {
        let onmousedown = onmousedown.clone();
        html! {
            <>
            <CircleView pos={pos.clone()} rad={PIN_RAD} col={if *state == Bool::T {BOOL_T_COL.to_string()} else {BOOL_F_COL.to_string()}} border="black" onmousedown={Callback::from(move |_|{onmousedown.emit(())})}/>
            <TextView pos={pos.clone()} text={inpin.to_string()}/>
            </>
        }
    }

    #[derive(Debug, Clone, PartialEq, Properties)]
    struct OtPinProps {
        pos: Pos,
        state: Bool,
        otpin: OtPin,
        onmousedown: Callback<()>,
    }

    #[function_component(OtPinView)]
    fn inpin_view(
        OtPinProps {
            pos,
            state,
            otpin,
            onmousedown,
        }: &OtPinProps,
    ) -> Html {
        let onmousedown = onmousedown.clone();
        html! {
            <>
            <CircleView pos={pos.clone()} rad={PIN_RAD} col={if *state == Bool::T {BOOL_T_COL.to_string()} else {BOOL_F_COL.to_string()}} border="black" onmousedown={Callback::from(move |_| onmousedown.emit(()))}/>
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
        onmousedownlc: Callback<Diff>,
        onmousedowninpin: Callback<usize>,
        onmousedownotpin: Callback<usize>,
        onrightclick: Callback<()>,
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

        fn input_pos_fromnum(&self, inpinnum: &usize) -> Option<Pos> {
            if self.inputs.len() <= *inpinnum {
                return None;
            }
            let diff = Diff(WIDTH_LC as isize, (PIN_LEN * self.inputs.len()) as isize);
            let diff_i = Diff(0, (PIN_LEN * inpinnum + PIN_RAD) as isize);

            Some(self.pos - rot(diff / 2, self.ori) + rot(diff_i / 2, self.ori))
        }

        fn otput_pos(&self, otpin: &OtPin) -> Option<Pos> {
            let diff = Diff(-(WIDTH_LC as isize), (PIN_LEN * self.otputs.len()) as isize);
            let diff_i = Diff(
                0,
                (PIN_LEN * self.otputs.iter().position(|o| o.0 == *otpin)? + PIN_RAD) as isize,
            );
            Some(self.pos - rot(diff / 2, self.ori) + rot(diff_i / 2, self.ori))
        }

        fn otput_pos_fromnum(&self, otpinnum: &usize) -> Option<Pos> {
            if self.otputs.len() <= *otpinnum {
                return None;
            }
            let diff = Diff(-(WIDTH_LC as isize), (PIN_LEN * self.otputs.len()) as isize);
            let diff_i = Diff(0, (PIN_LEN * otpinnum + PIN_RAD) as isize);
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
        let LoCProps {
            inputs,
            otputs,
            ori,
            pos,
            onmousedownlc,
            onmousedowninpin,
            onmousedownotpin,
            onrightclick,
        } = locprops.clone();
        let onmousedownlc = Callback::from(move |e: MouseEvent| {
            e.prevent_default();
            let pos_click = Pos(e.client_x() as usize, e.client_y() as usize);
            onmousedownlc.emit(pos_click - pos)
        });
        let onrightclick = Callback::from(move |e: MouseEvent| {
            e.prevent_default();
            onrightclick.emit(());
        });
        html! {
            <>
                <RectView pos={locprops.rect_lu()} diff={locprops.rect_diff()} col={"lightgray".to_string()} border={"black".to_string()} onmousedown={onmousedownlc} oncontextmenu={onrightclick}/>
                {for inputs.into_iter().enumerate().map(|(k, (inpin, state))|{
                    let onmousedown = onmousedowninpin.clone();
                    let onmousedown = Callback::from(move |_|{
                        onmousedown.emit(k)
                    });
                    html!{
                        <InPinView pos={locprops.input_pos(&inpin).unwrap()} {state} inpin={inpin.clone()} {onmousedown}/>
                    }
                })}
                {for otputs.into_iter().enumerate().map(|(k, (otpin, state))|{
                    let onmousedown = onmousedownotpin.clone();
                    let onmousedown = Callback::from(move |_|{
                        onmousedown.emit(k)
                    });
                    html!{
                        <OtPinView pos={locprops.otput_pos(&otpin).unwrap()} {state} otpin={otpin.clone()} {onmousedown}/>
                    }
                })}
            </>
        }
    }

    #[derive(Debug, Clone, PartialEq, Properties)]
    pub struct ActLoCProps {
        pub fingraph: FinGraph,
        pub poslc: HashMap<Name, (Pos, Ori)>,
        pub on_inpin_clicks: Callback<InPin>,
    }

    impl ActLoCProps {
        fn get_lc_props(&self, name: &Name) -> Option<LoCProps> {
            let loc = self.fingraph.get_lc(name)?;
            let (pos, ori) = self.poslc.get(&name).unwrap();
            Some(LoCProps {
                inputs: loc.get_inpins(),
                otputs: loc.get_otpins(),
                ori: *ori,
                pos: *pos,
                onmousedownlc: Callback::noop(),
                onmousedowninpin: Callback::noop(),
                onmousedownotpin: Callback::noop(),
                onrightclick: Callback::noop(),
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

        let inpin_edge: Vec<(InPin, Pos, Pos, Bool)> = actlocprops
            .fingraph
            .get_inpins()
            .into_iter()
            .enumerate()
            .map(|(k, (i, state))| {
                let (n, inpin) = actlocprops
                    .fingraph
                    .get_inpin_to_lc_inpin(&i)
                    .unwrap()
                    .clone();
                let loc_props = actlocprops.get_lc_props(&n).unwrap();
                (
                    i,
                    Pos(IN_LINE, k * PIN_LEN + UP_LINE),
                    loc_props.input_pos(&inpin).unwrap(),
                    state,
                )
            })
            .collect();

        let otpin_edge: Vec<(OtPin, Pos, Pos, Bool)> = actlocprops
            .fingraph
            .get_otpins()
            .into_iter()
            .enumerate()
            .map(|(k, (o, state))| {
                let (n, otpin) = actlocprops
                    .fingraph
                    .get_otpin_to_lc_otpin(&o)
                    .unwrap()
                    .clone();
                let loc_props = actlocprops.get_lc_props(&n).unwrap();
                (
                    o,
                    Pos(OT_LINE, k * PIN_LEN + UP_LINE),
                    loc_props.otput_pos(&otpin).unwrap(),
                    state,
                )
            })
            .collect();

        let inpins = actlocprops.fingraph.get_inpins();
        let callback = actlocprops.on_inpin_clicks.clone();
        let onclick = Callback::from(move |e: MouseEvent| {
            e.prevent_default();
            let pos: Pos = Pos(e.client_x() as usize, e.client_y() as usize);
            for k in 0..inpins.len() {
                let dist = pos.abs_diff(&Pos(IN_LINE, k * PIN_LEN + UP_LINE));
                if dist <= PIN_RAD.pow(2) {
                    callback.emit(inpins[k].0.clone());
                }
            }
        });
        html! {
            <svg width="800" height="500" viewBox="0 0 800 500" {onclick}>
            {for actlocprops.poslc.keys().map(|name|{
                let LoCProps { inputs, otputs, ori, pos, onmousedownlc, onmousedowninpin, onmousedownotpin, onrightclick } = actlocprops.get_lc_props(name).unwrap();
                html!{
                    <LoCView {inputs} {otputs} {ori} {pos} {onmousedownlc} {onmousedowninpin} {onmousedownotpin} {onrightclick}/>
                }
            })}
            {for inpin_edge.into_iter().map(|(inpin, pos_i, pos_ni, state)|{
                html!{
                    <>
                    <InPinView pos={pos_i} {inpin} {state} onmousedown={Callback::noop()}/>
                    <PolyLineView vec={vec![pos_i, pos_ni]} col={if state == Bool::T {BOOL_T_COL} else {BOOL_F_COL}}/>
                    </>
                }
            })
            }
            {for otpin_edge.into_iter().map(|(otpin, pos_o, pos_no, state)|{
                html!{
                    <>
                    <OtPinView pos={pos_o} {otpin} {state} onmousedown={Callback::noop()}/>
                    <PolyLineView vec={vec![pos_no, pos_o]} col={if state == Bool::T {BOOL_T_COL} else {BOOL_F_COL}}/>
                    </>
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
    pub struct FingraphMachine {
        pub fingraph: FinGraph,
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub enum FingraphMachineMsg {
        Step(usize),
        ToggleIn(InPin),
    }

    #[derive(Debug, Clone, PartialEq, Eq, Properties)]
    pub struct FingraphMachineProps {
        init_fingraph: FinGraph,
        init_pos_lc: Vec<(Name, (Pos, Ori))>,
    }

    impl FingraphMachineProps {
        pub fn new(init_fingraph: FinGraph, init_pos_lc: Vec<(Name, (Pos, Ori))>) -> Result<Self> {
            let names = init_fingraph.get_lc_names();
            for name in names {
                if init_pos_lc.iter().all(|(n, _)| n != &name) {
                    bail!("not found {name} in init_pos_lc");
                }
            }
            Ok(Self {
                init_fingraph,
                init_pos_lc,
            })
        }
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
            }
        }
        fn view(&self, ctx: &Context<Self>) -> Html {
            let on_step = ctx.link().callback(FingraphMachineMsg::Step);
            let on_inpin_clicks = ctx.link().callback(FingraphMachineMsg::ToggleIn);
            let poslc = ctx.props().init_pos_lc.clone();
            html! {
                <>
                <ActLoCView fingraph={self.fingraph.clone()} poslc={poslc.into_iter().collect::<HashMap<_,_>>()} {on_inpin_clicks}/>
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

    type InPinNum = usize;
    type OtPinNum = usize;

    type PinVariant =
        Either<Either<InPinNum, (usize, InPinNum)>, Either<OtPinNum, (usize, OtPinNum)>>;

    #[derive(Debug, Clone, PartialEq, Eq)]
    enum State {
        None,
        MoveLC(usize, Diff),
        SelectPin(PinVariant),
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct GraphicEditor {
        inpins: Vec<InPin>,
        otpins: Vec<OtPin>,
        component: Vec<(usize, Pos, Ori)>,
        edges: Vec<((usize, InPinNum), (usize, OtPinNum))>,
        inputs: Vec<(InPinNum, (usize, InPinNum))>,
        otputs: Vec<(OtPinNum, (usize, OtPinNum))>,
        state: State,
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub enum GraphicEditorMsg {
        AddInpin(InPin),
        DeleteInPin(InPin),
        AddOtPin(OtPin),
        DeleteOtPin(OtPin),

        SelectCopy(usize, Diff),
        SelectMove(usize, Diff),
        SelectPin(PinVariant),
        UnSelect,
        Update(Pos),

        Delte(usize),

        GoToTest,
        None,
    }

    type AllPositions = (
        Vec<InPin>,
        Vec<OtPin>,
        Vec<(usize, Pos, Ori)>,
        Vec<((usize, InPinNum), (usize, OtPinNum))>,
        Vec<(InPinNum, (usize, InPinNum))>,
        Vec<(OtPinNum, (usize, OtPinNum))>,
    );

    #[derive(Debug, Clone, PartialEq, Properties)]
    pub struct GraphicEditorProps {
        pub logic_circuits_components: Vec<LoC>,
        pub on_goto_test: Callback<(FinGraph, Vec<(Name, (Pos, Ori))>)>,
        pub on_log: Callback<String>,
        pub maybe_initial_locpos: Option<AllPositions>,
    }

    impl Component for GraphicEditor {
        type Message = GraphicEditorMsg;
        type Properties = GraphicEditorProps;
        fn create(ctx: &Context<Self>) -> Self {
            let v = ctx.props().maybe_initial_locpos.clone();
            let (inpins, otpins, component, edges, inputs, otputs) = match v {
                None => (vec![], vec![], vec![], vec![], vec![], vec![]),
                Some((a, b, c, d, e, f)) => (a, b, c, d, e, f),
            };
            Self {
                inpins,
                otpins: vec![],
                component,
                edges,
                inputs,
                otputs,
                state: State::None,
            }
        }
        fn view(&self, ctx: &Context<Self>) -> Html {
            let GraphicEditor {
                inpins,
                otpins,
                component,
                edges,
                inputs,
                otputs,
                state,
            } = self.clone();
            let GraphicEditorProps {
                logic_circuits_components,
                on_goto_test,
                on_log,
                maybe_initial_locpos: _,
            } = ctx.props();

            let goto_test = ctx.link().callback(|_| GraphicEditorMsg::GoToTest);
            let add_inpins = ctx
                .link()
                .callback(|s: String| GraphicEditorMsg::AddInpin(s.into()));

            let remove_inpins = ctx
                .link()
                .callback(|s: String| GraphicEditorMsg::DeleteInPin(s.into()));

            let add_otpins = ctx
                .link()
                .callback(|s: String| GraphicEditorMsg::AddOtPin(s.into()));

            let remove_otpins = ctx
                .link()
                .callback(|s: String| GraphicEditorMsg::DeleteOtPin(s.into()));

            let onmousemove = ctx.link().callback(|e: MouseEvent| {
                e.prevent_default();
                let pos = Pos(e.client_x() as usize, e.client_y() as usize);
                GraphicEditorMsg::Update(pos)
            });

            let onmouseuporleave = ctx.link().callback(|e: MouseEvent| {
                e.prevent_default();
                GraphicEditorMsg::UnSelect
            });

            let loc_vec = component
                .iter()
                .enumerate()
                .map(|(k, (num, pos, ori))| {
                    let loc = &logic_circuits_components[*num];
                    let inputs = loc.get_inpins();
                    let otputs = loc.get_otpins();
                    let onmousedownlc = ctx
                        .link()
                        .callback(move |diff: Diff| GraphicEditorMsg::SelectMove(k, diff));
                    let onmousedowninpin = ctx.link().callback(move |num_i: usize| {
                        GraphicEditorMsg::SelectPin(Either::Left(Either::Right((k, num_i))))
                    });
                    let onmousedownotpin = ctx.link().callback(move |num_o: usize| {
                        GraphicEditorMsg::SelectPin(Either::Right(Either::Right((k, num_o))))
                    });
                    let onrightclick = ctx.link().callback(move |_| GraphicEditorMsg::Delte(k));
                    LoCProps {
                        inputs,
                        otputs,
                        pos: *pos,
                        ori: *ori,
                        onmousedownlc,
                        onmousedowninpin,
                        onmousedownotpin,
                        onrightclick,
                    }
                })
                .collect::<Vec<_>>();

            let inpins_edge = inputs
                .iter()
                .enumerate()
                .map(|(l, (i, (k, i1)))| {
                    let pos_i = Pos(IN_LINE, l * PIN_LEN + UP_LINE);
                    let pos_i2 = loc_vec[*k].input_pos_fromnum(i1).unwrap();
                    (pos_i, pos_i2)
                })
                .collect::<Vec<_>>();

            let otpins_edge = otputs
                .iter()
                .enumerate()
                .map(|(l, (o, (k, o1)))| {
                    let pos_o = Pos(OT_LINE, l * PIN_LEN + UP_LINE);
                    let pos_o2 = loc_vec[*k].otput_pos_fromnum(o1).unwrap();
                    (pos_o, pos_o2)
                })
                .collect::<Vec<_>>();

            let graph_edges = edges
                .iter()
                .map(|((ko, o), (ki, i))| {
                    let pos_o = loc_vec[*ko].otput_pos_fromnum(o).unwrap();
                    let pos_i = loc_vec[*ki].input_pos_fromnum(i).unwrap();
                    (pos_o, pos_i)
                })
                .collect::<Vec<_>>();

            html! {
                <>
                <svg width="800" height="500" viewBox="0 0 800 500" {onmousemove} onmouseup={onmouseuporleave.clone()} onmouseleave={onmouseuporleave}>
                    {for loc_vec.into_iter().map(|locprop|{
                        let LoCProps {inputs, otputs, pos, ori, onmousedownlc, onmousedowninpin, onmousedownotpin, onrightclick} = locprop;
                        html!{
                            <LoCView {inputs} {otputs} pos={pos} ori={ori} {onmousedownlc} {onmousedowninpin} {onmousedownotpin} {onrightclick}/>
                        }
                    })}
                    {for inpins.into_iter().enumerate().map(|(k, inpin)|{
                        let pos = Pos(IN_LINE, k * PIN_LEN + UP_LINE);
                        let onmousedown = ctx.link().callback(move |_|{
                            GraphicEditorMsg::SelectPin(Either::Left(Either::Left(k)))
                        });
                        html!{
                            <InPinView inpin={inpin.clone()} {pos} state={Bool::F} {onmousedown}/>
                        }
                    })}
                    {for otpins.into_iter().enumerate().map(|(k, otpin)|{
                        let pos = Pos(OT_LINE, k * PIN_LEN + UP_LINE);
                        let onmousedown = ctx.link().callback(move |_|{
                            GraphicEditorMsg::SelectPin(Either::Right(Either::Left(k)))
                        });
                        html!{
                            <OtPinView otpin={otpin.clone()} {pos} state={Bool::F} {onmousedown}/>
                        }
                    })}
                    {for inpins_edge.into_iter().map(|(pos_i, pos_i2)|{
                        html!{
                            <PolyLineView vec={vec![pos_i, pos_i2]} col={BOOL_F_COL}/>
                        }
                    })}
                    {for otpins_edge.into_iter().map(|(pos_o, pos_o2)|{
                        html!{
                            <PolyLineView vec={vec![pos_o, pos_o2]} col={BOOL_F_COL}/>
                        }
                    })}
                    {for graph_edges.into_iter().map(|(pos_o, pos_i)|{
                        html!{
                            <PolyLineView vec={vec![pos_o, pos_i]} col={BOOL_F_COL}/>
                        }
                    })}
                    {for logic_circuits_components.clone().into_iter().enumerate().map(|(k, loc)|{
                        let inputs = loc.get_inpins();
                        let otputs = loc.get_otpins();
                        let pos = Pos(k * COMP_LEN, COMP_LINE);
                        let onmousedownlc = ctx.link().callback(move |diff: Diff|{
                            GraphicEditorMsg::SelectCopy(k, diff)
                        });
                        html!{
                            <LoCView {inputs} {otputs} {pos} ori={Ori::U} {onmousedownlc} onmousedowninpin={Callback::noop()} onmousedownotpin={Callback::noop()} onrightclick={Callback::noop()}/>
                        }
                    })}
                </svg> <br/>
                <utils::view::InputText description={"add inpins".to_string()} on_push_load_button={add_inpins}/>
                <utils::view::InputText description={"add otpins".to_string()} on_push_load_button={add_otpins}/> <br/>
                <utils::view::InputText description={"remove inpins".to_string()} on_push_load_button={remove_inpins}/>
                <utils::view::InputText description={"remove otpins".to_string()} on_push_load_button={remove_otpins}/>
                // <button onclick={goto_test}> {"test"} </button>
                <utils::view::ButtonView on_click={goto_test} text={"test"}/>
                </>
            }
        }
        fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
            let prop = ctx.props();
            if !matches!(msg, GraphicEditorMsg::Update(_)) {
                prop.on_log.emit(format!("{msg:?} {:?}", self.state));
            }
            match msg {
                GraphicEditorMsg::AddInpin(inpin) => {
                    if self.inpins.iter().any(|i| *i == inpin) {
                        return false;
                    }
                    self.inpins.push(inpin);
                }
                GraphicEditorMsg::AddOtPin(otpin) => {
                    if self.otpins.iter().any(|o| *o == otpin) {
                        return false;
                    }
                    self.otpins.push(otpin);
                }
                GraphicEditorMsg::DeleteInPin(inpin) => {
                    self.inpins.retain(|i| *i != inpin);
                }
                GraphicEditorMsg::DeleteOtPin(otpin) => {
                    self.otpins.retain(|o| *o != otpin);
                }
                GraphicEditorMsg::SelectCopy(k, diff) => {
                    let pos = Pos(k * COMP_LEN, COMP_LINE);
                    self.component.push((k, pos, Ori::U));
                    self.state = State::MoveLC(self.component.len() - 1, diff);
                }
                GraphicEditorMsg::SelectMove(k, diff) => {
                    self.state = State::MoveLC(k, diff);
                }
                GraphicEditorMsg::Update(pos) => match &self.state {
                    State::None => {
                        return false;
                    }
                    State::MoveLC(k, diff) => {
                        self.component[*k].1 = pos - *diff;
                    }
                    State::SelectPin(pin) => {
                        return true;
                    }
                },
                GraphicEditorMsg::UnSelect => match &self.state {
                    State::None => {
                        return false;
                    }
                    State::MoveLC(_, _) => {
                        self.state = State::None;
                    }
                    State::SelectPin(_) => {
                        self.state = State::None;
                    }
                },
                GraphicEditorMsg::SelectPin(pin) => {
                    // remove pin from connected edges
                    match &pin {
                        Either::Left(Either::Left(inpin)) => {
                            self.inputs.retain(|(i, _)| i != inpin);
                        }
                        Either::Right(Either::Left(otpin)) => {
                            self.inputs.retain(|(o, _)| o != otpin);
                        }
                        Either::Left(Either::Right(name_inpin)) => {
                            self.inputs.retain(|(_, ni)| ni != name_inpin);
                            self.edges.retain(|(_, ni)| ni != name_inpin);
                        }
                        Either::Right(Either::Right(name_otpin)) => {
                            self.otputs.retain(|(_, no)| no != name_otpin);
                            self.edges.retain(|(no, _)| no != name_otpin);
                        }
                    }
                    match self.state.clone() {
                        State::None | State::MoveLC(_, _) => {
                            self.state = State::SelectPin(pin);
                            return true;
                        }
                        State::SelectPin(pin2) => {
                            let b = match &pin2 {
                                Either::Left(Either::Left(inpin2))
                                    if self.inputs.iter().any(|(i, _)| i == inpin2) =>
                                {
                                    true
                                }

                                Either::Right(Either::Left(otpin2))
                                    if self.otputs.iter().any(|(o, _)| o == otpin2) =>
                                {
                                    true
                                }
                                Either::Left(Either::Right(name_inpin2))
                                    if self.edges.iter().any(|(no, ni)| name_inpin2 == ni) =>
                                {
                                    true
                                }
                                Either::Right(Either::Right(name_otpin2))
                                    if self.edges.iter().any(|(no, ni)| no == name_otpin2) =>
                                {
                                    true
                                }
                                _ => false,
                            };
                            if b {
                                self.state = State::None;
                                return true;
                            }
                            match (pin, pin2) {
                                (
                                    Either::Left(Either::Left(inpin1)),
                                    Either::Left(Either::Right(k_inpin2)),
                                )
                                | (
                                    Either::Left(Either::Right(k_inpin2)),
                                    Either::Left(Either::Left(inpin1)),
                                ) => {
                                    self.inputs.push((inpin1, k_inpin2));
                                }
                                (
                                    Either::Right(Either::Left(otpin1)),
                                    Either::Right(Either::Right(k_otpin2)),
                                )
                                | (
                                    Either::Right(Either::Right(k_otpin2)),
                                    Either::Right(Either::Left(otpin1)),
                                ) => {
                                    self.otputs.push((otpin1, k_otpin2));
                                }
                                (
                                    Either::Left(Either::Right(k_inpin)),
                                    Either::Right(Either::Right(k_otpin)),
                                )
                                | (
                                    Either::Right(Either::Right(k_otpin)),
                                    Either::Left(Either::Right(k_inpin)),
                                ) => {
                                    self.edges.push((k_otpin, k_inpin));
                                }
                                _ => {}
                            }
                            self.state = State::None;
                        }
                    }
                }
                GraphicEditorMsg::Delte(k) => {
                    self.component.remove(k);
                }
                GraphicEditorMsg::GoToTest => {
                    let GraphicEditorProps {
                        logic_circuits_components,
                        on_goto_test,
                        on_log,
                        maybe_initial_locpos: _,
                    } = ctx.props();
                    let lcs: Vec<(Name, LoC)> = self
                        .component
                        .iter()
                        .enumerate()
                        .map(|(k, (num, _, _))| {
                            (
                                format!("{k}").into(),
                                logic_circuits_components[*num].clone(),
                            )
                        })
                        .collect::<Vec<_>>();
                    let edges: Vec<((Name, OtPin), (Name, InPin))> = self
                        .edges
                        .iter()
                        .map(|((ko, o), (ki, i))| {
                            let i: InPin = {
                                let inpins = lcs[*ki].1.get_inpins();
                                inpins[*i].0.clone()
                            };
                            let o: OtPin = {
                                let otpins = lcs[*ko].1.get_otpins();
                                otpins[*o].0.clone()
                            };
                            ((format!("{ko}").into(), o), (format!("{ki}").into(), i))
                        })
                        .collect::<Vec<_>>();
                    let input: Vec<(InPin, (Name, InPin))> = self
                        .inputs
                        .iter()
                        .cloned()
                        .map(|(i, (n, inpin))| {
                            let i: InPin = self.inpins[i].clone();
                            let inpin: InPin = {
                                let inpins = lcs[n].1.get_inpins();
                                inpins[inpin].0.clone()
                            };
                            (i, (format!("{n}").into(), inpin))
                        })
                        .collect::<Vec<_>>();
                    let output: Vec<(OtPin, (Name, OtPin))> = self
                        .otputs
                        .iter()
                        .cloned()
                        .map(|(o, (n, otpin))| {
                            let o: OtPin = self.otpins[o].clone();
                            let otpin: OtPin = {
                                let otpins = lcs[n].1.get_otpins();
                                otpins[otpin].0.clone()
                            };
                            (o, (format!("{n}").into(), otpin))
                        })
                        .collect::<Vec<_>>();
                    utils::view::log(format!("{lcs:?} {edges:?} {input:?} {output:?}"));
                    let loc = LoC::new_graph("new".into(), lcs, edges, input, output);
                    let lcs_pos_ori: Vec<(Name, (Pos, Ori))> = self
                        .component
                        .iter()
                        .map(|(n, p, o)| (format!("{n}").into(), (*p, *o)))
                        .collect();
                    match loc {
                        Ok(loc) => {
                            on_goto_test.emit((loc.take_fingraph().unwrap(), lcs_pos_ori));
                        }
                        Err(err) => on_log.emit(format!("{err:?}")),
                    }
                }
                _ => {
                    unimplemented!()
                }
            }
            true
        }
    }

    #[derive(Debug, Clone, PartialEq)]
    pub enum PlayGroundState {
        Test(FinGraph, Vec<(Name, (Pos, Ori))>),
        Edit(Option<AllPositions>),
    }

    #[derive(Debug, Clone, PartialEq)]

    pub struct PlayGround {
        state: PlayGroundState,
    }

    #[derive(Debug, Clone, PartialEq)]
    pub enum PlayGroundMsg {
        GotoTest((FinGraph, Vec<(Name, (Pos, Ori))>)),
        GotoEdit(),
    }

    #[derive(Debug, Clone, PartialEq, Properties)]
    pub struct PlayGroudProps {
        init_component: Vec<LoC>,
    }

    impl Component for PlayGround {
        type Message = PlayGroundMsg;
        type Properties = PlayGroudProps;
        fn create(ctx: &Context<Self>) -> Self {
            todo!()
        }
        fn view(&self, ctx: &Context<Self>) -> Html {
            let logic_circuits_components = ctx.props().init_component.clone();
            match self.state.clone() {
                PlayGroundState::Test(init_fingraph, init_pos_lc) => {
                    html! {
                        <FingraphMachine {init_fingraph} {init_pos_lc}/>
                    }
                }
                PlayGroundState::Edit(maybe_initial_locpos) => {
                    let on_log = Callback::noop();
                    let on_goto_test: Callback<(FinGraph, _)> = ctx
                        .link()
                        .callback(|loc: (FinGraph, Vec<_>)| PlayGroundMsg::GotoTest(loc));
                    html! {
                        <GraphicEditor {on_goto_test} {on_log} {logic_circuits_components} {maybe_initial_locpos}/>
                    }
                }
            }
        }
        fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
            match msg {
                PlayGroundMsg::GotoEdit() => {
                    let PlayGroundState::Test(fingraph, pos) = self.state.clone() else {
                        unreachable!("不整合")
                    };
                    todo!()
                }
                PlayGroundMsg::GotoTest((loc, poss)) => {
                    self.state = PlayGroundState::Test(loc, poss);
                }
            }
            true
        }
    }
}
