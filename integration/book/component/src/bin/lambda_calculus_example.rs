use lambda_calculus_view::*;

fn main() {
    let document = gloo::utils::document();
    let target_element = document
        .get_element_by_id("lambda_calculus_example")
        .unwrap();

    {
        let element = document.create_element("div").unwrap();
        target_element.append_child(&element).unwrap();

        let _ = yew::Renderer::<LambdaCalculusView>::with_root(element).render();
    }
}
