use std::collections::HashMap;

use crate::manipulation::List;

use super::machine::{LoC, Name};
use super::manipulation;

pub fn examples() -> List {
    let code = include_str!("logic_circuits/examples.txt");
    let mut initmap = manipulation::init_maps();
    if let Err(err) = manipulation::parse(code, &mut initmap) {
        panic!("{err:}")
    };
    initmap
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn call() {
        let examples = examples();
    }
}
