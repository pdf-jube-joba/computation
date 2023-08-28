use while_minus_lang::machine::{
    Var,
    Environment,
    FlatWhileLanguage,
    FlatWhileStatement,
};
use while_minus_lang_view::{UnConnectedMachineView, UnConnectedMachineProp};

fn main() {
    let document = gloo::utils::document();
    let machine_element = document.query_selector("#machine").unwrap().unwrap();

    let prog: FlatWhileLanguage = vec![
            FlatWhileStatement::inc(Var::from(0)),
            FlatWhileStatement::inc(Var::from(0)),
            FlatWhileStatement::inc(Var::from(0)),
            FlatWhileStatement::inc(Var::from(0)),
            FlatWhileStatement::inc(Var::from(0)),
            FlatWhileStatement::while_not_zero(Var::from(0)),
            FlatWhileStatement::dec(Var::from(0)),
            FlatWhileStatement::inc(Var::from(1)),
            FlatWhileStatement::while_end(),
        ]
        .into();
    let env = Environment::new();
    let _machine_handle = yew::Renderer::<UnConnectedMachineView>::with_root_and_props(
        machine_element,
        UnConnectedMachineProp {
            init_prog: prog,
            init_env: env
        }
    ).render();
}
