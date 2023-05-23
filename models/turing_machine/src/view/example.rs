use super::machine::*;
use crate::example::*;
use crate::machine::TapeAsVec;
use web_sys::HtmlInputElement;
use yew::html::Scope;
use yew::prelude::*;

#[derive(Debug, Clone)]
enum ExampleInputState {
    WaitInput,
    Input(Option<(BinInt, BinInt)>),
}

impl ToString for ExampleInputState {
    fn to_string(&self) -> String {
        match self {
            ExampleInputState::WaitInput => "wait input".to_string(),
            ExampleInputState::Input(Some((u1, u2))) => {
                format!("input: ({}, {})", String::from(u1), String::from(u2))
            }
            ExampleInputState::Input(None) => {
                "parse error".to_string()
            }
        }
    }
}

#[derive(Debug, Clone)]
enum ExampleResultState {
    NoResult,
    Result(Result<String, String>),
}

impl ToString for ExampleResultState {
    fn to_string(&self) -> String {
        match self {
            ExampleResultState::NoResult => "wait output".to_string(),
            ExampleResultState::Result(Ok(ok)) => {
                format!("output: {ok}")
            }
            ExampleResultState::Result(Err(err)) => {
                format!("error: {err}")
            }
        }
    }
}

#[derive(Debug)]
pub struct ExampleView {
    scope: Option<Scope<TuringMachineView>>,
    now_input_state: ExampleInputState,
    now_output_state: ExampleResultState,
}
pub enum ExampleMsg {
    SetTargetMachineView(Scope<TuringMachineView>),
    // SendIncMachine,
    SendBinAdderMachine,
    ChangeInput(String),
    Result(Result<String, String>),
}

#[derive(Debug, Default, Clone, PartialEq, Properties)]
pub struct ExampleProps {}

impl Component for ExampleView {
    type Message = ExampleMsg;
    type Properties = ExampleProps;
    fn create(_ctx: &Context<Self>) -> Self {
        Self {
            scope: None,
            now_input_state: ExampleInputState::WaitInput,
            now_output_state: ExampleResultState::NoResult,
        }
    }
    fn view(&self, ctx: &Context<Self>) -> Html {
        let oninput_callback = ctx.link().callback(|e: Event| {
            let value: HtmlInputElement = e.target_unchecked_into();
            let str = value.value();
            ExampleMsg::ChangeInput(str)
        });
        html! {
            <>
                {"example"} <br/>
                <br/>
                <>
                    <button onclick={ctx.link().callback(|_| ExampleMsg::SendBinAdderMachine)}> { "inc inc" } </button>
                    {"input"} <input type="text" onchange={oninput_callback}/> {self.now_input_state.clone()} <br/>
                    {"output"} {self.now_output_state.clone()}
                </>
            </>
        }
    }
    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            ExampleMsg::SetTargetMachineView(scope) => {
                self.scope = Some(scope);
            }
            ExampleMsg::ChangeInput(str) => {
                self.now_input_state = ExampleInputState::Input(str_to_two_bin(&str));
            }
            ExampleMsg::Result(result) => {
                self.now_output_state = ExampleResultState::Result(result);
            }
            ExampleMsg::SendBinAdderMachine => {
                if let Some(scope) = &self.scope {
                    let mut builder = bin_adder_str();
                    let u = match &self.now_input_state {
                        ExampleInputState::Input(Some(num)) => num,
                        _ => {
                            return true;
                        }
                    };
                    builder.input(two_bin_to_str(u));
                    let callback = ctx.link().callback(|tape: TapeAsVec| {
                        let i = BinInt::interpretation_str();
                        ExampleMsg::Result(i.read()(tape))
                    });
                    scope.send_message(TuringMachineMsg::LoadFromMachine(builder.build().unwrap()));
                    scope.send_message(TuringMachineMsg::SetMachineOnTerminate(callback));
                }
            }
        }
        true
    }
}
