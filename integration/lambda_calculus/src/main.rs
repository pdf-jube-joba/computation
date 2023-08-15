use lambda_calculus_view::LambdaCalculusView;

fn main() {
    let document = gloo::utils::document();
    let machine_element = document.query_selector("#machine").unwrap().unwrap();
    let _ = yew::Renderer::<LambdaCalculusView>::with_root(machine_element).render();
}