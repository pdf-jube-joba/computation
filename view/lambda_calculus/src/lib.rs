use lambda_calculus::{machine::*, manipulation};
use wasm_bindgen::prelude::*;
use web_sys::HtmlInputElement;
use yew::prelude::*;

#[derive(Debug, Clone, PartialEq, Properties)]
struct VarBoxProps {
    var: Var,
}

#[function_component(VarBox)]
fn var_box_view(VarBoxProps { var }: &VarBoxProps) -> Html {
    html! {
        var.to_string()
    }
}

#[derive(Default)]
pub struct InputParseOnelineView {
    parse_result: Option<LambdaTerm>,
}

pub enum InputParseOnelineMsg {
    Change(String),
    Ok,
}

#[derive(Debug, Clone, PartialEq, Properties)]
pub struct InputParseOnelineProps {
    input_term_call: Callback<LambdaTerm>,
}

impl Component for InputParseOnelineView {
    type Message = InputParseOnelineMsg;
    type Properties = InputParseOnelineProps;
    fn create(_ctx: &Context<Self>) -> Self {
        Self::default()
    }
    fn view(&self, ctx: &Context<Self>) -> Html {
        let onchange_input = ctx.link().callback(|e: Event| {
            let value: HtmlInputElement = e.target_unchecked_into();
            let str = value.value();
            InputParseOnelineMsg::Change(str)
        });
        let parse_result = if let Some(term) = &self.parse_result {
            term.to_string()
        } else {
            "no term found".to_owned()
        };
        html! {
            <>
            <input onchange={onchange_input}/> {"result"} {parse_result} <br/>
            <button onclick={ctx.link().callback(|_| InputParseOnelineMsg::Ok)}> {"ok"} </button>
            </>
        }
    }
    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            InputParseOnelineMsg::Change(str) => {
                self.parse_result = manipulation::parse::parse_lambda(&str);
                true
            }
            InputParseOnelineMsg::Ok => {
                web_sys::console::log_1(&JsValue::from_str("hello"));
                if let Some(term) = &self.parse_result {
                    let InputParseOnelineProps { input_term_call } = ctx.props();
                    input_term_call.emit(term.clone());
                }
                false
            }
        }
    }
}

struct ReduceHistoryView {}

#[derive(Debug, Clone, PartialEq, Properties)]
struct ReduceHistoryProps {
    vec: Vec<LambdaTerm>,
}

enum ReduceHistoryMsg {}

impl Component for ReduceHistoryView {
    type Message = ReduceHistoryMsg;
    type Properties = ReduceHistoryProps;
    fn create(_ctx: &Context<Self>) -> Self {
        Self {}
    }
    fn view(&self, ctx: &Context<Self>) -> Html {
        let ReduceHistoryProps { vec } = ctx.props();
        html! {
            <>
            {for vec.iter().map(|term| html!{<> {term.to_string()} <br/> </>}) }
            </>
        }
    }
    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {}
    }
}

pub struct LambdaCalculusView {
    term: Option<LambdaTerm>,
    history: Vec<LambdaTerm>,
}

#[derive(Debug, Clone, PartialEq, Properties, Default)]
pub struct LambdaCalculusProps {}

pub enum LambdaCalculusMsg {
    Change(LambdaTerm),
    Step,
}

impl Component for LambdaCalculusView {
    type Message = LambdaCalculusMsg;
    type Properties = LambdaCalculusProps;
    fn create(_ctx: &Context<Self>) -> Self {
        Self {
            term: None,
            history: vec![],
        }
    }
    fn view(&self, ctx: &Context<Self>) -> Html {
        let callback = ctx.link().callback(LambdaCalculusMsg::Change);
        let term_html = html! {<>{
            if let Some(ref term) = self.term {
                html!{ <> {term.to_string()} {"is_normal:"} {is_normal(term)} </> }
            } else {
                html!{ <> {"no term found"} </>}
            }
        }</>};
        html! {
            <>
                <InputParseOnelineView input_term_call={callback}/> <br/>
                {term_html} <br/>
                <button onclick={ctx.link().callback(|_| LambdaCalculusMsg::Step)}> {"step"} </button>  <br/>
                <ReduceHistoryView vec={self.history.clone()} />
            </>
        }
    }
    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            LambdaCalculusMsg::Change(term) => {
                self.term = Some(term);
                self.history = vec![];
            }
            LambdaCalculusMsg::Step => {
                if let Some(ref term) = self.term {
                    self.history.push(term.clone());
                    if !is_normal(term) {
                        self.term = Some(left_most_reduction(term.clone()))
                    };
                }
            }
        }
        true
    }
}
