use web_sys::HtmlInputElement;
use while_minus_lang::machine::WhileLanguage;
use while_minus_lang::machine::{
    Environment, FlatWhileLanguage, FlatWhileStatement, ProgramProcess,
};
use yew::prelude::*;
use yew::{callback, html, Callback, Component, Properties};

#[derive(Debug, Clone, PartialEq, Properties)]
pub struct FlatWhileStatementProps {
    statement: FlatWhileStatement,
}

#[function_component(FlatWhileStatementView)]
fn flat_while_statement_view(
    FlatWhileStatementProps { statement }: &FlatWhileStatementProps,
) -> Html {
    html! {
        <>
        {statement.to_string()}
        </>
    }
}

#[derive(Debug, Clone, PartialEq, Properties)]
pub struct FlatWhileLanguageProps {
    prog: FlatWhileLanguage,
}

#[function_component(FlatWhileLanguageView)]
fn flat_while_lang_view(FlatWhileLanguageProps { prog }: &FlatWhileLanguageProps) -> Html {
    html! {
        <>
        { for prog.to_vec().into_iter().map(|statement| html!{
            <> <FlatWhileStatementView statement={statement}/> <br/> </>
        })}
        </>
    }
}

#[derive(Debug, Clone, PartialEq, Properties)]
pub struct EnvironmentProps {
    env: Environment,
}

#[function_component(EnvironmentView)]
fn environment_view(EnvironmentProps { env }: &EnvironmentProps) -> Html {
    let all_var = env.all_written_var();
    let env_html: Vec<Html> = all_var.into_iter().map(|var| {
        let value = env.get(&var);
        html!{ <> <tr> <td> {format!("{var:?}")} </td> <td> {format!("{value:?}")} </td> </tr> </> }
    }).collect();
    html! {
        <>
            <table>
                <thead>
                    <tr> <td> {"var"} </td> <td> {"value"} </td> </tr>
                </thead>
                <tbody>
                    {for env_html}
                </tbody>
            </table>
        </>
    }
}

#[derive(Debug, Clone, PartialEq, Properties)]
pub struct ProgramProcessProps {
    program_process: ProgramProcess,
}

#[function_component(ProgramProcessView)]
fn flat_while_lang_process_view(props: &ProgramProcessProps) -> Html {
    let ProgramProcessProps { program_process } = props;
    let vec_html: Vec<Html> = program_process.code().to_vec().into_iter().enumerate().map(|(index, statement)|{
        if index == program_process.now_index() {
            html!{<> <div class={"selected"}> <FlatWhileStatementView statement={statement} /> </div> </>}
        } else {
            html!{<> <div class={"unselected"}> <FlatWhileStatementView statement={statement} /> </div> </>}
        }
    }).collect();
    html! {
        <>
            <div class={"prog"}>
                {for vec_html}
            </div>
            <div class={"env"}>
                <EnvironmentView env={program_process.env()} />
            </div>
        </>
    }
}

#[derive(Debug, Clone, PartialEq, Properties)]
struct ControlProps {
    on_reset: Callback<()>,
    on_step: Callback<usize>,
}

#[function_component(ControlView)]
fn control_view(props: &ControlProps) -> Html {
    let ControlProps { on_reset, on_step } = props;
    let on_step = {
        let on_step = on_step.clone();
        let on_step = move |_| on_step.emit(1);
        on_step
    };
    let on_reset = {
        let on_reset = on_reset.clone();
        let on_reset = move |_| on_reset.emit(());
        on_reset
    };
    html! {
        <>
            <button onclick={on_step}> {"step"} </button>
            <button onclick={on_reset}> {"reset"} </button>
        </>
    }
}

#[derive(Debug, Clone, PartialEq, Properties)]
pub struct MachineProps {
    on_reset: Callback<()>,
    on_step: Callback<usize>,
    prog: ProgramProcess,
}

#[function_component(MachineView)]
fn machine_view(props: &MachineProps) -> Html {
    let MachineProps {
        on_reset,
        on_step,
        prog,
    } = props;
    html! {
        <>
            <ControlView on_reset={on_reset} on_step={on_step}/>
            <ProgramProcessView program_process={prog.clone()}/>
        </>
    }
}

pub struct UnConnectedMachineView {
    prog: ProgramProcess,
}

#[derive(Debug, Clone, PartialEq, Properties)]
pub struct UnConnectedMachineProp {
    pub init_prog: FlatWhileLanguage,
    pub init_env: Environment,
}

#[derive(Clone)]
pub enum UnConnectedMachineMsg {
    Step(usize),
    Reset,
}

impl Component for UnConnectedMachineView {
    type Message = UnConnectedMachineMsg;
    type Properties = UnConnectedMachineProp;
    fn create(ctx: &Context<Self>) -> Self {
        let UnConnectedMachineProp {
            init_prog,
            init_env,
        } = ctx.props();
        let prog = ProgramProcess::new(init_prog.clone(), init_env.clone());
        Self { prog }
    }
    fn view(&self, ctx: &Context<Self>) -> Html {
        let on_step = ctx
            .link()
            .callback(|step| UnConnectedMachineMsg::Step(step));
        let on_reset = ctx.link().callback(|_| UnConnectedMachineMsg::Reset);
        html! {
            <>
                <MachineView on_step={on_step} on_reset={on_reset} prog={self.prog.clone()}/>
            </>
        }
    }
    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        let UnConnectedMachineProp {
            init_prog,
            init_env,
        } = ctx.props();
        match msg {
            UnConnectedMachineMsg::Reset => {
                self.prog = ProgramProcess::new(init_prog.clone(), init_env.clone());
                true
            }
            UnConnectedMachineMsg::Step(_) => {
                self.prog.step();
                true
            }
        }
    }
}

pub struct WhileLangView {
    prog: Option<ProgramProcess>,
}

#[derive(Debug, Clone, PartialEq, Properties)]
pub struct WhileLangProps {}

#[derive(Clone)]
pub enum WhileLangMsg {
    Change(WhileLanguage, Environment),
    Step(usize),
}

impl Component for WhileLangView {
    type Message = WhileLangMsg;
    type Properties = WhileLangProps;
    fn create(_ctx: &Context<Self>) -> Self {
        Self { prog: None }
    }
    fn view(&self, ctx: &Context<Self>) -> Html {
        let on_step = ctx.link().callback(|step| WhileLangMsg::Step(step));
        let on_reset = yew::callback::Callback::noop();
        let html = if let Some(ref prog) = self.prog {
            html! {
                <MachineView on_reset={on_reset} on_step={on_step} prog={prog.clone()}/>
            }
        } else {
            html! {"none"}
        };
        html! {
            {html}
        }
    }
    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            WhileLangMsg::Step(_) => {
                self.prog.as_mut().map(|process| process.step());
                true
            }
            WhileLangMsg::Change(prog, env) => {
                self.prog = Some(ProgramProcess::new((&prog).into(), env));
                true
            }
        }
    }
}

pub struct CodeView {
    code: Result<WhileLanguage, ()>,
}

#[derive(Debug, Clone, PartialEq, Properties)]
pub struct CodeProps {
    pub on_input_code: Callback<WhileLanguage>,
}

pub enum CodeMsg {
    Change(String),
}

impl Component for CodeView {
    type Message = CodeMsg;
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
            callback::Callback::from(move |_| callback.emit(function.clone()))
        } else {
            callback::Callback::noop()
        };
        html! {
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
                self.code = while_minus_lang::manipulation::parse(&string);
                true
            }
        }
    }
}

#[derive(Debug, Default)]
pub struct WhileLangControlView {
    prog: Option<WhileLanguage>,
}

#[derive(Debug, Clone, PartialEq, Properties)]
pub struct WhileLangControlProps {}

pub enum WhileLangControlMsg {
    SetFunction(WhileLanguage),
}

impl Component for WhileLangControlView {
    type Message = WhileLangControlMsg;
    type Properties = WhileLangControlProps;
    fn create(_ctx: &Context<Self>) -> Self {
        Self::default()
    }
    fn view(&self, ctx: &Context<Self>) -> Html {
        let callback: Callback<WhileLanguage> = ctx
            .link()
            .callback(|func| WhileLangControlMsg::SetFunction(func));
        let html = if let Some(prog) = self.prog.clone() {
            let flat_prog: FlatWhileLanguage = (&prog).into();
            html! {
                <UnConnectedMachineView init_prog={flat_prog} init_env={Environment::new()} />
            }
        } else {
            html! { "none" }
        };
        html! {
            <>
            <CodeView on_input_code={callback} /> <br/>
            {html}
            </>
        }
    }
}
