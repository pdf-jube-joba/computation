use recursive_function_to_while_minus_lang::compile;
use while_minus_lang::machine::Environment;
use while_minus_lang_view::*;
use yew::Callback;

fn main() {
    let document = gloo::utils::document();
    let element = document
        .get_element_by_id("recursive_function_to_while_minus_lang")
        .unwrap();

    let machine_element = document.create_element("div").unwrap();
    element.append_child(&machine_element).unwrap();
    let machine_handle = yew::Renderer::<WhileLangView>::with_root_and_props(machine_element, WhileLangProps {}).render();

    let control_element = document.create_element("div").unwrap();
    element.append_child(&control_element).unwrap();
    let _control_handle = yew::Renderer::<recursive_function_view::CodeView>::with_root_and_props(
        control_element,
        recursive_function_view::CodeProps {
            on_input_code: Callback::from(move |func| {
                let prog = compile(&func);
                machine_handle.send_message(WhileLangMsg::Change(prog, Environment::new()))
            }),
        },
    )
    .render();
}
