use recursive_function_to_turing_machine::compile::*;
use turing_machine_view::machine::*;
use yew::Callback;

fn main() {
    let document = gloo::utils::document();
    let element = document
        .get_element_by_id("recursive_function_to_turing_machine_playground")
        .unwrap();

    let machine_element = document.create_element("div").unwrap();
    element.append_child(&machine_element).unwrap();
    let machine_handle = yew::Renderer::<TuringMachineView>::with_root(machine_element).render();

    let control_element = document.create_element("div").unwrap();
    element.append_child(&control_element).unwrap();
    let _control_handle = yew::Renderer::<recursive_function_view::CodeView>::with_root_and_props(
        control_element,
        recursive_function_view::CodeProps {
            on_input_code: Callback::from(move |func| {
                let machine = compile(&func).build().unwrap();
                machine_handle.send_message(TuringMachineMsg::LoadFromMachine(machine));
            }),
        },
    )
    .render();
}
