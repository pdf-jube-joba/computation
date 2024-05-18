use turing_machine::machine::TuringMachineSet;
use web_sys::Element;

pub mod control;
pub mod machine;

pub fn set_machine(element: Element, machine: TuringMachineSet) {
    let machine_handle = yew::Renderer::<machine::TuringMachineView>::with_root(element).render();
    machine_handle.send_message(machine::TuringMachineMsg::LoadFromMachine(Box::new(
        machine,
    )));
}
