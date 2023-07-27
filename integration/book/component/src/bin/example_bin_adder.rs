use turing_machine::{machine::*, manipulation::{*, tape::string_split_by_bar_interpretation, builder::TuringMachineBuilder}};
use turing_machine_view::{machine::*};

fn bin_adder(str: &str) -> TuringMachineBuilder {
    let interpretation = string_split_by_bar_interpretation();
    let code =  code::parse_code(include_str!("bin_adder.txt")).unwrap();
    let tape_input = interpretation.write()(str.to_string()).unwrap();
    let mut builder = builder::TuringMachineBuilder::new("bin_adder").unwrap();
    builder
        .code_new(code)
        .init_state(State::try_from("start").unwrap())
        .accepted_state(vec![State::try_from("end").unwrap()])
        .input(tape_input);
    builder
}

fn main() {
    let document = gloo::utils::document();
    let target_element = document.get_element_by_id("example_bin_adder").unwrap();

    let element_1 = document.create_element("div").unwrap();
    target_element.append_child(&element_1).unwrap();
    let _ = yew::Renderer::<UnConnectedMachineView>::with_root_and_props(element_1, UnConnectedMachineProp { builder: bin_adder(" - 1 1 0 0 1 |-|")}).render();
}
