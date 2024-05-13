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
    on_load: Callback<String>,
}

impl Component for CodeView {
    type Message = CodeMsg;
    type Properties = CodeProps;
    fn create(ctx: &Context<Self>) -> Self {
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
                <textarea row="30" oninput={oninput}/>
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
    fn create(ctx: &Context<Self>) -> Self {
        Self::default()
    }
    fn view(&self, ctx: &Context<Self>) -> Html {
        html! {
            <>
            {"eventlog"} <br/>
            {
                for self.log.iter().rev().take(10).map(|s| html!{<> {s} <br/> </>})
            }
            </>
        }
    }
    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        let EventLogMsg::Log(log) = msg;
        self.log.push(log);
        true
    }
}


// #[derive(Debug, Clone, PartialEq)]
// pub struct ControlStepView {
//     now_input_step: Result<usize, ()>,
// }

// #[derive(Debug, Clone, PartialEq)]
// pub struct ControlStepMsg {}

// #[derive(Debug, Clone, PartialEq, Properties)]
// pub struct ControlStepProps {}

// impl Component for ControlStepView {
//     type Message = ControlStepMsg;
//     type Properties = ControlStepProps;
//     fn create(ctx: &Context<Self>) -> Self {
        
//     }
// }
