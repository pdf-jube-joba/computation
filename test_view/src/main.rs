use logic_circuit::machine::LoC;
use logic_circuit_view::machine::svg_lc;
use utils::view::svg::*;

fn main() {
    let maps = logic_circuit::example::examples();
    let lc = maps.get(&"one-shot".into()).unwrap().clone();
    let lc = lc.take_fingraph().unwrap();
    let init_pos_lc = vec![
        ("B".into(), (Ori::U, Pos(100, 100))),
        ("N".into(), (Ori::U, Pos(150, 150))),
        ("A".into(), (Ori::U, Pos(200, 100))),
    ]
    .into_iter()
    .collect();
    yew::Renderer::<svg_lc::FingraphMachine>::with_props(
        svg_lc::FingraphMachineProps::new(lc, init_pos_lc).unwrap(),
    )
    .render();
}
