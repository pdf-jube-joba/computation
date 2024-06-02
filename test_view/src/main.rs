use logic_circuit::machine::LoC;
use logic_circuit_view::machine::svg_lc;

fn main() {
    let maps = logic_circuit::example::examples()
        .0
        .into_iter()
        .map(|i| i.1)
        .collect::<Vec<_>>();
    yew::Renderer::<svg_lc::PlayGround>::with_props(svg_lc::PlayGroudProps {
        init_component: maps,
    })
    .render();
}
