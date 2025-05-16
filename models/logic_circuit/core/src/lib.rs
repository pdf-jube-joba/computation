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
    use utils::{alphabet::Identifier, bool::Bool};

    use crate::machine::{InPin, LogicCircuitTrait};

    use super::example::utils_map;

    #[test]
    fn test_example() {
        let list = utils_map();
        let lc = list
            .get(&Identifier::new_user("one-shot").unwrap())
            .unwrap();
        eprintln!("{:?}", lc);
        eprintln!("{:?}", lc.as_graph_group());
        eprintln!("{:?}", lc.get_inpins());
    }
    #[test]
    fn test_xor() {
        let list = utils_map();
        let mut lc = list
            .get(&Identifier::new_user("XOR").unwrap())
            .unwrap()
            .clone();
        eprintln!("{:?}", lc.get_otputs());

        let inputs: Vec<(InPin, Bool)> = vec![
            ("IN0".parse().unwrap(), Bool::F),
            ("IN1".parse().unwrap(), Bool::F),
        ];
        for _ in 0..6 {
            lc.step(inputs.clone());
            eprintln!("{:?}", lc.get_otputs());
        }

        eprintln!("----");

        let inputs: Vec<(InPin, Bool)> = vec![
            ("IN0".parse().unwrap(), Bool::T),
            ("IN1".parse().unwrap(), Bool::T),
        ];
        for _ in 0..6 {
            lc.step(inputs.clone());
            eprintln!("{:?}", lc.get_otputs());
        }

        eprintln!("----");

        let inputs: Vec<(InPin, Bool)> = vec![
            ("IN0".parse().unwrap(), Bool::T),
            ("IN1".parse().unwrap(), Bool::F),
        ];
        for _ in 0..6 {
            lc.step(inputs.clone());
            eprintln!("{:?}", lc.get_otputs());
        }
    }
}
