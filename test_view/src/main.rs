use logic_circuit::machine::LoC;

fn main() {
    let document = gloo::utils::document();
    let element = document.get_element_by_id("m").unwrap();
    let lc = lc();
    logic_circuit_view::set_machine(element, lc);
}

fn lc() -> LoC {
    let code = include_str!("lc.txt");
    let maps = logic_circuit::example::examples();
    maps.get(&"RS-latch".into()).unwrap().clone()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn t() {
        let lc = lc();
    }
}
