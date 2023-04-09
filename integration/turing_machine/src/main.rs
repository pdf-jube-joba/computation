use turing_machine::view;
use turing_machine::control;
use turing_machine::example;

fn main() {
    let document = gloo::utils::document();
    let machine_element = document.query_selector("#machine").unwrap().unwrap();
    let machine_handle = yew::Renderer::<view::TuringMachineView>::with_root(machine_element).render();

    let builder = example::inc_composition_example(5);
    machine_handle.send_message(view::TuringMachineMsg::LoadFromMachine(builder.build().unwrap()));

    let control_element = document.query_selector("#control").unwrap().unwrap();
    let control_handle = yew::Renderer::<control::ControlView>::with_root(control_element).render();
    control_handle.send_message(control::ControlMsg::SetTargetMachineView(machine_handle.clone()));

    let example_element = document.query_selector("#example").unwrap().unwrap();
    let example_handle = yew::Renderer::<example::view::ExampleView>::with_root(example_element).render();
    example_handle.send_message(example::view::ExampleMsg::SetTargetMachineView(machine_handle.clone()));

}