use yew::prelude::*;

#[function_component]
fn App() -> Html {
    let counter = use_state(|| 0);
    let onclick = {
        let counter = counter.clone();
        move |_| {
            let value = *counter + 1;
            counter.set(value);
        }
    };

    html! {
        <div>
            <button {onclick}>{ "-1" }</button>
            <p>{ *counter }</p>
        </div>
    }
}

fn main() {
    let document = gloo::utils::document();
    let target_element = document.get_element_by_id("other").unwrap();
    yew::Renderer::<App>::with_root(target_element).render();
}