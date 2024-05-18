use turing_machine::{
    machine::*,
    manipulation::{builder::TuringMachineBuilder, tape::string_split_by_bar_interpretation, *},
};

fn bin_adder(str: &str) -> TuringMachineBuilder {
    let interpretation = string_split_by_bar_interpretation();
    let code = code::parse_code(include_str!("turing_machine_bin_adder.txt")).unwrap();
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
    let target_element = document
        .get_element_by_id("turing_machine_example")
        .unwrap();

    let machine = bin_adder(" - 1 1 0 0 1 |-|").build().unwrap();
    turing_machine_view::set_machine(target_element, machine);
}
