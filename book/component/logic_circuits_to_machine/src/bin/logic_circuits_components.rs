fn main() {
    let document = gloo::utils::document();
    let element = document
        .get_element_by_id("logic_circuits_components")
        .unwrap();
    let initmap = logic_circuit::example::examples();
}