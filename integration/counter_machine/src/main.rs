fn main() {
    let document = gloo::utils::document();
    let machine_element = document.query_selector("#hello").unwrap().unwrap();
    let mut maps = logic_circuit::example::examples();
    let code = include_str!("test.txt");
    logic_circuit::manipulation::parse(code, &mut maps).unwrap();
    logic_circuit_view::playground_with(machine_element, maps);
}
