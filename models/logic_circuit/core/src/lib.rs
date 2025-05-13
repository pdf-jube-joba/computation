pub mod machine;
pub mod manipulation;
pub mod example {
    use crate::manipulation::{init_maps, parse, List};

    pub fn utils_map() -> List {
        let mut list = init_maps();
        let code = include_str!("./logic_circuits/examples.txt");
        parse(code, &mut list).unwrap();
        list
    }
}

#[cfg(test)]
mod tests {
    use utils::alphabet::Identifier;

    use crate::machine::LogicCircuitTrait;

    use super::example::utils_map;

    #[test]
    fn test_example() {
        let list = utils_map();
        let lc = list
            .get(&Identifier::new_user("one-shot").unwrap())
            .unwrap();
        eprintln!("{:?}", lc);
        eprintln!("{:?}", lc.as_graph_group())
    }
}
