use yew::prelude::*;
use web_sys::HtmlInputElement;
use lambda_calculus::{machine::*, manipulation};

#[derive(Debug, Clone, PartialEq, Properties)]
struct VarBoxProps {
    var: Var,
}

#[function_component(VarBox)]
fn var_box_view(VarBoxProps { var }: &VarBoxProps) -> Html {
    html!{
        var.to_string()
    }
}

pub struct InputParseOnelineView {
    parse_result: Result<LambdaTerm, ()>,
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
        InputParseOnelineView { 
            parse_result: Err(())
        }
    }
    fn view(&self, ctx: &Context<Self>) -> Html {
        let onchange_input = ctx.link().callback(|e: Event| {
            let value: HtmlInputElement = e.target_unchecked_into();
            let str = value.value();
            InputParseOnelineMsg::Change(str)
        });
        let parse_result = if let Ok(term) = &self.parse_result {
            term.to_string()
        } else {
            "no term found".to_owned()
        };
        html!{
            <>
            <input onchange={onchange_input}/> {"result"} {parse_result} <br/>
            <button oninput={ctx.link().callback(|_| InputParseOnelineMsg::Ok)}> {"ok"} </button>
            </>
        }
    }
    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            InputParseOnelineMsg::Change(str) => {
                self.parse_result = manipulation::parse::parse_lambda(&str);
                true
            },
            InputParseOnelineMsg::Ok => {
                if let Ok(term) = &self.parse_result {
                    let InputParseOnelineProps { input_term_call } = ctx.props();
                    input_term_call.emit(term.clone());
                }
                false
            }
        }
    }
}

struct ReduceHistoryView {
    vec: Vec<LambdaTerm>,
}

#[derive(Debug, Clone, PartialEq, Properties)]
struct ReduceHistoryProps {
}

enum ReduceHistoryMsg {
    Add(LambdaTerm),
}

impl Component for ReduceHistoryView {
    type Message = ReduceHistoryMsg;
    type Properties = ReduceHistoryProps;
    fn create(ctx: &Context<Self>) -> Self {
        Self { vec: vec![] }
    }
    fn view(&self, ctx: &Context<Self>) -> Html {
        html!{
            <>
            {for self.vec.iter().map(|term| html!{<> {term.to_string()} <br/> </>}) }
            </>
        }
    }
    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            ReduceHistoryMsg::Add(term) => {
                self.vec.push(term);
                true
            }
        }
    }
}
