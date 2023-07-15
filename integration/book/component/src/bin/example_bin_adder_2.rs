use turing_machine::{machine::*, manipulation::*};
use turing_machine_view::{machine::*};

fn bin_adder(str: &str) -> TuringMachineSet {
    let code =  code::parse_code(include_str!("bin_adder.txt")).unwrap();
    let tape_input = str.to_string();
    let mut builder = builder::TuringMachineBuilder::new("bin_adder", tape::string_split_by_bar_interpretation()).unwrap();
    builder
        .code_new(code)
        .init_state(State::try_from("start").unwrap())
        .accepted_state(vec![State::try_from("end").unwrap()])
        .input(tape_input);
    builder.build().unwrap()
}

fn main() {
    let document = gloo::utils::document();
    let target_element = document.get_element_by_id("example_bin_adder_2").unwrap();

    let element_1 = document.create_element("div").unwrap();
    target_element.append_child(&element_1).unwrap();
    let handle_1 = yew::Renderer::<TuringMachineView>::with_root_and_props(element_1, TuringMachineProp { code_visible: false}).render();
    handle_1.send_message(TuringMachineMsg::LoadFromMachine(bin_adder(" - 1 1 0 0 1 |-|")));

    let element_2 = document.create_element("div").unwrap();
    target_element.append_child(&element_2).unwrap();
    let handle_2 = yew::Renderer::<TuringMachineView>::with_root_and_props(element_2, TuringMachineProp { code_visible: false}).render();
    handle_2.send_message(TuringMachineMsg::LoadFromMachine(bin_adder(" - 1 1 |-|")));
}
