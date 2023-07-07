use turing_machine::{machine::*, view::machine::*, manipulation::*};
use yew::prelude::*;
use gloo::timers::callback::Interval;

fn bin_adder() -> TuringMachineSet {
    let code =  code::parse_code(include_str!("bin_adder.txt")).unwrap();
    let tape_input = "1 1 0 0 1 - |-|".to_string();
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
    let target_element = document.get_element_by_id("example_bin_adder").unwrap();
    let handle = yew::Renderer::<TuringMachineView>::with_root_and_props(target_element, TuringMachineProp { code_visible: false}).render();
    handle.send_message(TuringMachineMsg::LoadFromMachine(bin_adder()));
}
