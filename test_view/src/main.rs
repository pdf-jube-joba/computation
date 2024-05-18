fn main() {
    let document = gloo::utils::document();
    let element = document
        .get_element_by_id("m")
        .unwrap();
    let examples = logic_circuit::example::examples();
    let lc = examples.get(&"DFF".into()).unwrap().clone();
    logic_circuit_view::set_machine(element, lc);
}
