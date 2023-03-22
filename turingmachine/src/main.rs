mod machine;
use machine::*;
mod control;
use control::*;

fn main() {
    let document = gloo::utils::document();
    let machine_element = document.query_selector("#machine").unwrap().unwrap();
    let machine_handle = yew::Renderer::<TuringMachineView>::with_root(machine_element).render();

    let control_element = document.query_selector("#control").unwrap().unwrap();
    let control_handle = yew::Renderer::<ControlView>::with_root(control_element).render();
    control_handle.send_message(ControlMsg::SetTargetMachineView(machine_handle.clone()));
}