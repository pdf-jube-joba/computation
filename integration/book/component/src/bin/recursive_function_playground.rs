use recursive_function_view::{FunctionControlProps, FunctionControlView};

fn main() {
    let document = gloo::utils::document();
    let element = document
        .get_element_by_id("recursive_function_playground")
        .unwrap();
    let _ =
        yew::Renderer::<FunctionControlView>::with_root_and_props(element, FunctionControlProps {})
            .render();
}
