use super::*;
use anyhow::{bail, Result};
use either::Either;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fmt::Display};
use utils::view::{log, svg::*};

const BOOL_T_COL: &str = "salmon";
const BOOL_F_COL: &str = "skyblue";

const WIDTH_LC: usize = 50;
const PIN_LEN: usize = 25;
const PIN_RAD: usize = 7;

const COMP_LINE: usize = 400;
const COMP_LEN: usize = 100;

const PIN_TEXT_SIZE: usize = 8;
const LOC_TEXT_SIZE: usize = 15;

const INIT_INPIN_POS: Pos = Pos(50, 100);
const INIT_OTPIN_POS: Pos = Pos(800, 100);

#[derive(Debug, Clone, PartialEq, Properties)]
struct InPinProps {
    pos: Pos,
    state: Bool,
    inpin: InPin,
    onmousedown: Callback<()>,
    onmouseup: Callback<()>,
    onclick: Callback<()>,
}

#[function_component(InPinView)]
fn inpin_view(
    InPinProps {
        pos,
        state,
        inpin,
        onmousedown,
        onmouseup,
        onclick,
    }: &InPinProps,
) -> Html {
    let onmousedown = onmousedown.clone();
    let onmouseup = onmouseup.clone();
    let onclick = onclick.clone();
    html! {
        <>
        <CircleView pos={*pos} rad={PIN_RAD} col={if *state == Bool::T {BOOL_T_COL.to_string()} else {BOOL_F_COL.to_string()}} border="black" onmousedown={Callback::from(move |_|{onmousedown.emit(())})} onmouseup={Callback::from(move |_| onmouseup.emit(()))} onclick={Callback::from(move |_| onclick.emit(()))}/>
        <TextView pos={*pos + Diff(PIN_RAD as isize , 0)} text={inpin.to_string()} size={PIN_TEXT_SIZE}/>
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
        <CircleView pos={*pos} rad={PIN_RAD} col={if *state == Bool::T {BOOL_T_COL.to_string()} else {BOOL_F_COL.to_string()}} border="black" onmousedown={Callback::from(move |_| onmousedown.emit(()))}/>
        <TextView pos={*pos + Diff(- ((otpin.len() * PIN_TEXT_SIZE) as isize), 0)} text={otpin.to_string()} size={PIN_TEXT_SIZE}/>
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
    onmousedownlc: Callback<Diff>,
    onmousedowninpin: Callback<usize>,
    onmousedownotpin: Callback<usize>,
    onrightclick: Callback<()>,
}

impl LoCProps {
    fn input_pos(&self, inpin: &InPin) -> Option<Pos> {
        let k = self.inputs.iter().position(|i| &i.0 == inpin)?;
        self.input_pos_fromnum(&k)
    }

    fn input_pos_fromnum(&self, inpinnum: &usize) -> Option<Pos> {
        if self.inputs.len() <= *inpinnum {
            return None;
        }
        let diff = Diff(0, PIN_LEN as isize) * *inpinnum;
        Some(self.pos - rot(self.rect_diff_row() / 2 - diff, self.ori))
    }

    fn otput_pos(&self, otpin: &OtPin) -> Option<Pos> {
        let k = self.otputs.iter().position(|o| &o.0 == otpin)?;
        self.otput_pos_fromnum(&k)
    }

    fn otput_pos_fromnum(&self, otpinnum: &usize) -> Option<Pos> {
        if self.otputs.len() <= *otpinnum {
            return None;
        }
        let diff = Diff(0, PIN_LEN as isize) * *otpinnum;
        let mut diff_rect = self.rect_diff_row();
        diff_rect.0 = -diff_rect.0;
        Some(self.pos - rot(diff_rect / 2 - diff, self.ori))
    }

    fn rect_lu(&self) -> Pos {
        let m = std::cmp::max(self.inputs.len(), self.otputs.len());
        let diff = Diff(WIDTH_LC as isize, (PIN_LEN * m) as isize);
        self.pos - rot(diff / 2, self.ori)
    }

    fn rect_diff_row(&self) -> Diff {
        let m = std::cmp::max(self.inputs.len(), self.otputs.len());
        Diff(WIDTH_LC as isize, (PIN_LEN * m) as isize)
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
        name,
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
    let name: String = format!("{name}");
    let text_pos: Pos = pos - Diff(LOC_TEXT_SIZE as isize, 0) * (name.len() / 2);
    html! {
        <>
            <RectView pos={locprops.rect_lu()} diff={rot(locprops.rect_diff(), ori)} col={"lightgray".to_string()} border={"black".to_string()} onmousedown={onmousedownlc} oncontextmenu={onrightclick}/>
            <TextView pos={text_pos} text={name} size={LOC_TEXT_SIZE}/>
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
            let LoCProps { name, inputs, otputs, ori, pos, onmousedownlc, onmousedowninpin, onmousedownotpin, onrightclick } = actlocprops.get_lc_props(name).unwrap();
            html!{
                <LoCView {name} {inputs} {otputs} {ori} {pos} {onmousedownlc} {onmousedowninpin} {onmousedownotpin} {onrightclick}/>
            }
        })}
        {for inpin_edge.into_iter().zip(inpin_callback).map(|((inpin, pos_i, pos_ni, state), onmousedown)|{
            html!{
                <>
                <InPinView pos={pos_i} {inpin} {state} {onmousedown}/>
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

type PinVariant = Either<Either<InPinNum, (usize, InPinNum)>, Either<OtPinNum, (usize, OtPinNum)>>;

#[derive(Debug, Clone, PartialEq, Eq)]
enum State {
    None,
    MoveLC(usize, Diff),
    MoveInPin(usize),
    MoveOtPin(usize),
    SelectPin(PinVariant),
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct AllPositions {
    inpins: Vec<(InPin, Pos)>,
    otpins: Vec<(OtPin, Pos)>,
    component: Vec<(usize, Pos, Ori)>,
    edges: Vec<((usize, InPinNum), (usize, OtPinNum))>,
    inputs_edge: Vec<(InPinNum, (usize, InPinNum))>,
    otputs_edge: Vec<(OtPinNum, (usize, OtPinNum))>,
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

    SelectCopy(usize, Diff),
    SelectMove(usize, Diff),
    SelectInPin(usize, Diff),
    SelectOtPin(usize, Diff),
    SelectPin(PinVariant),
    UnSelect,
    Update(Pos),

    RotClock(usize),
    RotCount(usize),

    Delete(usize),

    GoToTest,
    Load(serde_json::Value),
}

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

        let temp_json = serde_json::to_value(allpositions.clone()).unwrap();

        let on_drop_json = ctx
            .link()
            .callback(|json: serde_json::Value| GraphicEditorMsg::Load(json));

        let loc_vec = (0..allpositions.component.len())
            .map(|k| {
                let (num, pos, ori) = allpositions.component[k].clone();
                let loc = &logic_circuits_components[num];
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
                let onrightclick = ctx.link().callback(move |_| GraphicEditorMsg::Delete(k));
                LoCProps {
                    name: loc.get_name(),
                    inputs,
                    otputs,
                    pos,
                    ori,
                    onmousedownlc,
                    onmousedowninpin,
                    onmousedownotpin,
                    onrightclick,
                }
            })
            .collect::<Vec<_>>();

        let inpins_edge = (0..allpositions.inpins.len())
            .map(|k| {
                let (i, (k, i1)) = allpositions.inputs_edge[k].clone();
                let pos_i = allpositions.inpins[k].clone();
                let pos_i2 = loc_vec[k].input_pos_fromnum(&i1).unwrap();
                (pos_i, pos_i2)
            })
            .collect::<Vec<_>>();

        let otpins_edge = (0..allpositions.otpins.len())
            .map(|k| {
                let (o, (k, o1)) = allpositions.otputs_edge[k].clone();
                let pos_o = allpositions.otpins[k].clone();
                let pos_o2 = loc_vec[k].otput_pos_fromnum(&o1).unwrap();
                (pos_o, pos_o2)
            })
            .collect::<Vec<_>>();

        let graph_edges = allpositions
            .edges
            .iter()
            .map(|((ko, o), (ki, i))| {
                let pos_o = loc_vec[*ko].otput_pos_fromnum(o).unwrap();
                let pos_i = loc_vec[*ki].input_pos_fromnum(i).unwrap();
                (pos_o, pos_i)
            })
            .collect::<Vec<_>>();

        let width = (logic_circuits_components.len()) * COMP_LEN;

        html! {
            <div height="500" width="900" border="solid #000" overflow="scroll">
            <svg width={width.to_string()} height="500" viewBox={format!("0 0 {width} 500")} {onmousemove} onmouseup={onmouseuporleave.clone()} onmouseleave={onmouseuporleave}>
                {for loc_vec.into_iter().map(|locprop|{
                    let LoCProps {name, inputs, otputs, pos, ori, onmousedownlc, onmousedowninpin, onmousedownotpin, onrightclick} = locprop;
                    html!{
                        <LoCView {name} {inputs} {otputs} pos={pos} ori={ori} {onmousedownlc} {onmousedowninpin} {onmousedownotpin} {onrightclick}/>
                    }
                })}
                {for allpositions.inpins.into_iter().enumerate().map(|(k, (inpin, pos))|{
                    let onmousedown = ctx.link().callback(move |_|{
                        GraphicEditorMsg::SelectPin(Either::Left(Either::Left(k)))
                    });
                    html!{
                        <InPinView inpin={inpin.clone()} {pos} state={Bool::F} {onmousedown}/>
                    }
                })}
                {for allpositions.otpins.into_iter().enumerate().map(|(k, (otpin, pos))|{
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
                        <LoCView name={loc.get_name()} {inputs} {otputs} {pos} ori={Ori::U} {onmousedownlc} onmousedowninpin={Callback::noop()} onmousedownotpin={Callback::noop()} onrightclick={Callback::noop()}/>
                    }
                })}
            </svg> <br/>
            <utils::view::InputText description={"add inpins".to_string()} on_push_load_button={add_inpins}/>
            <utils::view::InputText description={"add otpins".to_string()} on_push_load_button={add_otpins}/> <br/>
            <utils::view::InputText description={"remove inpins".to_string()} on_push_load_button={remove_inpins}/>
            <utils::view::InputText description={"remove otpins".to_string()} on_push_load_button={remove_otpins}/>
            // <button onclick={goto_test}> {"test"} </button>
            <utils::view::ButtonView on_click={goto_test} text={"test"}/>
            <utils::view::JsonFileSaveView json_value={temp_json}/>
            <utils::view::JsonFileReadView {on_drop_json}/>
            </div>
        }
    }
    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        let prop = ctx.props();
        if !matches!(msg, GraphicEditorMsg::Update(_)) {
            prop.on_log.emit(format!("{msg:?} {:?}", self.state));
        }
        match msg {
            GraphicEditorMsg::AddInpin(inpin) => {
                if self.allpositions.inpins.iter().any(|(i, _)| *i == inpin) {
                    return false;
                }
                self.allpositions.inpins.push((inpin,));
            }
            GraphicEditorMsg::AddOtPin(otpin) => {
                if self.allpositions.otpins.iter().any(|(o, _)| *o.0 == otpin) {
                    return false;
                }
                self.allpositions.otpins.push(otpin);
            }
            GraphicEditorMsg::DeleteInPin(inpin) => {
                self.allpositions.inpins.retain(|i| *i.0 != inpin);
            }
            GraphicEditorMsg::DeleteOtPin(otpin) => {
                self.allpositions.otpins.retain(|o| *o.0 != otpin);
            }
            GraphicEditorMsg::SelectCopy(k, diff) => {
                let pos = Pos(k * COMP_LEN, COMP_LINE);
                self.allpositions.component.push((k, pos, Ori::U));
                self.state = State::MoveLC(self.allpositions.component.len() - 1, diff);
            }
            GraphicEditorMsg::SelectMove(k, diff) => {
                self.state = State::MoveLC(k, diff);
            }
            GraphicEditorMsg::RotClock(k) => {
                self.allpositions.component[k].2.rot_clockwise();
            }
            GraphicEditorMsg::RotCount(k) => {
                self.allpositions.component[k].2.rot_counterclockwise();
            }
            GraphicEditorMsg::Update(pos) => match &self.state {
                State::None => {
                    return false;
                }
                State::MoveLC(k, diff) => {
                    self.allpositions.component[*k].1 = pos - *diff;
                }
                State::MoveInPin(k, diff) => {}
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
                    // self.state = State::None;
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
            GraphicEditorMsg::Delete(k) => {
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
            GraphicEditorMsg::Load(json) => {
                let Ok((inpins, otpins, component, edges, inputs, otputs)): Result<
                    AllPositions,
                    _,
                > = serde_json::from_value(json) else {
                    return false;
                };
                self.inpins = inpins;
                self.otpins = otpins;
                self.component = component;
                self.edges = edges;
                self.inputs = inputs;
                self.otputs = otputs;
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
            PlayGroundState::Test(init_fingraph, init_pos_lc) => {
                let json_loc = serde_json::to_value(&init_fingraph).unwrap();
                html! {
                    <>
                    <FingraphMachine {init_fingraph} {init_pos_lc}/>
                    <utils::view::JsonFileSaveView json_value={json_loc} />
                    <utils::view::ButtonView on_click={ctx.link().callback(|_| PlayGroundMsg::GotoEdit())} text={"edit"}/>
                    </>
                }
            }
            PlayGroundState::Edit(maybe_initial_locpos) => {
                let on_log = Callback::from(|string: String| log(string));
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
                let lcs = fingraph.get_lc_names();
                let name_to_num =
                    |name: &Name| -> usize { lcs.iter().position(|n| n == name).unwrap() };
                let inpins: Vec<InPin> = fingraph
                    .get_inpins()
                    .into_iter()
                    .map(|v| v.0)
                    .collect::<Vec<_>>();
                let otpins: Vec<OtPin> = fingraph
                    .get_otpins()
                    .into_iter()
                    .map(|v| v.0)
                    .collect::<Vec<_>>();
                let loc_positions: Vec<(usize, Pos, Ori)> = lcs
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
                let inputs: Vec<(InPinNum, (usize, InPinNum))> = inpins
                    .iter()
                    .enumerate()
                    .map(|(k, i)| {
                        let ni = fingraph.get_inpin_to_lc_inpin(i).unwrap();
                        let lc_inpin_list = fingraph.get_inpins_of_lc(&ni.0).unwrap();
                        let pos = lc_inpin_list
                            .into_iter()
                            .position(|inpin| inpin.0 == *i)
                            .unwrap();
                        (k, (name_to_num(&ni.0), pos))
                    })
                    .collect::<Vec<_>>();
                let otputs: Vec<(OtPinNum, (usize, OtPinNum))> = otpins
                    .iter()
                    .enumerate()
                    .map(|(k, o)| {
                        let no = fingraph.get_otpin_to_lc_otpin(o).unwrap();
                        let lc_otpin_list = fingraph.get_otpins_of_lc(&no.0).unwrap();
                        let pos = lc_otpin_list
                            .into_iter()
                            .position(|otpin| otpin.0 == *o)
                            .unwrap();
                        (k, (name_to_num(&no.0), pos))
                    })
                    .collect::<Vec<_>>();
                let allposition: AllPositions =
                    (inpins, otpins, loc_positions, edges, inputs, otputs);
                self.state = PlayGroundState::Edit(Some(allposition));
            }
            PlayGroundMsg::GotoTest((loc, poss)) => {
                self.state = PlayGroundState::Test(loc, poss);
            }
        }
        true
    }
}
