use turing_machine::view;
use turing_machine::control;
use turing_machine::example;

fn main() {
    let document = gloo::utils::document();
    let machine_element = document.query_selector("#machine").unwrap().unwrap();
    let machine_handle = yew::Renderer::<view::TuringMachineView>::with_root(machine_element).render();

    let inc_5 = example::inc_example(5);
    machine_handle.send_message(view::TuringMachineMsg::LoadFromMachine(inc_5.build().unwrap()));

    // let control_element = document.query_selector("#control").unwrap().unwrap();
    // let control_handle = yew::Renderer::<control::ControlView>::with_root(control_element).render();
    // control_handle.send_message(control::ControlMsg::SetTargetMachineView(machine_handle.clone()));

}