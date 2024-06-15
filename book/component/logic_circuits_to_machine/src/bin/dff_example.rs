use logic_circuit_view::machine::svg_lc::{ActLoCView, ActLoCProps};
use utils::view::svg::Pos;

fn main() {
    let document = gloo::utils::document();
    let element = document.get_element_by_id("dff_example").unwrap();
    let initmap = logic_circuit::example::examples();
    let dff = initmap.get(&"DFF".into()).unwrap();
    // let dff = dff.take_fingraph().unwrap();
    // yew::Renderer::<ActLoCView>::with_root_and_props(
    //     element,
    //     ActLoCProps {
    //         fingraph: dff,
    //         inpins: vec![Pos()],
    //     },
    // );
}
