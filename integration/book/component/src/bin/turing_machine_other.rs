use recursive_function::{machine::RecursiveFunctions, *};
use recursive_function_to_turing_machine::*;
use turing_machine_view::machine::*;

fn add() -> RecursiveFunctions {
    RecursiveFunctions::primitive_recursion(
        RecursiveFunctions::projection(1, 0).unwrap(),
        RecursiveFunctions::composition(
            3,
            vec![RecursiveFunctions::projection(3, 0).unwrap()],
            RecursiveFunctions::succ(),
        )
        .unwrap(),
    )
    .unwrap()
}

fn any_to_zero() -> RecursiveFunctions {
    RecursiveFunctions::composition(1, vec![], RecursiveFunctions::zero()).unwrap()
}

fn mul() -> RecursiveFunctions {
    RecursiveFunctions::primitive_recursion(
        any_to_zero(),
        RecursiveFunctions::composition(
            3,
            vec![
                RecursiveFunctions::projection(3, 0).unwrap(),
                RecursiveFunctions::projection(3, 2).unwrap(),
            ],
            add(),
        )
        .unwrap(),
    )
    .unwrap()
}

fn zero_from_mul() -> RecursiveFunctions {
    RecursiveFunctions::muoperator(mul()).unwrap()
}

fn main() {
    let document = gloo::utils::document();
    let target_element = document.get_element_by_id("turing_machine_other").unwrap();

    {
        let element = document.create_element("div").unwrap();
        target_element.append_child(&element).unwrap();

        let mut builder = compile::compile(
            &RecursiveFunctions::composition(
                3,
                vec![RecursiveFunctions::projection(3, 0).unwrap()],
                RecursiveFunctions::succ(),
            )
            .unwrap(),
        );
        let input = compile::num_tape::write(vec![1, 2, 3].into());
        builder.input(input);

        let _ = yew::Renderer::<UnConnectedMachineView>::with_root_and_props(
            element,
            UnConnectedMachineProp {
                builder,
                toggle_interval: 100,
            },
        )
        .render();
    }

    {
        let element = document.create_element("div").unwrap();
        target_element.append_child(&element).unwrap();

        let mut builder = compile::compile(&add());
        let input =
            recursive_function_to_turing_machine::compile::num_tape::write(vec![3, 1].into());
        builder.input(input);

        let _ = yew::Renderer::<UnConnectedMachineView>::with_root_and_props(
            element,
            UnConnectedMachineProp {
                builder,
                toggle_interval: 100,
            },
        )
        .render();
    }

    {
        let element = document.create_element("div").unwrap();
        target_element.append_child(&element).unwrap();

        let mut builder = compile::compile(&zero_from_mul());
        let input = recursive_function_to_turing_machine::compile::num_tape::write(vec![3].into());
        builder.input(input);

        target_element.append_child(&element).unwrap();
        let _ = yew::Renderer::<UnConnectedMachineView>::with_root_and_props(
            element,
            UnConnectedMachineProp {
                builder,
                toggle_interval: 100,
            },
        )
        .render();
    }
}
