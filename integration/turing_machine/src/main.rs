use turing_machine::machine::view::*;
use turing_machine::machine::manipulation;
use turing_machine::control::*;
// mod machine;
// use machine::*;
// mod control;
// use control::*;

fn main() {
    let document = gloo::utils::document();
    let machine_element = document.query_selector("#machine").unwrap().unwrap();
    let machine_handle = yew::Renderer::<TuringMachineView>::with_root(machine_element).render();

    let inc_5 = manipulation::example::inc_composition_example(5);
    machine_handle.send_message(TuringMachineMsg::LoadFromBuilder(inc_5));

    let control_element = document.query_selector("#control").unwrap().unwrap();
    let control_handle = yew::Renderer::<ControlView>::with_root(control_element).render();
    control_handle.send_message(ControlMsg::SetTargetMachineView(machine_handle.clone()));
}