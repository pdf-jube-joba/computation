use logic_circuit_view::{control, machine};
use logic_circuits_to_machine::*;

fn main() {
    let document = gloo::utils::document();
    let element = document
        .get_element_by_id("logic_circuits_playground")
        .unwrap();
    logic_circuit_view::playground(element);
}
