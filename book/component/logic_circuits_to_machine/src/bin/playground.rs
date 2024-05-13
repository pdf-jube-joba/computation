use logic_circuit_view::{control, machine};
use logic_circuits_to_machine::*;

fn main() {
    let document = gloo::utils::document();
    let element = document
        .get_element_by_id("playground")
        .unwrap();

    let control_element = document.create_element("div").unwrap();
    element.append_child(&control_element).unwrap();

    let control_handle = yew::Renderer::<control::ControlView>::with_root(control_element).render();

    let machine_element = document.create_element("div").unwrap();
    element.append_child(&machine_element).unwrap();
    let machine_handle = yew::Renderer::<machine::MachineView>::with_root(machine_element).render();

    eprintln!("hello world");

    control_handle.send_message(control::ControlMsg::SetTargetMachineView(
        machine_handle.clone(),
    ));
}
