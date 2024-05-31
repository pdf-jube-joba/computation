use super::*;
use anyhow::{bail, Result};
use either::Either;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fmt::Display};
use utils::view::{log, svg::*};
use yew::prelude::*;
use yew::Properties;

mod colors {
    pub const BOOL_T_COL: &str = "salmon";
    pub const BOOL_F_COL: &str = "skyblue";
    pub const COMP_COL: &str = "lightgray";
    pub const BORDER: &str = "black";
}

const WIDTH_LC: usize = 50;
const PIN_LEN: usize = 25;
const PIN_RAD: usize = 7;

const COMP_LINE: usize = 400;
const COMP_LEN: usize = 100;

const PIN_TEXT_SIZE: usize = 8;
const LOC_TEXT_SIZE: usize = 15;

const INIT_INPIN_POS: Pos = Pos(50, 100);
const INIT_OTPIN_POS: Pos = Pos(800, 100);

const INPUTPIN_DIFF: Diff = Diff(30, 30);

#[derive(Debug, Clone, PartialEq, Properties)]
struct InPinProps {
    pos: Pos,
    state: Bool,
    inpin: InPin,
    #[prop_or(Callback::noop())]
    onclick: Callback<()>,
}

#[function_component(InPinView)]
fn inpin_view(props: &InPinProps) -> Html {
    let InPinProps {
        pos,
        state,
        inpin,
        onclick,
    } = props.clone();
    html! {
        <>
        <CircleView
            {pos}
            rad={PIN_RAD}
            col={if state == Bool::T {colors::BOOL_T_COL.to_string()} else {colors::BOOL_F_COL.to_string()}}
            border="black"
            onclick={Callback::from(move |_| onclick.emit(()))}
        />
        <TextView pos={pos + Diff(PIN_RAD as isize , 0)} text={inpin.to_string()} size={PIN_TEXT_SIZE} transparent={true}/>
        </>
    }
}

#[derive(Debug, Clone, PartialEq, Properties)]
struct OtPinProps {
    pos: Pos,
    state: Bool,
    otpin: OtPin,
    #[prop_or(Callback::noop())]
    onclick: Callback<()>,
}

#[function_component(OtPinView)]
fn inpin_view(props: &OtPinProps) -> Html {
    let OtPinProps {
        pos,
        state,
        otpin,
        onclick,
    } = props.clone();
    html! {
        <>
        <CircleView
            {pos}
            rad={PIN_RAD}
            col={if state == Bool::T {colors::BOOL_T_COL.to_string()} else {colors::BOOL_F_COL.to_string()}}
            border="black"
            onclick={Callback::from(move |_| onclick.emit(()))}
        />
        <TextView pos={pos + Diff(- ((otpin.len() * PIN_TEXT_SIZE) as isize), 0)} text={otpin.to_string()} size={PIN_TEXT_SIZE} transparent={true}/>
        </>
    }
}

#[derive(Debug, Clone, PartialEq, Properties)]
struct InPinForInputProps {
    pos: Pos,
    state: Bool,
    inpin: InPin,
    #[prop_or(Callback::noop())]
    onclickinpin: Callback<()>,
    #[prop_or(Callback::noop())]
    onmove: Callback<MoveMsg>,
}

#[function_component(InPinForInPutView)]
fn inpin_for_input_view(props: &InPinForInputProps) -> Html {
    let InPinForInputProps {
        pos,
        state,
        inpin,
        onclickinpin,
        onmove,
    } = props.clone();
    let MouseEventCallbacks {
        onmousedown,
        onmousemove,
        onmouseup,
        onmouseleave,
    } = make_callback(pos, onmove);
    html! {
        <>
            <RectView pos={pos - INPUTPIN_DIFF / 2} diff={INPUTPIN_DIFF} col={colors::COMP_COL} border={colors::BORDER} {onmousedown} {onmouseup} {onmousemove} {onmouseleave}/>
            <InPinView {pos} {state} {inpin} onclick={onclickinpin}/>
        </>
    }
}

#[derive(Debug, Clone, PartialEq, Properties)]
struct OtPinForOtputProps {
    pos: Pos,
    state: Bool,
    otpin: OtPin,
    #[prop_or(Callback::noop())]
    onclickotpin: Callback<()>,
    #[prop_or(Callback::noop())]
    onmove: Callback<MoveMsg>,
}

#[function_component(OtPinForOtPutView)]
fn inpin_for_input_view(props: &OtPinForOtputProps) -> Html {
    let OtPinForOtputProps {
        pos,
        state,
        otpin,
        onclickotpin,
        onmove,
    } = props.clone();
    let MouseEventCallbacks {
        onmousedown,
        onmousemove,
        onmouseup,
        onmouseleave,
    } = make_callback(pos, onmove);
    html! {
        <>
            <RectView pos={pos - INPUTPIN_DIFF / 2} diff={INPUTPIN_DIFF} col={colors::COMP_COL} border={colors::BORDER} {onmousedown} {onmouseup} {onmousemove} {onmouseleave}/>
            <OtPinView {pos} {state} {otpin} onclick={onclickotpin}/>
        </>
    }
}

#[derive(Debug, Clone, PartialEq, Properties)]
struct LoCProps {
    name: Name,
    inputs: Vec<(InPin, Bool)>,
    otputs: Vec<(OtPin, Bool)>,
    ori: Ori,
    pos: Pos, // center of loc
    #[prop_or(Callback::noop())]
    onmovelc: Callback<MoveMsg>,
    #[prop_or(Callback::noop())]
    onclickinpin: Callback<usize>,
    #[prop_or(Callback::noop())]
    onclickotpin: Callback<usize>,
    #[prop_or(Callback::noop())]
    onrightclick: Callback<()>,
    #[prop_or(Callback::noop())]
    onrotclockwise: Callback<()>,
    #[prop_or(Callback::noop())]
    onrotcounterclockwise: Callback<()>,
}

fn rot(mut diff: Diff, ori: Ori) -> Diff {
    match ori {
        Ori::U => diff,
        Ori::D => -diff,
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

impl LoCProps {
    fn rect_diff(&self) -> Diff {
        let m = std::cmp::max(self.inputs.len(), self.otputs.len());
        Diff(WIDTH_LC as isize, (PIN_LEN * m) as isize)
    }

    fn input_rowdiff_num(&self, inpinnum: &usize) -> Option<Diff> {
        if self.inputs.len() <= *inpinnum {
            return None;
        }
        let diff = Diff(0, PIN_LEN as isize) * *inpinnum;
        Some(-self.rect_diff() / 2 + diff)
    }

    fn input_pos_num(&self, inpinnum: &usize) -> Option<Pos> {
        Some(self.pos + rot(self.input_rowdiff_num(inpinnum)?, self.ori))
    }

    fn input_pos(&self, inpin: &InPin) -> Option<Pos> {
        let inpinnum = self.inputs.iter().position(|i| &i.0 == inpin)?;
        Some(self.pos + rot(self.input_rowdiff_num(&inpinnum)?, self.ori))
    }

    fn otput_rowdiff_num(&self, otpinnum: &usize) -> Option<Diff> {
        if self.otputs.len() <= *otpinnum {
            return None;
        }
        let diff = Diff(0, PIN_LEN as isize) * *otpinnum;
        let mut diff_rect = self.rect_diff();
        diff_rect.refl_x();
        Some(-diff_rect / 2 + diff)
    }

    fn otput_pos_num(&self, otpinnum: &usize) -> Option<Pos> {
        Some(self.pos + rot(self.otput_rowdiff_num(otpinnum)?, self.ori))
    }

    fn otput_pos(&self, otpin: &OtPin) -> Option<Pos> {
        let otpinnum = self.otputs.iter().position(|o| &o.0 == otpin)?;
        Some(self.pos + rot(self.otput_rowdiff_num(&otpinnum)?, self.ori))
    }
}

#[function_component(LoCView)]
fn lc_view(locprops: &LoCProps) -> Html {
    let LoCProps {
        name,
        inputs,
        otputs,
        ori,
        pos,
        onmovelc,
        onclickinpin,
        onclickotpin,
        onrightclick,
        onrotclockwise,
        onrotcounterclockwise,
    } = locprops.clone();
    let MouseEventCallbacks {
        onmousedown,
        onmousemove,
        onmouseup,
        onmouseleave,
    } = make_callback(pos, onmovelc);
    let onrightclick = Callback::from(move |e: MouseEvent| {
        e.prevent_default();
        onrightclick.emit(());
    });
    let onrotclockwise = Callback::from(move |e: MouseEvent| {
        e.prevent_default();
        onrotclockwise.emit(());
    });
    let onrotcounterclockwise = Callback::from(move |e: MouseEvent| {
        e.prevent_default();
        onrotcounterclockwise.emit(());
    });
    let name: String = format!("{name}");
    let diff = locprops.rect_diff();
    let text_pos: Pos = pos - Diff(LOC_TEXT_SIZE as isize, 0) * (name.len() / 2);
    let rot_clock = {
        let mut diff = diff;
        diff.refl_x();
        pos + diff / 2
    };
    let rot_count = { pos + diff / 2 };
    let rotate: String = format!(
        "rotate({}, {}, {})",
        match ori {
            Ori::U => 0,
            Ori::D => 180,
            Ori::R => 270,
            Ori::L => 90,
        },
        pos.0,
        pos.1
    );
    html! {
        <>
            <g transform={rotate}>
            <RectView pos={pos - diff / 2} {diff} col={"lightgray".to_string()} border={"black".to_string()} {onmousedown} {onmousemove} {onmouseleave} {onmouseup} oncontextmenu={onrightclick}/>
                <TextView pos={text_pos} text={name} size={LOC_TEXT_SIZE} transparent={true}/>
            {for inputs.into_iter().enumerate().map(|(k, (inpin, state))|{
                let onclick = onclickinpin.clone();
                let onclick = Callback::from(move |_|{
                    onclick.emit(k);
                });
                html!{
                    <InPinView pos={pos + locprops.input_rowdiff_num(&k).unwrap()} {state} inpin={inpin.clone()} {onclick}/>
                }
            })}
            {for otputs.into_iter().enumerate().map(|(k, (otpin, state))|{
                let onclick = onclickotpin.clone();
                let onclick = Callback::from(move |_|{
                    onclick.emit(k);
                });
                html!{
                    <OtPinView pos={pos + locprops.otput_rowdiff_num(&k).unwrap()} {state} otpin={otpin.clone()} {onclick}/>
                }
            })}
            <CircleView pos={rot_count} rad={PIN_RAD} col="white" border="black" onclick={onrotcounterclockwise} />
            <CircleView pos={rot_clock} rad={PIN_RAD} col="white" border="black" onclick={onrotclockwise} />
            </g>
        </>
    }
}

#[derive(Debug, Clone, PartialEq, Properties)]
pub struct ActLoCProps {
    fingraph: FinGraph,
    inpins: Vec<Pos>,
    otpins: Vec<Pos>,
    poslc: Vec<(Name, (Pos, Ori))>,
    on_inpin_clicks: Callback<InPin>,
}

impl ActLoCProps {
    fn new(
        fingraph: FinGraph,
        inpins: Vec<Pos>,
        otpins: Vec<Pos>,
        poslc: Vec<(Name, (Pos, Ori))>,
        on_inpin_clicks: Callback<InPin>,
    ) -> Result<Self> {
        if inpins.len() != fingraph.get_inpins().len() {
            bail!("length of inpin ivalid")
        }
        if otpins.len() != fingraph.get_otpins().len() {
            bail!("length of otpin ivalid")
        }
        Ok(Self {
            fingraph,
            inpins,
            otpins,
            poslc,
            on_inpin_clicks,
        })
    }
    fn get_lc_props(&self, name: &Name) -> Option<LoCProps> {
        let loc = self.fingraph.get_lc(name)?;
        let (pos, ori) = self
            .poslc
            .iter()
            .find_map(|(n, pos_ori)| if name == n { Some(pos_ori) } else { None })
            .unwrap();
        Some(LoCProps {
            name: loc.get_name(),
            inputs: loc.get_inpins(),
            otputs: loc.get_otpins(),
            ori: *ori,
            pos: *pos,
            onmovelc: Callback::noop(),
            onclickinpin: Callback::noop(),
            onclickotpin: Callback::noop(),
            onrightclick: Callback::noop(),
            onrotclockwise: Callback::noop(),
            onrotcounterclockwise: Callback::noop(),
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

    let lc_inpins = actlocprops.fingraph.get_inpins();
    let lc_otpins = actlocprops.fingraph.get_otpins();

    let inpin_edge: Vec<(InPin, Pos, Pos, Bool)> = {
        (0..lc_inpins.len())
            .map(|k| {
                let (inpin, state) = lc_inpins[k].clone();
                let (name, inpin_to) = actlocprops.fingraph.get_inpin_to_lc_inpin(&inpin).unwrap();
                let loc_prop = actlocprops.get_lc_props(&name).unwrap();
                let inpin_to_pos = loc_prop.input_pos(&inpin_to).unwrap();
                (inpin, actlocprops.inpins[k].clone(), inpin_to_pos, state)
            })
            .collect()
    };

    let otpin_edge: Vec<(OtPin, Pos, Pos, Bool)> = {
        (0..lc_otpins.len())
            .map(|k| {
                let (otpin, state) = lc_otpins[k].clone();
                let (name, otpin_to) = actlocprops.fingraph.get_otpin_to_lc_otpin(&otpin).unwrap();
                let loc_prop = actlocprops.get_lc_props(&name).unwrap();
                let otpin_to_pos = loc_prop.otput_pos(&otpin_to).unwrap();
                (otpin, actlocprops.otpins[k].clone(), otpin_to_pos, state)
            })
            .collect()
    };

    let inpins = lc_inpins.clone();

    let inpin_callback: Vec<Callback<()>> = {
        let mut v = vec![];
        for k in 0..inpins.len() {
            let callback = actlocprops.on_inpin_clicks.clone();
            let inpin = inpins[k].0.clone();
            let callback = Callback::from(move |()| {
                callback.emit(inpin.clone());
            });
            v.push(callback);
        }
        v
    };

    html! {
        <div height="500" width="900" border="solid #000" overflow="scroll">
        <svg width="1500" height="500" viewBox="0 0 1500 500">
        {for actlocprops.poslc.iter().map(|(name, _)|{
            let LoCProps { name, inputs, otputs, ori, pos, onmovelc, onclickinpin, onclickotpin, onrightclick, onrotclockwise, onrotcounterclockwise } = actlocprops.get_lc_props(name).unwrap();
            html!{
                <LoCView {name} {inputs} {otputs} {ori} {pos}/>
            }
        })}
        {for inpin_edge.into_iter().zip(inpin_callback).map(|((inpin, pos_i, pos_ni, state), onclick)|{
            html!{
                <>
                <InPinView pos={pos_i} {inpin} {state} {onclick}/>
                <PolyLineView vec={vec![pos_i, pos_ni]} col={if state == Bool::T {colors::BOOL_T_COL} else {colors::BOOL_F_COL}}/>
                </>
            }
        })
        }
        {for otpin_edge.into_iter().map(|(otpin, pos_o, pos_no, state)|{
            html!{
                <>
                <OtPinView pos={pos_o} {otpin} {state} onclick={Callback::noop()}/>
                <PolyLineView vec={vec![pos_no, pos_o]} col={if state == Bool::T {colors::BOOL_T_COL} else {colors::BOOL_F_COL}}/>
                </>
            }
        })}
        {for graph_edge.into_iter().map(|(pos_o, pos_i, state)|{

            html!{
                <PolyLineView vec={vec![pos_o, pos_i]} col={if state == Bool::T {colors::BOOL_T_COL} else {colors::BOOL_F_COL}}/>
            }
        })}
        </svg>
        </div>
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
    init_pos_inpins: Vec<Pos>,
    init_pos_otpins: Vec<Pos>,
}

impl FingraphMachineProps {
    pub fn new(
        init_fingraph: FinGraph,
        init_pos_lc: Vec<(Name, (Pos, Ori))>,
        init_pos_inpins: Vec<Pos>,
        init_pos_otpins: Vec<Pos>,
    ) -> Result<Self> {
        let names = init_fingraph.get_lc_names();
        for name in names {
            if init_pos_lc.iter().all(|(n, _)| n != &name) {
                bail!("not found {name} in init_pos_lc");
            }
        }
        if init_fingraph.get_inpins().len() != init_pos_inpins.len() {
            bail!("length of inpin invalid");
        }

        if init_fingraph.get_otpins().len() != init_pos_otpins.len() {
            bail!("length of otpin invalid");
        }
        Ok(Self {
            init_fingraph,
            init_pos_lc,
            init_pos_inpins,
            init_pos_otpins,
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
            init_pos_inpins,
            init_pos_otpins,
        } = ctx.props();
        Self {
            fingraph: init_fingraph.clone(),
        }
    }
    fn view(&self, ctx: &Context<Self>) -> Html {
        let on_step = ctx.link().callback(FingraphMachineMsg::Step);
        let on_inpin_clicks = ctx.link().callback(FingraphMachineMsg::ToggleIn);
        let FingraphMachineProps {
            init_fingraph: _,
            init_pos_lc,
            init_pos_inpins,
            init_pos_otpins,
        } = ctx.props().clone();
        html! {
            <>
            <ActLoCView fingraph={self.fingraph.clone()} poslc={init_pos_lc} {on_inpin_clicks} inpins={init_pos_inpins} otpins={init_pos_otpins}/>
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
type LoCNum = usize;

type PinVariant = Either<Either<InPinNum, (usize, InPinNum)>, Either<OtPinNum, (usize, OtPinNum)>>;

#[derive(Debug, Clone, PartialEq, Eq)]
enum State {
    None,
    MoveLC(Diff),
    MoveInPin(Diff),
    MoveOtPin(Diff),
    CopyLC(usize, Pos, Diff),
    SelectPin(PinVariant),
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct AllPositions {
    inpins: Vec<(InPin, Pos)>,
    otpins: Vec<(OtPin, Pos)>,
    component: Vec<(usize, Pos, Ori)>,
    edges: Vec<((LoCNum, InPinNum), (LoCNum, OtPinNum))>,
    inputs_edge: Vec<(InPinNum, (LoCNum, InPinNum))>,
    otputs_edge: Vec<(OtPinNum, (LoCNum, OtPinNum))>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GraphicEditor {
    allpositions: AllPositions,
    state: State,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GraphicEditorMsg {
    AddInpin(InPin),
    DeleteInPin(InPin),
    AddOtPin(OtPin),
    DeleteOtPin(OtPin),

    MoveLoC(usize, MoveMsg),
    DeleteLoC(usize), // right click

    MoveInput(usize, MoveMsg),
    MoveOtput(usize, MoveMsg),

    SelectCopy(usize, Pos), // copy from tools
    MoveCopy(usize, MoveMsg),

    SelectPin(PinVariant), // click to connect pins

    RotClock(usize),
    RotCount(usize),

    GoToTest,
    Load(serde_json::Value),

    None,
}

#[derive(Debug, Clone, PartialEq, Properties)]
pub struct GraphicEditorProps {
    pub logic_circuits_components: Vec<LoC>,
    pub on_goto_test: Callback<(FinGraph, Vec<(Name, (Pos, Ori))>, Vec<Pos>, Vec<Pos>)>,
    pub on_log: Callback<String>,
    pub maybe_initial_locpos: Option<AllPositions>,
}

impl Component for GraphicEditor {
    type Message = GraphicEditorMsg;
    type Properties = GraphicEditorProps;
    fn create(ctx: &Context<Self>) -> Self {
        let v = ctx.props().maybe_initial_locpos.clone();
        let allpositions = v.unwrap_or_default();
        Self {
            allpositions,
            state: State::None,
        }
    }
    fn view(&self, ctx: &Context<Self>) -> Html {
        let GraphicEditor {
            allpositions,
            state,
        } = self.clone();
        let GraphicEditorProps {
            logic_circuits_components,
            on_goto_test,
            on_log,
            maybe_initial_locpos: _,
        } = ctx.props();

        let temp_json = serde_json::to_value(allpositions.clone()).unwrap();

        let loc_vec = (0..allpositions.component.len())
            .map(|k| {
                let (num, pos, ori) = allpositions.component[k];
                let loc = &logic_circuits_components[num];
                let inputs = loc.get_inpins();
                let otputs = loc.get_otpins();
                let onmovelc = ctx
                    .link()
                    .callback(move |msg: MoveMsg| GraphicEditorMsg::MoveLoC(k, msg));
                let onclickinpin = ctx.link().callback(move |num_i: usize| {
                    GraphicEditorMsg::SelectPin(Either::Left(Either::Right((k, num_i))))
                });
                let onclickotpin = ctx.link().callback(move |num_o: usize| {
                    GraphicEditorMsg::SelectPin(Either::Right(Either::Right((k, num_o))))
                });
                let onrightclick = ctx.link().callback(move |_| GraphicEditorMsg::DeleteLoC(k));
                let onrotclockwise = ctx.link().callback(move |_| GraphicEditorMsg::RotClock(k));
                let onrotcounterclockwise =
                    ctx.link().callback(move |_| GraphicEditorMsg::RotCount(k));
                LoCProps {
                    name: loc.get_name(),
                    inputs,
                    otputs,
                    pos,
                    ori,
                    onmovelc,
                    onclickinpin,
                    onclickotpin,
                    onrightclick,
                    onrotclockwise,
                    onrotcounterclockwise,
                }
            })
            .collect::<Vec<_>>();

        let inpins_edge = (0..allpositions.inputs_edge.len())
            .map(|k| {
                let (i, (k, i1)) = allpositions.inputs_edge[k];
                let pos_i = allpositions.inpins[i].clone();
                let pos_i2 = loc_vec[k].input_pos_num(&i1).unwrap();
                (pos_i, pos_i2)
            })
            .collect::<Vec<_>>();

        let otpins_edge = (0..allpositions.otputs_edge.len())
            .map(|k| {
                let (o, (k, o1)) = allpositions.otputs_edge[k];
                let pos_o = allpositions.otpins[o].clone();
                let pos_o2 = loc_vec[k].otput_pos_num(&o1).unwrap();
                (pos_o, pos_o2)
            })
            .collect::<Vec<_>>();

        let graph_edges = allpositions
            .edges
            .iter()
            .map(|((ko, o), (ki, i))| {
                let pos_o = loc_vec[*ko].otput_pos_num(o).unwrap();
                let pos_i = loc_vec[*ki].input_pos_num(i).unwrap();
                (pos_o, pos_i)
            })
            .collect::<Vec<_>>();

        let width = (logic_circuits_components.len()) * COMP_LEN;

        html! {
            <div height="500" width="900" border="solid #000" overflow="scroll">
            <svg width={width.to_string()} height="500" viewBox={format!("0 0 {width} 500")}>
                {
                    for logic_circuits_components.clone().into_iter().enumerate().map(|(k, loc)|{
                    let inputs = loc.get_inpins();
                    let otputs = loc.get_otpins();
                    let pos: Pos =
                        match &self.state {
                            State::CopyLC(k1, pos, diff) if k == *k1 => *pos - *diff,
                            _ => Pos((k + 1) * COMP_LEN, COMP_LINE),
                        };
                    let onmovelc = ctx.link().callback(move |msg: MoveMsg|{
                        GraphicEditorMsg::MoveCopy(k, msg)
                    });
                    html!{
                        <LoCView name={loc.get_name()} {inputs} {otputs} {pos} ori={Ori::U} {onmovelc} />
                    }
                })}
                {for loc_vec.into_iter().map(|locprop|{
                    let LoCProps { name, inputs, otputs, ori, pos, onmovelc, onclickinpin, onclickotpin, onrightclick, onrotclockwise, onrotcounterclockwise } = locprop;
                    html!{
                        <LoCView {name} {inputs} {otputs} pos={pos} ori={ori} {onmovelc} {onclickinpin} {onclickotpin} {onrightclick} {onrotclockwise} {onrotcounterclockwise}/>
                    }
                })}
                {for allpositions.inpins.into_iter().enumerate().map(|(k, (inpin, pos))|{
                    let onclickinpin = ctx.link().callback(move |_|{
                        GraphicEditorMsg::SelectPin(Either::Left(Either::Left(k)))
                    });
                    let onmove = ctx.link().callback(move |msg: MoveMsg|{
                        GraphicEditorMsg::MoveInput(k, msg)
                    });
                    html!{
                        <InPinForInPutView {pos} state={Bool::F} inpin={inpin.clone()} {onmove} {onclickinpin}/>
                    }
                })}
                {for allpositions.otpins.into_iter().enumerate().map(|(k, (otpin, pos))|{
                    let onclickotpin = ctx.link().callback(move |_|{
                        GraphicEditorMsg::SelectPin(Either::Right(Either::Left(k)))
                    });
                    let onmove = ctx.link().callback(move |msg: MoveMsg|{
                        GraphicEditorMsg::MoveOtput(k, msg)
                    });
                    html!{
                        <OtPinForOtPutView otpin={otpin.clone()} {pos} state={Bool::F} {onclickotpin} {onmove}/>
                    }
                })}
                {for inpins_edge.into_iter().map(|(pos_i, pos_i2)|{
                    html!{
                        <PolyLineView vec={vec![pos_i.1, pos_i2]} col={colors::BOOL_F_COL}/>
                    }
                })}
                {for otpins_edge.into_iter().map(|(pos_o, pos_o2)|{
                    html!{
                        <PolyLineView vec={vec![pos_o.1, pos_o2]} col={colors::BOOL_F_COL}/>
                    }
                })}
                {for graph_edges.into_iter().map(|(pos_o, pos_i)|{
                    html!{
                        <PolyLineView vec={vec![pos_o, pos_i]} col={colors::BOOL_F_COL}/>
                    }
                })}
            </svg> <br/>
            <utils::view::InputText
                description={"add inpins".to_string()}
                on_push_load_button={ctx
                    .link()
                    .callback(|s: String| GraphicEditorMsg::AddInpin(s.into()))
                }
            />
            <utils::view::InputText
                description={"add otpins".to_string()}
                on_push_load_button={ctx
                    .link()
                    .callback(|s: String| GraphicEditorMsg::AddOtPin(s.into()))}
            /> <br/>
            <utils::view::InputText
                description={"remove inpins".to_string()}
                on_push_load_button={ctx
                    .link()
                    .callback(|s: String| GraphicEditorMsg::DeleteInPin(s.into()))}
            />
            <utils::view::InputText
                description={"remove otpins".to_string()}
                on_push_load_button={ctx
                    .link()
                    .callback(|s: String| GraphicEditorMsg::DeleteOtPin(s.into()))}
            />
            <utils::view::ButtonView
                on_click={ctx.link().callback(|_| GraphicEditorMsg::GoToTest)}
                text={"test"}
            />
            <utils::view::JsonFileSaveView json_value={temp_json}/>
            <utils::view::JsonFileReadView
                on_drop_json={ctx.link().callback(|json: serde_json::Value| GraphicEditorMsg::Load(json))}
            />
            </div>
        }
    }
    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        log(format!("{msg:?} {:?}", self.state));
        match (msg, self.state.clone()) {
            // pin add or remove
            (GraphicEditorMsg::AddInpin(inpin), State::None) => {
                if self.allpositions.inpins.iter().any(|(i, _)| *i == inpin) {
                    return false;
                }
                self.allpositions.inpins.push((inpin, INIT_INPIN_POS));
            }
            (GraphicEditorMsg::AddOtPin(otpin), State::None) => {
                if self.allpositions.otpins.iter().any(|(o, _)| *o == otpin) {
                    return false;
                }
                self.allpositions.otpins.push((otpin, INIT_OTPIN_POS));
            }
            (GraphicEditorMsg::DeleteInPin(inpin), State::None) => {
                self.allpositions.inpins.retain(|i| i.0 != inpin);
            }
            (GraphicEditorMsg::DeleteOtPin(otpin), State::None) => {
                self.allpositions.otpins.retain(|o| o.0 != otpin);
            }
            // do some on loc
            (GraphicEditorMsg::MoveLoC(k, MoveMsg::Select(diff)), State::None) => {
                self.state = State::MoveLC(diff);
            }
            (GraphicEditorMsg::MoveLoC(k, MoveMsg::Move(pos)), State::MoveLC(diff)) => {
                self.allpositions.component[k].1 = pos - diff;
            }
            (GraphicEditorMsg::MoveLoC(_, MoveMsg::UnSelect), State::MoveLC(_)) => {
                self.state = State::None;
            }
            (GraphicEditorMsg::MoveLoC(_, MoveMsg::Move(_) | MoveMsg::UnSelect), State::None) => {
                return false;
            }
            (GraphicEditorMsg::RotClock(k), State::None) => {
                self.allpositions.component[k].2.rot_clockwise();
            }
            (GraphicEditorMsg::RotCount(k), State::None) => {
                self.allpositions.component[k].2.rot_counterclockwise();
            }
            (GraphicEditorMsg::DeleteLoC(k), State::None) => {
                self.allpositions.component.remove(k);
            }
            // do some on tools
            (GraphicEditorMsg::MoveCopy(k, MoveMsg::Select(diff)), State::None) => {
                self.state = State::CopyLC(k, Pos((k + 1) * COMP_LEN, COMP_LINE), diff);
            }
            (GraphicEditorMsg::MoveCopy(k, MoveMsg::Move(pos)), State::CopyLC(k1, _, diff))
                if k == k1 =>
            {
                self.state = State::CopyLC(k, pos, diff);
            }
            (GraphicEditorMsg::MoveCopy(k, MoveMsg::UnSelect), State::CopyLC(k1, pos, diff))
                if k == k1 =>
            {
                self.allpositions.component.push((k, pos, Ori::U));
                self.state = State::None;
            }
            (GraphicEditorMsg::MoveCopy(_, MoveMsg::Move(_) | MoveMsg::UnSelect), State::None) => {
                return false;
            }
            // do some on inpin
            (GraphicEditorMsg::MoveInput(k, MoveMsg::Select(diff)), State::None) => {
                self.state = State::MoveInPin(diff);
            }
            (GraphicEditorMsg::MoveInput(k, MoveMsg::Move(pos)), State::MoveInPin(diff)) => {
                self.allpositions.inpins[k].1 = pos - diff;
            }
            (GraphicEditorMsg::MoveInput(k, MoveMsg::UnSelect), State::MoveInPin(_)) => {
                self.state = State::None;
            }
            (GraphicEditorMsg::MoveInput(_, MoveMsg::Move(_) | MoveMsg::UnSelect), State::None) => {
                return false;
            }
            // do some on otpin
            (GraphicEditorMsg::MoveOtput(k, MoveMsg::Select(diff)), State::None) => {
                self.state = State::MoveOtPin(diff);
            }
            (GraphicEditorMsg::MoveOtput(k, MoveMsg::Move(pos)), State::MoveOtPin(diff)) => {
                self.allpositions.otpins[k].1 = pos - diff;
            }
            (GraphicEditorMsg::MoveOtput(k, MoveMsg::UnSelect), State::MoveOtPin(_)) => {
                self.state = State::None;
            }
            (GraphicEditorMsg::MoveOtput(k, MoveMsg::Move(_) | MoveMsg::UnSelect), State::None) => {
                return false;
            }
            (GraphicEditorMsg::SelectPin(pin), State::None) => {
                // remove pin from edges
                match &pin {
                    Either::Left(Either::Left(inpin)) => {
                        self.allpositions.inputs_edge.retain(|(i, _)| i != inpin);
                    }
                    Either::Right(Either::Left(otpin)) => {
                        self.allpositions.otputs_edge.retain(|(o, _)| o != otpin);
                    }
                    Either::Left(Either::Right(name_inpin)) => {
                        self.allpositions
                            .inputs_edge
                            .retain(|(_, ni)| ni != name_inpin);
                        self.allpositions.edges.retain(|(_, ni)| ni != name_inpin);
                    }
                    Either::Right(Either::Right(name_otpin)) => {
                        self.allpositions
                            .otputs_edge
                            .retain(|(_, no)| no != name_otpin);
                        self.allpositions.edges.retain(|(no, _)| no != name_otpin);
                    }
                }
                self.state = State::SelectPin(pin);
            }
            (GraphicEditorMsg::SelectPin(pin), State::SelectPin(pin2)) => {
                // assert pin is not connected by edge
                // remove pin2 from edge
                match &pin2 {
                    Either::Left(Either::Left(inpin)) => {
                        self.allpositions.inputs_edge.retain(|(i, _)| i != inpin);
                    }
                    Either::Right(Either::Left(otpin)) => {
                        self.allpositions.otputs_edge.retain(|(o, _)| o != otpin);
                    }
                    Either::Left(Either::Right(name_inpin)) => {
                        self.allpositions
                            .inputs_edge
                            .retain(|(_, ni)| ni != name_inpin);
                        self.allpositions.edges.retain(|(_, ni)| ni != name_inpin);
                    }
                    Either::Right(Either::Right(name_otpin)) => {
                        self.allpositions
                            .otputs_edge
                            .retain(|(_, no)| no != name_otpin);
                        self.allpositions.edges.retain(|(no, _)| no != name_otpin);
                    }
                }
                match (pin, pin2) {
                    (Either::Left(Either::Left(inpin1)), Either::Left(Either::Right(k_inpin2)))
                    | (Either::Left(Either::Right(k_inpin2)), Either::Left(Either::Left(inpin1))) =>
                    {
                        self.allpositions.inputs_edge.push((inpin1, k_inpin2));
                    }
                    (
                        Either::Right(Either::Left(otpin1)),
                        Either::Right(Either::Right(k_otpin2)),
                    )
                    | (
                        Either::Right(Either::Right(k_otpin2)),
                        Either::Right(Either::Left(otpin1)),
                    ) => {
                        self.allpositions.otputs_edge.push((otpin1, k_otpin2));
                    }
                    (
                        Either::Left(Either::Right(k_inpin)),
                        Either::Right(Either::Right(k_otpin)),
                    )
                    | (
                        Either::Right(Either::Right(k_otpin)),
                        Either::Left(Either::Right(k_inpin)),
                    ) => {
                        self.allpositions.edges.push((k_otpin, k_inpin));
                    }
                    _ => {}
                }
                self.state = State::None;
            }
            (_, State::SelectPin(_)) => {
                return false;
            }
            (GraphicEditorMsg::GoToTest, State::None) => {
                let GraphicEditorProps {
                    logic_circuits_components,
                    on_goto_test,
                    on_log,
                    maybe_initial_locpos: _,
                } = ctx.props();
                let lcs: Vec<(Name, LoC)> = self
                    .allpositions
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
                    .allpositions
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
                    .allpositions
                    .inputs_edge
                    .iter()
                    .cloned()
                    .map(|(i, (n, inpin))| {
                        let i: InPin = self.allpositions.inpins[i].0.clone();
                        let inpin: InPin = {
                            let inpins = lcs[n].1.get_inpins();
                            inpins[inpin].0.clone()
                        };
                        (i, (format!("{n}").into(), inpin))
                    })
                    .collect::<Vec<_>>();
                let otput: Vec<(OtPin, (Name, OtPin))> = self
                    .allpositions
                    .otputs_edge
                    .iter()
                    .cloned()
                    .map(|(o, (n, otpin))| {
                        let o: OtPin = self.allpositions.otpins[o].0.clone();
                        let otpin: OtPin = {
                            let otpins = lcs[n].1.get_otpins();
                            otpins[otpin].0.clone()
                        };
                        (o, (format!("{n}").into(), otpin))
                    })
                    .collect::<Vec<_>>();
                utils::view::log(format!("{lcs:?} {edges:?} {input:?} {otput:?}"));
                let loc = LoC::new_graph("new".into(), lcs, edges, input, otput);
                let lcs_pos_ori: Vec<(Name, (Pos, Ori))> = self
                    .allpositions
                    .component
                    .iter()
                    .map(|(n, p, o)| (format!("{n}").into(), (*p, *o)))
                    .collect();
                let inpins_pos: Vec<Pos> = self
                    .allpositions
                    .inpins
                    .iter()
                    .map(|(_, pos)| *pos)
                    .collect();
                let otpins_pos: Vec<Pos> = self
                    .allpositions
                    .otpins
                    .iter()
                    .map(|(_, pos)| *pos)
                    .collect();
                match loc {
                    Ok(loc) => {
                        on_goto_test.emit((
                            loc.take_fingraph().unwrap(),
                            lcs_pos_ori,
                            inpins_pos,
                            otpins_pos,
                        ));
                    }
                    Err(err) => on_log.emit(format!("{err:?}")),
                }
            }
            (GraphicEditorMsg::Load(json), State::None) => {
                let Ok(AllPositions {
                    inpins,
                    otpins,
                    component,
                    edges,
                    inputs_edge,
                    otputs_edge,
                }): Result<AllPositions, _> = serde_json::from_value(json)
                else {
                    return false;
                };
                self.allpositions.inpins = inpins;
                self.allpositions.otpins = otpins;
                self.allpositions.component = component;
                self.allpositions.edges = edges;
                self.allpositions.inputs_edge = inputs_edge;
                self.allpositions.otputs_edge = otputs_edge;
            }
            (GraphicEditorMsg::None, _) => {
                return false;
            }
            (msg, state) => {
                unreachable!("不整合 {:?}, {:?}", msg, state)
            }
        }
        true
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum PlayGroundState {
    Test(FinGraph, Vec<(Name, (Pos, Ori))>, Vec<Pos>, Vec<Pos>),
    Edit(Option<AllPositions>),
}

#[derive(Debug, Clone, PartialEq)]

pub struct PlayGround {
    state: PlayGroundState,
}

#[derive(Debug, Clone, PartialEq)]
pub enum PlayGroundMsg {
    GotoTest((FinGraph, Vec<(Name, (Pos, Ori))>, Vec<Pos>, Vec<Pos>)),
    GotoEdit(),
}

#[derive(Debug, Clone, PartialEq, Properties)]
pub struct PlayGroudProps {
    pub init_component: Vec<LoC>,
}

impl Component for PlayGround {
    type Message = PlayGroundMsg;
    type Properties = PlayGroudProps;
    fn create(ctx: &Context<Self>) -> Self {
        Self {
            state: PlayGroundState::Edit(None),
        }
    }
    fn view(&self, ctx: &Context<Self>) -> Html {
        let logic_circuits_components = ctx.props().init_component.clone();
        match self.state.clone() {
            PlayGroundState::Test(init_fingraph, init_pos_lc, init_pos_inpins, init_pos_otpins) => {
                let json_loc = serde_json::to_value(&init_fingraph).unwrap();
                html! {
                    <>
                    <FingraphMachine {init_fingraph} {init_pos_lc} {init_pos_inpins} {init_pos_otpins}/>
                    <utils::view::JsonFileSaveView json_value={json_loc} />
                    <utils::view::ButtonView on_click={ctx.link().callback(|_| PlayGroundMsg::GotoEdit())} text={"edit"}/>
                    </>
                }
            }
            PlayGroundState::Edit(maybe_initial_locpos) => {
                let on_log = Callback::from(|string: String| log(string));
                let on_goto_test: Callback<(FinGraph, _, _, _)> =
                    ctx.link().callback(PlayGroundMsg::GotoTest);
                html! {
                    <GraphicEditor {on_goto_test} {on_log} {logic_circuits_components} {maybe_initial_locpos}/>
                }
            }
        }
    }
    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            PlayGroundMsg::GotoEdit() => {
                let PlayGroundState::Test(fingraph, pos, inpins_pos, otpins_pos) =
                    self.state.clone()
                else {
                    unreachable!("不整合")
                };
                let lcs = fingraph.get_lc_names();
                let name_to_num =
                    |name: &Name| -> usize { lcs.iter().position(|n| n == name).unwrap() };
                let inpins: Vec<(InPin, Pos)> = fingraph
                    .get_inpins()
                    .into_iter()
                    .map(|v| v.0)
                    .zip(inpins_pos)
                    .collect::<Vec<_>>();
                let otpins: Vec<(OtPin, Pos)> = fingraph
                    .get_otpins()
                    .into_iter()
                    .map(|v| v.0)
                    .zip(otpins_pos)
                    .collect::<Vec<_>>();
                let component: Vec<(usize, Pos, Ori)> = lcs
                    .iter()
                    .map(|name| {
                        let pos_loc = pos.iter().position(|(name2, _)| name == name2).unwrap();
                        (pos_loc, pos[pos_loc].1 .0, pos[pos_loc].1 .1)
                    })
                    .collect();
                let edges: Vec<((usize, InPinNum), (usize, OtPinNum))> = fingraph
                    .edges()
                    .iter()
                    .map(|(no, ni)| {
                        let inpin_num = {
                            let lc_inpins = fingraph.get_inpins_of_lc(&ni.0).unwrap();
                            lc_inpins
                                .into_iter()
                                .position(|inpin| ni.1 == inpin.0)
                                .unwrap()
                        };
                        let otpin_num = {
                            let lc_otpins = fingraph.get_otpins_of_lc(&no.0).unwrap();
                            lc_otpins
                                .into_iter()
                                .position(|otpin| no.1 == otpin.0)
                                .unwrap()
                        };
                        (
                            (name_to_num(&no.0), otpin_num),
                            (name_to_num(&ni.0), inpin_num),
                        )
                    })
                    .collect();
                let inputs_edge: Vec<(InPinNum, (usize, InPinNum))> = inpins
                    .iter()
                    .enumerate()
                    .map(|(k, (i, _))| {
                        let ni = fingraph.get_inpin_to_lc_inpin(i).unwrap();
                        let lc_inpin_list = fingraph.get_inpins_of_lc(&ni.0).unwrap();
                        let pos = lc_inpin_list
                            .into_iter()
                            .position(|inpin| inpin.0 == *i)
                            .unwrap();
                        (k, (name_to_num(&ni.0), pos))
                    })
                    .collect::<Vec<_>>();
                let otputs_edge: Vec<(OtPinNum, (usize, OtPinNum))> = otpins
                    .iter()
                    .enumerate()
                    .map(|(k, (o, _))| {
                        let no = fingraph.get_otpin_to_lc_otpin(o).unwrap();
                        let lc_otpin_list = fingraph.get_otpins_of_lc(&no.0).unwrap();
                        let pos = lc_otpin_list
                            .into_iter()
                            .position(|otpin| otpin.0 == *o)
                            .unwrap();
                        (k, (name_to_num(&no.0), pos))
                    })
                    .collect::<Vec<_>>();
                let allposition = AllPositions {
                    inpins,
                    otpins,
                    component,
                    edges,
                    inputs_edge,
                    otputs_edge,
                };
                self.state = PlayGroundState::Edit(Some(allposition));
            }
            PlayGroundMsg::GotoTest((loc, pos, inputs_pos, otputs_pos)) => {
                self.state = PlayGroundState::Test(loc, pos, inputs_pos, otputs_pos);
            }
        }
        true
    }
}
