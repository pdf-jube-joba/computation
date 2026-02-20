use crate::machine::*;
use utils::{bool::Bool, identifier::Identifier};

#[derive(Debug, Clone, PartialEq)]
pub struct List(pub Vec<(Identifier, LogicCircuit)>);

impl<T> From<T> for List
where
    T: IntoIterator<Item = (Identifier, LogicCircuit)>,
{
    fn from(value: T) -> Self {
        List(value.into_iter().collect())
    }
}

impl List {
    pub fn get(&self, name: &Identifier) -> Option<&LogicCircuit> {
        self.0
            .iter()
            .find_map(|v| if &v.0 == name { Some(&v.1) } else { None })
    }
    pub fn insert(&mut self, name_loc: (Identifier, LogicCircuit)) -> Option<()> {
        if self.get(&name_loc.0).is_none() {
            self.0.push(name_loc);
            Some(())
        } else {
            None
        }
    }
}

pub fn by_const(name: &str) -> Identifier {
    Identifier::new(name).unwrap()
}

// contains fundamental gate
pub fn init_maps() -> List {
    vec![
        (
            by_const("NOT-T"),
            LogicCircuit::new_gate(GateKind::Not, Bool::T),
        ),
        (
            by_const("NOT-F"),
            LogicCircuit::new_gate(GateKind::Not, Bool::F),
        ),
        (
            by_const("AND-T"),
            LogicCircuit::new_gate(GateKind::And, Bool::T),
        ),
        (
            by_const("AND-F"),
            LogicCircuit::new_gate(GateKind::And, Bool::F),
        ),
        (
            by_const("OR-T"),
            LogicCircuit::new_gate(GateKind::Or, Bool::T),
        ),
        (
            by_const("OR-F"),
            LogicCircuit::new_gate(GateKind::Or, Bool::F),
        ),
        (
            by_const("CST-T"),
            LogicCircuit::new_gate(GateKind::Cst, Bool::T),
        ),
        (
            by_const("CST-F"),
            LogicCircuit::new_gate(GateKind::Cst, Bool::F),
        ),
        (
            by_const("BR-T"),
            LogicCircuit::new_gate(GateKind::Br, Bool::T),
        ),
        (
            by_const("BR-F"),
            LogicCircuit::new_gate(GateKind::Br, Bool::F),
        ),
        (
            by_const("END"),
            LogicCircuit::new_gate(GateKind::End, Bool::F),
        ),
        (
            by_const("DLY-T"),
            LogicCircuit::new_gate(GateKind::Delay, Bool::T),
        ),
        (
            by_const("DLY-F"),
            LogicCircuit::new_gate(GateKind::Delay, Bool::F),
        ),
    ]
    .into()
}
