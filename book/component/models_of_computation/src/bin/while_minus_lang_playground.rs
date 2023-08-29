use while_minus_lang_view::*;

fn main() {
    let document = gloo::utils::document();
    let element = document
        .get_element_by_id("while_minus_lang_playground")
        .unwrap();
    let _handle = yew::Renderer::<WhileLangControlView>::with_root_and_props(
        element,
        WhileLangControlProps {},
    );
}
