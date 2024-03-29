use turing_machine_view::{
    control::{ControlMsg, ControlView},
    // example::{ExampleView, ExampleMsg}
    machine::TuringMachineView,
};
// use turing_machine::example;

fn main() {
    let document = gloo::utils::document();
    let machine_element = document.query_selector("#machine").unwrap().unwrap();
    let machine_handle = yew::Renderer::<TuringMachineView>::with_root(machine_element).render();

    let control_element = document.query_selector("#control").unwrap().unwrap();
    let control_handle = yew::Renderer::<ControlView>::with_root(control_element).render();
    control_handle.send_message(ControlMsg::SetTargetMachineView(machine_handle.clone()));

    // let builder = example::bin_adder();
    // machine_handle.send_message(TuringMachineMsg::LoadFromMachine(builder.build().unwrap()));

    // let example_element = document.query_selector("#example").unwrap().unwrap();
    // let example_handle = yew::Renderer::<ExampleView>::with_root(example_element).render();
    // example_handle.send_message(ExampleMsg::SetTargetMachineView(machine_handle.clone()));
}
