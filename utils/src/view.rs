use gloo::timers::callback::Interval;
use web_sys::HtmlInputElement;
use yew::prelude::*;
use yew::Properties;

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
                <textarea row={30} oninput={oninput}/>
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
            <>
            {"eventlog"} <br/>
            {
                for self.log.iter().rev().take(10).map(|s| html!{<> {s} <br/> </>})
            }
            </>
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
