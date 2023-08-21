use recursive_function_view::{FunctionControlView, FunctionControlProps};

fn main() {
    let document = gloo::utils::document();
    let machine_element = document.query_selector("#machine").unwrap().unwrap();
    let _ = yew::Renderer::<FunctionControlView>::with_root_and_props(
        machine_element,
        FunctionControlProps {}
    ).render();
}