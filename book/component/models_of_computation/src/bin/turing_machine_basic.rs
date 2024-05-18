use turing_machine_view::machine::*;

fn main() {
    let document = gloo::utils::document();
    let target_element = document.get_element_by_id("turing_machine_basic").unwrap();

    {
        let element = document.create_element("div").unwrap();
        target_element.append_child(&element).unwrap();

        let input = recursive_function_to_turing_machine::compile::num_tape::write(
            Vec::<usize>::new().into(),
        );
        let mut builder = recursive_function_to_turing_machine::compile::zero_builder();
        builder.input(input);
        let machine = builder.build().unwrap();

        let machine_handle = yew::Renderer::<TuringMachineView>::with_root(element).render();
        machine_handle.send_message(TuringMachineMsg::LoadFromMachine(Box::new(machine)));
    }

    {
        let element = document.create_element("div").unwrap();
        target_element.append_child(&element).unwrap();

        let input = recursive_function_to_turing_machine::compile::num_tape::write(vec![3].into());
        let mut builder = recursive_function_to_turing_machine::compile::succ_builder();
        builder.input(input);
        let machine = builder.build().unwrap();

        let machine_handle = yew::Renderer::<TuringMachineView>::with_root(element).render();
        machine_handle.send_message(TuringMachineMsg::LoadFromMachine(Box::new(machine)));
    }

    {
        let element = document.create_element("div").unwrap();
        target_element.append_child(&element).unwrap();

        let input =
            recursive_function_to_turing_machine::compile::num_tape::write(vec![3, 1, 2].into());
        let mut builder =
            recursive_function_to_turing_machine::compile::projection::projection(3, 1);
        builder.input(input);
        let machine = builder.build().unwrap();

        let machine_handle = yew::Renderer::<TuringMachineView>::with_root(element).render();
        machine_handle.send_message(TuringMachineMsg::LoadFromMachine(Box::new(machine)));
    }
}
