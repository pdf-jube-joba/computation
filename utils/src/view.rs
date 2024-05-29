use anyhow::bail;
use gloo::file::callbacks::FileReader;
use gloo::timers::callback::Interval;
use wasm_bindgen::JsValue;
use web_sys::HtmlInputElement;
use yew::prelude::*;
use yew::Properties;

pub fn log<T: AsRef<str>>(str: T) {
    web_sys::console::log_1(&JsValue::from_str(str.as_ref()))
}

#[derive(Debug, Clone, PartialEq, Properties)]
pub struct ButtonProps {
    pub on_click: Callback<MouseEvent>,
    pub text: String,
}

#[function_component(ButtonView)]
pub fn button_view(ButtonProps { on_click, text }: &ButtonProps) -> Html {
    html! {
        <button onclick={on_click}> {text} </button>
    }
}

#[derive(Debug, Default, Clone, PartialEq)]
pub struct CodeView {
    source_code: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum CodeMsg {
    Load,
    Update(String),
}

#[derive(Debug, Clone, PartialEq, Properties)]
pub struct CodeProps {
    pub on_load: Callback<String>,
}

impl Component for CodeView {
    type Message = CodeMsg;
    type Properties = CodeProps;
    fn create(_ctx: &Context<Self>) -> Self {
        Self::default()
    }
    fn view(&self, ctx: &Context<Self>) -> Html {
        let load_callback = ctx.link().callback(|_| CodeMsg::Load);
        let oninput = ctx.link().callback(|e: InputEvent| {
            let value: HtmlInputElement = e.target_unchecked_into();
            let s: String = value.value();
            CodeMsg::Update(s)
        });
        html! {
            <div class="code">
            {"code"} <br/>
            <div class="box">
                <textarea rows={30} cols={50} oninput={oninput}/>
            </div>
            <div class="box">
                <button onclick={load_callback}> {"load"} </button>
            </div>
            </div>
        }
    }
    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            CodeMsg::Update(str) => {
                self.source_code = str;
            }
            CodeMsg::Load => {
                let CodeProps { on_load } = ctx.props();
                on_load.emit(self.source_code.clone());
            }
        }
        true
    }
}

#[derive(Debug, Default, Clone, PartialEq)]
pub struct EventLogView {
    log: Vec<String>,
}
#[derive(Debug, Clone, PartialEq)]
pub enum EventLogMsg {
    Log(String),
}
#[derive(Debug, Default, Clone, PartialEq, Properties)]
pub struct EventLogProps {}
impl Component for EventLogView {
    type Message = EventLogMsg;
    type Properties = EventLogProps;
    fn create(_ctx: &Context<Self>) -> Self {
        Self::default()
    }
    fn view(&self, _ctx: &Context<Self>) -> Html {
        html! {
            <div class="log">
            {"eventlog"} <br/>
            {
                for self.log.iter().rev().take(10).map(|s| html!{<> {s} <br/> </>})
            }
            </div>
        }
    }
    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        let EventLogMsg::Log(log) = msg;
        self.log.push(log);
        true
    }
}

#[derive(Debug)]
pub struct ControlStepView {
    now_auto: bool,
    now_secs: u32,
    #[allow(dead_code)]
    total_step: usize,
    #[allow(dead_code)]
    interval: Interval,
    now_input_step: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ControlStepMsg {
    Toggle,
    Tick,
    ChangeSecs(u32),
    ChangeStep(usize),
    Step,
}

#[derive(Debug, Clone, PartialEq, Properties)]
pub struct ControlStepProps {
    pub on_step: Callback<usize>,
}

impl Component for ControlStepView {
    type Message = ControlStepMsg;
    type Properties = ControlStepProps;
    fn create(ctx: &Context<Self>) -> Self {
        let callback = ctx.link().callback(|_| ControlStepMsg::Tick);
        let interval = Interval::new(1000, move || callback.emit(()));
        Self {
            now_input_step: 1,
            now_secs: 1000,
            interval,
            total_step: 0,
            now_auto: false,
        }
    }
    fn view(&self, ctx: &Context<Self>) -> Html {
        let onchange_step = ctx.link().callback(|e: Event| {
            let value: HtmlInputElement = e.target_unchecked_into();
            let str = value.value();
            let step: usize = str.parse().unwrap_or(1000);
            ControlStepMsg::ChangeStep(step)
        });
        let onchange_secs = ctx.link().callback(|e: Event| {
            let value: HtmlInputElement = e.target_unchecked_into();
            let str = value.value();
            let step: u32 = str.parse().unwrap_or(1);
            ControlStepMsg::ChangeSecs(step)
        });
        let onclick_input = ctx.link().callback(|_| ControlStepMsg::Step);
        let onclick_toggle = ctx.link().callback(|_| ControlStepMsg::Toggle);
        html! {
            <>
                <input onchange={onchange_step}/>
                <button onclick={onclick_input}> {{self.now_input_step}} {"step"} </button>
                <input onchange={onchange_secs}/>
                <button onclick={onclick_toggle}> {"auto step:"} {{if self.now_auto {"on"} else {"off"}}} {"per"} {self.now_secs} </button>
            </>
        }
    }
    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        let ControlStepProps { on_step } = ctx.props();
        match msg {
            ControlStepMsg::ChangeStep(step) => {
                self.now_input_step = step;
            }
            ControlStepMsg::Tick => {
                if self.now_auto {
                    on_step.emit(self.now_input_step);
                    // self.total_step += self.now_input_step;
                }
            }
            ControlStepMsg::Toggle => {
                self.now_auto = !self.now_auto;
            }
            ControlStepMsg::Step => {
                on_step.emit(self.now_input_step);
                // self.total_step += self.now_input_step;
            }
            ControlStepMsg::ChangeSecs(secs) => {
                self.now_secs = secs;
                let callback = ctx.link().callback(|_| ControlStepMsg::Tick);
                let interval = Interval::new(secs, move || callback.emit(()));
                self.interval = interval;
            }
        }
        true
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct InputText {
    text: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum InputTextMsg {
    Change(String),
}

#[derive(Debug, Clone, PartialEq, Properties)]
pub struct InputTextProps {
    pub description: String,
    pub on_push_load_button: Callback<String>,
}

impl Component for InputText {
    type Message = InputTextMsg;
    type Properties = InputTextProps;
    fn create(_ctx: &Context<Self>) -> Self {
        Self {
            text: String::new(),
        }
    }
    fn view(&self, ctx: &Context<Self>) -> Html {
        let onchange = ctx.link().callback(|e: Event| {
            let value: HtmlInputElement = e.target_unchecked_into();
            let string = value.value();
            InputTextMsg::Change(string)
        });
        let InputTextProps {
            description,
            on_push_load_button,
        } = ctx.props();
        let callback = on_push_load_button.clone();
        let text = self.text.clone();
        let onclick = Callback::from(move |_| {
            callback.emit(text.clone());
        });
        html! {
            <>
            <input type="text" {onchange}/>
            <button {onclick}> {description} </button>
            </>
        }
    }
    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            InputTextMsg::Change(string) => {
                self.text = string;
                true
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Properties)]
pub struct JsonFileSaveProps {
    pub json_value: serde_json::Value,
}

#[function_component(JsonFileSaveView)]
pub fn json_file_save_view(JsonFileSaveProps { json_value }: &JsonFileSaveProps) -> Html {
    let head_string = "data:text/json;charset=utf-8,";
    let data = json_value.to_string();
    html! {
        <a href={format!("{}{}", head_string, data)} download="data.json"> {"save as json"}</a>
    }
}

pub enum JsonFileReadMsg {
    Read(DragEvent),
    LoadEnd(Result<String, anyhow::Error>),
}

#[derive(Debug, Clone, PartialEq, Properties)]
pub struct JsonFileReadProps {
    pub on_drop_json: Callback<serde_json::Value>,
}

mod useless {
    use super::*;

    pub fn read_json_from_file(dragevent: DragEvent) -> Result<serde_json::Value, anyhow::Error> {
        let Some(data_transfer) = dragevent.data_transfer() else {
            bail!("data transfer fail")
        };
        let Some(files) = data_transfer.files() else {
            bail!("files fail")
        };
        let Some(file) = files.get(0) else {
            bail!("file fail")
        };
        let reader = match web_sys::FileReaderSync::new() {
            Ok(reader) => reader,
            Err(err) => {
                bail!("fail {err:?}");
            }
        };
        let string = match reader.read_as_text(&file) {
            Ok(string) => string,
            Err(err) => {
                bail!("fail {err:?}");
            }
        };
        serde_json::from_str(&string).map_err(|e| e.into())
    }

    #[function_component(JsonFileReadView)]
    fn json_file_read_view(JsonFileReadProps { on_drop_json }: &JsonFileReadProps) -> Html {
        let callback = on_drop_json.clone();
        html! {<>
            <div id="drop-container"
                ondrop={Callback::from(move |e: DragEvent|{
                    e.prevent_default();
                    match read_json_from_file(e) {
                        Ok(json) => {
                            callback.emit(json);
                        }
                        Err(err) => {log(format!("{err}"));}
                    };
                })}
                ondragover={Callback::from(|event: DragEvent| {
                    event.prevent_default();
                })}
                ondragenter={Callback::from(|event: DragEvent| {
                    event.prevent_default();
                })}
            > <p> {"drop here"} </p> </div>
        </>}
    }
}

#[derive(Debug, Default)]
pub struct JsonFileReadView {
    reader: Option<FileReader>,
}

impl Component for JsonFileReadView {
    type Message = JsonFileReadMsg;
    type Properties = JsonFileReadProps;
    fn create(ctx: &Context<Self>) -> Self {
        Self::default()
    }
    fn view(&self, ctx: &Context<Self>) -> Html {
        html! {
            <>
            <div id="drop-container"
                ondrop={ctx.link().callback(|event: DragEvent|{
                    event.prevent_default();
                    JsonFileReadMsg::Read(event)
                })}
                ondragover={Callback::from(|event: DragEvent| {
                    event.prevent_default();
                })}
                ondragenter={Callback::from(|event: DragEvent| {
                    event.prevent_default();
                })}
            > <p> {"drop here"} </p> </div>
            </>
        }
    }
    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            JsonFileReadMsg::Read(dragevent) => {
                let read = move |event: DragEvent| -> Result<FileReader, anyhow::Error> {
                    let Some(data_transfer) = event.data_transfer() else {
                        bail!("data transfer fail")
                    };
                    let Some(files) = data_transfer.files() else {
                        bail!("files fail")
                    };
                    let Some(file) = files.get(0) else {
                        bail!("file fail")
                    };
                    let file: gloo::file::File = file.into();
                    let link = ctx.link().clone();
                    let task = gloo::file::callbacks::read_as_text(&file, move |res| {
                        link.send_message(JsonFileReadMsg::LoadEnd(res.map_err(|e| e.into())))
                    });
                    Ok(task)
                };
                match read(dragevent) {
                    Ok(task) => {
                        self.reader = Some(task);
                    }
                    Err(err) => {
                        log(format!("{err:?}"));
                    }
                }
                true
            }
            JsonFileReadMsg::LoadEnd(res) => {
                match res {
                    Ok(string) => match serde_json::from_str(&string) {
                        Ok(val) => {
                            ctx.props().on_drop_json.emit(val);
                        }
                        Err(err) => {
                            log(format!("{err:?}"));
                        }
                    },
                    Err(err) => {
                        log(format!("{err:?}"));
                    }
                }
                true
            }
        }
    }
}

pub mod svg {
    use serde::{Deserialize, Serialize};
    use std::{
        fmt::Display,
        ops::{Add, Div, Mul, Neg, Sub},
    };
    use yew::prelude::*;

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
    pub struct Pos(pub usize, pub usize);

    impl Pos {
        pub fn abs_diff(&self, other: &Pos) -> usize {
            self.0.abs_diff(other.0).pow(2) + self.1.abs_diff(other.1).pow(1)
        }
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
    pub struct Diff(pub isize, pub isize);

    impl Diff {
        pub fn rot_counterclockwise(&mut self) {
            *self = Diff(-self.1, self.0);
        }
        pub fn rot_clockwise(&mut self) {
            *self = Diff(self.1, -self.0);
        }
    }

    impl Add<Diff> for Diff {
        type Output = Diff;
        fn add(self, rhs: Diff) -> Self::Output {
            Diff(self.0 + rhs.0, self.1 + rhs.1)
        }
    }

    impl Mul<usize> for Diff {
        type Output = Diff;
        fn mul(self, rhs: usize) -> Self::Output {
            Diff(self.0 * (rhs as isize), self.1 * (rhs as isize))
        }
    }

    impl Div<usize> for Diff {
        type Output = Diff;
        fn div(self, rhs: usize) -> Self::Output {
            Diff(self.0 / (rhs as isize), self.1 / (rhs as isize))
        }
    }

    impl Neg for Diff {
        type Output = Diff;
        fn neg(self) -> Self::Output {
            Diff(-self.0, -self.1)
        }
    }

    impl Sub<Diff> for Diff {
        type Output = Diff;
        fn sub(self, rhs: Diff) -> Self::Output {
            Diff(self.0 - rhs.0, self.1 - rhs.1)
        }
    }

    impl Add<Diff> for Pos {
        type Output = Pos;
        fn add(self, rhs: Diff) -> Self::Output {
            let x = if rhs.0.is_positive() {
                self.0 + rhs.0 as usize
            } else {
                self.0 - (-rhs.0 as usize)
            };
            let y = if rhs.1.is_positive() {
                self.1 + rhs.1 as usize
            } else {
                self.1 - (-rhs.1 as usize)
            };
            Pos(x, y)
        }
    }

    impl Sub<Pos> for Pos {
        type Output = Diff;
        fn sub(self, rhs: Self) -> Self::Output {
            Diff(
                self.0 as isize - rhs.0 as isize,
                self.1 as isize - rhs.1 as isize,
            )
        }
    }

    impl Sub<Diff> for Pos {
        type Output = Pos;
        fn sub(self, rhs: Diff) -> Self::Output {
            Pos(
                (self.0 as isize - rhs.0) as usize,
                (self.1 as isize - rhs.1) as usize,
            )
        }
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
    pub enum Ori {
        U,
        R,
        D,
        L,
    }

    impl Display for Ori {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            let s = match self {
                Ori::D => "D",
                Ori::L => "L",
                Ori::U => "U",
                Ori::R => "R",
            };
            write!(f, "{}", s)
        }
    }

    #[derive(Debug, Clone, PartialEq, Properties)]
    pub struct RectProps {
        pub pos: Pos,
        pub diff: Diff,
        pub col: String,
        pub border: String,
        pub onmousedown: Callback<MouseEvent>,
        pub oncontextmenu: Callback<MouseEvent>,
    }

    #[function_component(RectView)]
    pub fn rect_view(
        RectProps {
            pos,
            diff,
            col,
            border,
            onmousedown,
            oncontextmenu,
        }: &RectProps,
    ) -> Html {
        html! {
            <rect x={pos.0.to_string()} y={pos.1.to_string()} width={diff.0.to_string()} height={diff.1.to_string()} fill={col.to_string()} stroke={border.to_string()} {onmousedown} {oncontextmenu}/>
        }
    }

    #[derive(Debug, Clone, PartialEq, Properties)]
    pub struct CircleProps {
        pub pos: Pos,
        pub rad: usize,
        pub col: String,
        pub border: String,
        pub onmousedown: Callback<MouseEvent>,
    }

    #[function_component(CircleView)]
    pub fn circle_view(
        CircleProps {
            pos,
            rad,
            col,
            border,
            onmousedown,
        }: &CircleProps,
    ) -> Html {
        html! {
            <circle cx={pos.0.to_string()} cy={pos.1.to_string()} r={rad.to_string()} fill={col.to_string()} stroke={border.to_string()} {onmousedown}/>
        }
    }

    #[derive(Debug, Clone, PartialEq, Eq, Properties, Hash)]
    pub struct TextProps {
        pub pos: Pos,
        pub text: String,
        pub size: usize,
    }

    #[function_component(TextView)]
    pub fn text_view(TextProps { pos, text, size }: &TextProps) -> Html {
        html! {
            <text x={pos.0.to_string()} y={pos.1.to_string()} font-size={size.to_string()}> {text} </text>
        }
    }

    #[derive(Debug, Clone, PartialEq, Eq, Properties, Hash)]
    pub struct PolyLineProps {
        pub vec: Vec<Pos>,
        pub col: String,
    }

    #[function_component(PolyLineView)]
    pub fn path_view(PolyLineProps { vec, col }: &PolyLineProps) -> Html {
        let s = vec.iter().fold(String::new(), |string, vi| {
            format!("{string} {},{}", vi.0, vi.1)
        });
        html! {
            <polyline points={s} fill="none" stroke={col.to_string()}/>
        }
    }
}
