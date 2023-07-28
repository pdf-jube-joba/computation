use turing_machine::{machine::*, manipulation::{*, tape::string_split_by_bar_interpretation, builder::TuringMachineBuilder}};
use turing_machine_view::machine::*;

fn zero_func(tape: TapeAsVec) -> TuringMachineBuilder {
    let code =  code::parse_code(include_str!("turing_machine_zero_func.txt")).unwrap();
    let mut builder = builder::TuringMachineBuilder::new("bin_adder").unwrap();
    builder
        .code_new(code)
        .init_state(State::try_from("start").unwrap())
        .accepted_state(vec![State::try_from("end").unwrap()])
        .input(tape);
    builder
}

fn main() {
    let interpretation = string_split_by_bar_interpretation();
    let tape_input = interpretation.write()("|-|-".to_string()).unwrap();

    let document = gloo::utils::document();
    let target_element = document.get_element_by_id("turing_machine_basic").unwrap();

    let element_1 = document.create_element("div").unwrap();
    target_element.append_child(&element_1).unwrap();
    let _ = yew::Renderer::<UnConnectedMachineView>::with_root_and_props(element_1, UnConnectedMachineProp { builder: zero_func(tape_input)}).render();
}

#[cfg(test)]
mod tests {
    use turing_machine::machine::Sign;
    use turing_machine::machine::TapeAsVec;

    use crate::zero_func;
    use crate::string_split_by_bar_interpretation;

    fn sign(str: &str) -> Sign {
        Sign::try_from(str).unwrap()
    }

    #[test]
    fn zero(){
        let interpretation = string_split_by_bar_interpretation();
        let tape_input = interpretation.write()("|-|-".to_string()).unwrap();
    
        let builder = zero_func(tape_input);
        let mut machine = builder.build().unwrap();
        for _ in 0..10 {
            eprintln!("{:?} {:?}", machine.now_state(), machine.now_tape());
            let _ = machine.step(1);
        }
        
        let result_tape = machine.now_tape();
        let expect_tape = TapeAsVec {
            left: vec![],
            head: sign("-"),
            right: vec![Sign::blank(), sign("-")],
        };
        assert_eq!(result_tape, expect_tape);
    }
}
