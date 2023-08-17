use turing_machine_view::{
    control::{ControlMsg, ControlView},
    machine::TuringMachineView,
};

fn main() {
    let document = gloo::utils::document();
    let element = document.get_element_by_id("turing_machine_playground").unwrap();

    let control_element = document.create_element("div").unwrap();
    element.append_child(&control_element).unwrap();
    let control_handle = yew::Renderer::<ControlView>::with_root(control_element).render();

    let machine_element = document.create_element("div").unwrap();
    element.append_child(&machine_element).unwrap();
    let machine_handle = yew::Renderer::<TuringMachineView>::with_root(machine_element).render();

    control_handle.send_message(ControlMsg::SetTargetMachineView(machine_handle.clone()));
}
