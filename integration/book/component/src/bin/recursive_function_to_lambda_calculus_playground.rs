use recursive_function_view::*;
use yew::{Properties, Component, Callback};
use lambda_calculus_view::*;
use recursive_function_to_lambda_calculus::compile;

fn main() {
    let document = gloo::utils::document();
    let element = document.get_element_by_id("recursive_function_to_lambda_calculus_playground").unwrap();

    let machine_element = document.create_element("div").unwrap();
    element.append_child(&machine_element).unwrap();
    let machine_handle = yew::Renderer::<LambdaCalculusView>::with_root(machine_element).render();

    let control_element = document.create_element("div").unwrap();
    element.append_child(&control_element).unwrap();
    let _control_handle = yew::Renderer::<CodeView>::with_root_and_props(
        control_element,
        CodeProps {
            on_input_code: Callback::from(move |func| {
                let term = compile(&func);
                machine_handle.send_message(LambdaCalculusMsg::Change(term));
            })
        }
    ).render();
}