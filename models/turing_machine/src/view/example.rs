use yew::html::Scope;
use yew::prelude::*;
use super::machine::*;
use web_sys::HtmlInputElement;
// use crate::manipulation::*;
use crate::manipulation::compose_builder;
use crate::example::*;
use crate::machine::{State, TapeAsVec};

#[derive(Debug, Clone)]
enum ExampleInputState {
    WaitInput,
    Input(Result<usize, String>),
}


impl ToString for ExampleInputState {
    fn to_string(&self) -> String {
        match self {
            ExampleInputState::WaitInput => {
                "wait input".to_string()
            }
            ExampleInputState::Input(Ok(ok)) => {
                format!("input: {ok}")
            }
            ExampleInputState::Input(Err(err)) => {
                format!("error: {err}")
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
            ExampleResultState::NoResult => {
                "wait output".to_string()
            }
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
    SendIncIncMachine,
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
        let oninput_callback = ctx.link().callback(|e: Event|{
            let value: HtmlInputElement = e.target_unchecked_into();
            let str = value.value();
            ExampleMsg::ChangeInput(str)
        });
        html!{
            <>
                {"example"} <br/>
                <br/>
                <>
                    <button onclick={ctx.link().callback(|_| ExampleMsg::SendIncIncMachine)}> { "inc inc" } </button>
                    {"input"} <input type="text" onchange={oninput_callback}/> {self.now_input_state.clone()} <br/>
                    {"output"} {self.now_output_state.clone()}
                </>
                // <button onclick={ctx.link()}> { "zero" } </button>
            </>
        }
    }
    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            ExampleMsg::SetTargetMachineView(scope) => {
                self.scope = Some(scope);
            }
            ExampleMsg::ChangeInput(str) => {
                let i = str.parse::<usize>().map_err(|_| "parse error".to_string());
                // panic!("");
                self.now_input_state = ExampleInputState::Input(i);
            }
            ExampleMsg::Result(result) => {
                self.now_output_state = ExampleResultState::Result(result);
            }
            ExampleMsg::SendIncIncMachine => {
                if let Some(scope) = &self.scope {
                    let mut builder = compose_builder(inc(), State::try_from("end").unwrap(), inc()).unwrap();
                    let i = match self.now_input_state {
                        ExampleInputState::Input(Ok(num)) => {
                            num
                        }
                        _ => {
                            return true;
                        }
                    };
                    builder.input(format!("({i})"));
                    let callback = ctx.link().callback(|tape: TapeAsVec| {
                        let i = crate::example::NatNumInterpretation::interpretation();
                        ExampleMsg::Result(i.read()(tape))
                    });
                    scope.send_message(TuringMachineMsg::LoadFromMachine(builder.build().unwrap()));
                    scope.send_message(TuringMachineMsg::SetMachineOnTerminate(callback));
                    // scope.send_message(TuringMachineMsg::)
                }
            }
        }
        true
    }
}