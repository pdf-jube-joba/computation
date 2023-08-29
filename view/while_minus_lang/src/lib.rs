use while_minus_lang::machine::{
    Environment, FlatWhileLanguage, FlatWhileStatement, ProgramProcess,
};
use yew::prelude::*;
use yew::Properties;

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
