use recursive_function::machine::{RecursiveFunctions, NumberTuple};
use wasm_bindgen::JsValue;
use yew::{Properties, Component, html, Callback, callback};
use yew::prelude::*;
use web_sys::HtmlInputElement;

pub struct CodeView {
    code: Result<RecursiveFunctions, ()>,
}

#[derive(Debug, Clone, PartialEq, Properties)]
pub struct CodeProps {
    on_input_code: Callback<RecursiveFunctions>,
}

pub enum CodeMsg {
    Change(String),
}

impl Component for CodeView {
    type Message = CodeMsg ;
    type Properties = CodeProps;
    fn create(_ctx: &yew::Context<Self>) -> Self {
        Self { code: Err(()) }
    }
    fn view(&self, ctx: &yew::Context<Self>) -> yew::Html {
        let CodeProps { on_input_code } = ctx.props().clone();
        let onchange_input = ctx.link().callback(|e: Event| {
            let value: HtmlInputElement = e.target_unchecked_into();
            let str = value.value();
            CodeMsg::Change(str)
        });
        let callback = on_input_code.clone();
        let onclick = if let Ok(function) = self.code.clone() {
            callback::Callback::from(move |_| {callback.emit(function.clone())})
        } else {
            callback::Callback::noop()
        };
        html!{
            <>
            {"code"}
            <textarea rows={3} onchange={onchange_input}/>
            <button onclick={onclick}> {"load"}</button>
            </>
        }
    }
    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            CodeMsg::Change(string) => {
                self.code = recursive_function::manipulation::parse(&string);
                true
            }
        }
    }
}

pub struct FunctionView {
    input: Result<NumberTuple, String>,
    output: Result<NumberTuple, ()>,
}

#[derive(Debug, Clone, PartialEq, Properties)]
pub struct FunctionProps {
    rec_function: RecursiveFunctions,
}

pub enum FunctionMsg {
    InputChange(String),
    Compute,
    ComputeEnd(Result<NumberTuple, ()>),
}

impl Component for FunctionView {
    type Message = FunctionMsg;
    type Properties = FunctionProps;
    fn create(_ctx: &Context<Self>) -> Self {
        Self { input: Err("no".into()), output: Err(())}
    }
    fn view(&self, ctx: &Context<Self>) -> Html {
        let FunctionProps { rec_function } = ctx.props();
        let input_function = ctx.link().callback(|e: InputEvent| {
            let value: HtmlInputElement = e.target_unchecked_into();
            let str = value.value();
            FunctionMsg::InputChange(str)
        });
        html!{
            <>
                <input oninput={input_function}/> {format!("{:?}", self.input)}
                <div>
                {"function"} <br/>
                {rec_function.clone()}
                </div>
                <button onclick={ctx.link().callback(|_| FunctionMsg::Compute)}> {"compute"} </button>
                {
                    if let Ok(function) = self.output.clone() {format!("{function:?}")} else {
                        "none".to_string()
                    }
                }
            </>
        }
    }
    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            FunctionMsg::InputChange(string) => {
                self.input = NumberTuple::try_from(string.clone());
                true
            }
            FunctionMsg::Compute => {
                if let Ok(input) = self.input.clone() {
                    web_sys::console::log_1(&JsValue::from_str("hello"));
                    let FunctionProps { rec_function } = ctx.props();
                    let function = recursive_function::machine::interpreter(&rec_function);
                    let res: Result<NumberTuple, _> = function.checked_subst(input).map(|num| num.into());
                    self.output = res;
                    true    
                } else {
                    false
                }
            }
            FunctionMsg::ComputeEnd(result) => {
                self.output = result;
                true
            }
        }
    }
}

#[derive(Debug, Default)]
pub struct FunctionControlView {
    function: Option<RecursiveFunctions>,
}

#[derive(Debug, Clone, PartialEq, Properties)]
pub struct FunctionControlProps {}

pub enum FunctionControlMsg {
    SetFunction(RecursiveFunctions),
}

impl Component for FunctionControlView {
    type Message = FunctionControlMsg;
    type Properties = FunctionControlProps;
    fn create(_ctx: &Context<Self>) -> Self {
        Self { function: None }
    }
    fn view(&self, ctx: &Context<Self>) -> Html {
        let callback: Callback<RecursiveFunctions> = ctx.link().callback(|func| FunctionControlMsg::SetFunction(func));
        html!{
            <>
            <CodeView on_input_code={callback} /> <br/>
            { 
                if let Some(function) = self.function.clone() {
                    html!{<FunctionView rec_function={function}/>}
                } else {
                    html!{{"no function found"}}
                }
            }
            </>
        }
    }
    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            FunctionControlMsg::SetFunction(function) => {
                self.function = Some(function);
                true
            }
        }
    }
}
