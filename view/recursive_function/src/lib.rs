use recursive_function::machine::RecursiveFunctions;
use recursive_function::manipulation;
use utils::{number::*, view::*};

use wasm_bindgen::JsValue;
use web_sys::{Element, HtmlInputElement};
use yew::prelude::*;
use yew::{callback, html, Callback, Component, Properties};

pub struct InputOutputView {
    input: Result<NumberTuple, String>,
    output: Option<Number>,
}

#[derive(Debug, Clone, PartialEq, Properties)]
pub struct InputOutputProps {
    rec_function: RecursiveFunctions,
}

pub enum InputOutputMsg {
    InputChange(String),
    Compute,
    ComputeEnd(Option<Number>),
}

impl Component for InputOutputView {
    type Message = InputOutputMsg;
    type Properties = InputOutputProps;
    fn create(_ctx: &Context<Self>) -> Self {
        Self {
            input: Err("no".into()),
            output: None,
        }
    }
    fn view(&self, ctx: &Context<Self>) -> Html {
        let input_function = ctx.link().callback(|e: InputEvent| {
            let value: HtmlInputElement = e.target_unchecked_into();
            let str = value.value();
            InputOutputMsg::InputChange(str)
        });
        html! {
            <>
                <input oninput={input_function}/> {format!("{:?}", self.input)}
                <button onclick={ctx.link().callback(|_| InputOutputMsg::Compute)}> {"compute"} </button>
                {
                    if let Some(result) = self.output.clone() {format!("{result:?}")} else {
                        "none".to_string()
                    }
                }
            </>
        }
    }
    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            InputOutputMsg::InputChange(string) => {
                self.input = NumberTuple::try_from(string.clone());
                true
            }
            InputOutputMsg::Compute => {
                if let Ok(input) = self.input.clone() {
                    web_sys::console::log_1(&JsValue::from_str("hello"));
                    let InputOutputProps { rec_function } = ctx.props();
                    let function = recursive_function::machine::interpreter(rec_function);
                    self.output = function.checked_subst(input);
                    true
                } else {
                    false
                }
            }
            InputOutputMsg::ComputeEnd(result) => {
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

#[derive(Debug, Default, Clone, PartialEq, Properties)]
pub struct FunctionControlProps {}

pub enum FunctionControlMsg {
    LoadFunction(RecursiveFunctions),
}

impl Component for FunctionControlView {
    type Message = FunctionControlMsg;
    type Properties = FunctionControlProps;
    fn create(_ctx: &Context<Self>) -> Self {
        Self { function: None }
    }
    fn view(&self, ctx: &Context<Self>) -> Html {
        html! {
            <>
            {
                if let Some(function) = &self.function {
                    html!{<>
                        <InputOutputView rec_function={function.clone()}/>
                        {function.clone()}
                    </>}
                } else {
                    html!{{"no function found"}}
                }
            }
            </>
        }
    }
    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            FunctionControlMsg::LoadFunction(function) => {
                self.function = Some(function);
                true
            }
        }
    }
}

fn playground(element: Element) {
    let machine_handle = yew::Renderer::<FunctionControlView>::with_root(element.clone()).render();
    let load_machine_callback =
        machine_handle.callback(|fnc| FunctionControlMsg::LoadFunction(fnc));

    let eventlog_handle = yew::Renderer::<EventLogView>::with_root(element.clone()).render();
    let event_log_callback = eventlog_handle.callback(|log| EventLogMsg::Log(log));

    let on_load = Callback::from(move |code: String| match manipulation::parse(&code) {
        Ok(fnc) => load_machine_callback.emit(fnc),
        Err(err) => event_log_callback.emit(err),
    });

    let code_handle = yew::Renderer::<utils::view::CodeView>::with_root_and_props(
        element,
        utils::view::CodeProps { on_load },
    )
    .render();
}

fn set_machine(element: Element, fnc: RecursiveFunctions) {
    let machine_handle = yew::Renderer::<FunctionControlView>::with_root(element).render();
    machine_handle.send_message(FunctionControlMsg::LoadFunction(fnc));
}
