use logic_circuit::machine::LoC;
use logic_circuit_view::machine::svg_lc;
use utils::view::svg::*;
use yew::Callback;

fn main() {
    let maps = logic_circuit::example::examples()
        .into_iter()
        .map(|i| i.1)
        .collect::<Vec<_>>();
    yew::Renderer::<svg_lc::GraphicEditor>::with_props(svg_lc::GraphicEditorProps {
        logic_circuits_components: maps,
        on_goto_test: Callback::noop(),
        on_log: Callback::from(|string|{
            utils::view::log(string);
        }),
    })
    .render();
}
