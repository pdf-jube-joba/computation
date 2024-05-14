fn main() {
    let document = gloo::utils::document();
    let element = document
        .get_element_by_id("logic_circuits_playground")
        .unwrap();
    logic_circuit_view::playground(element);
}
