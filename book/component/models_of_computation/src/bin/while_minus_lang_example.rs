use while_minus_lang::machine::*;
use while_minus_lang_view::*;

fn main() {
    let document = gloo::utils::document();
    let element = document
        .get_element_by_id("while_minus_lang_example")
        .unwrap();

    let prog: WhileLanguage = vec![
        WhileStatement::inc(0.into()),
        WhileStatement::inc(0.into()),
        WhileStatement::inc(0.into()),
        WhileStatement::inc(0.into()),
        WhileStatement::inc(0.into()),
        WhileStatement::while_not_zero(
            0.into(),
            vec![WhileStatement::dec(0.into()), WhileStatement::inc(1.into())],
        ),
    ]
    .into();
    let env: Environment = Environment::new();

    let _handle = yew::Renderer::<UnConnectedMachineView>::with_root_and_props(
        element,
        UnConnectedMachineProp {
            init_prog: (&prog).into(),
            init_env: env,
        },
    );
}
