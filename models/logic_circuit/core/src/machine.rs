use std::{
    collections::{HashMap, HashSet},
    fmt::Display,
    str::FromStr,
};
use utils::{alphabet::Identifier, bool::Bool};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct InPin(Vec<Identifier>);

pub fn destruct_inpin(inpin: InPin) -> Option<(Identifier, InPin)> {
    let (a, b) = inpin.0.split_first()?;
    Some((a.clone(), InPin(b.to_vec())))
}

pub fn concat_inpin(indent: Identifier, inpin: InPin) -> InPin {
    let mut v = vec![indent];
    v.extend(inpin.0);
    InPin(v)
}

impl FromStr for InPin {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut v = vec![];
        for i in s.split('.') {
            v.push(Identifier::new_user(i)?);
        }
        Ok(InPin(v))
    }
}

impl Display for InPin {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            self.0
                .iter()
                .map(|i| i.to_string())
                .collect::<Vec<_>>()
                .join(".")
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct OtPin(Vec<Identifier>);

fn concat_otpin(indent: Identifier, otpin: OtPin) -> OtPin {
    let mut v = vec![indent];
    v.extend(otpin.0);
    OtPin(v)
}

impl FromStr for OtPin {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut v = vec![];
        for i in s.split('.') {
            v.push(Identifier::new_user(i)?);
        }
        Ok(OtPin(v))
    }
}

impl Display for OtPin {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            self.0
                .iter()
                .map(|i| i.to_string())
                .collect::<Vec<_>>()
                .join(".")
        )
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Graph {
    pub verts: Vec<(Identifier, LogicCircuit)>,
    pub edges: Vec<((Identifier, OtPin), (Identifier, InPin))>,
}

pub trait LogicCircuitTrait {
    fn kind(&self) -> Identifier;
    fn get_inpins(&self) -> Vec<InPin>;
    fn get_otpins(&self) -> Vec<OtPin>;
    fn get_otputs(&self) -> Vec<(OtPin, Bool)>;
    fn step(&mut self, inputs: Vec<(InPin, Bool)>);
    fn as_graph_group(&self) -> Graph;
}

#[derive(Debug, Clone, PartialEq)]
pub enum LogicCircuit {
    Gate(Gate),
    MixLogicCircuit(Box<MixLogicCircuit>),
    IterLogicCircuit(Box<IterLogicCircuit>),
    PinMap(Box<PinMap>),
}

impl LogicCircuit {
    pub fn new_gate(kind: GateKind, state: Bool) -> LogicCircuit {
        LogicCircuit::Gate(Gate { kind, state })
    }
    pub fn new_mix(
        kind: Identifier,
        verts: Vec<(Identifier, LogicCircuit)>,
        edges: Vec<((Identifier, OtPin), (Identifier, InPin))>,
    ) -> Result<LogicCircuit, String> {
        // prepare all inputs and outputs for each Logic Circuit
        let mut maps: HashMap<Identifier, (HashSet<InPin>, HashSet<OtPin>)> = HashMap::new();
        // initialize
        for (name, lc) in &verts {
            if maps.contains_key(name) {
                return Err(format!("duplicate name {name:?}"));
            }
            maps.insert(
                name.clone(),
                (
                    lc.get_inpins().into_iter().collect(),
                    lc.get_otpins().into_iter().collect(),
                ),
            );
        }

        // check if all edges are in verts
        // and no overlap
        for (no, ni) in &edges {
            let Some((_, otpins)) = maps.get_mut(&no.0) else {
                return Err(format!("edge {no:?} not in verts"));
            };
            if !otpins.remove(&no.1) {
                return Err(format!("edge {no:?} not in verts"));
            }

            let Some((inpins, _)) = maps.get_mut(&ni.0) else {
                return Err(format!("edge {ni:?} not in verts"));
            };
            if !inpins.remove(&ni.1) {
                return Err(format!("edge {ni:?} not in verts"));
            }
        }
        Ok(LogicCircuit::MixLogicCircuit(Box::new(MixLogicCircuit {
            kind,
            verts,
            edges,
        })))
    }
}

impl LogicCircuitTrait for LogicCircuit {
    fn kind(&self) -> Identifier {
        match self {
            LogicCircuit::Gate(gate) => gate.kind(),
            LogicCircuit::MixLogicCircuit(mix) => mix.kind(),
            LogicCircuit::IterLogicCircuit(iter) => iter.kind(),
            LogicCircuit::PinMap(pin_map) => pin_map.kind(),
        }
    }

    fn get_inpins(&self) -> Vec<InPin> {
        match self {
            LogicCircuit::Gate(gate) => gate.get_inpins(),
            LogicCircuit::MixLogicCircuit(mix) => mix.get_inpins(),
            LogicCircuit::IterLogicCircuit(iter) => iter.get_inpins(),
            LogicCircuit::PinMap(pin_map) => pin_map.get_inpins(),
        }
    }

    fn get_otpins(&self) -> Vec<OtPin> {
        match self {
            LogicCircuit::Gate(gate) => gate.get_otpins(),
            LogicCircuit::MixLogicCircuit(mix) => mix.get_otpins(),
            LogicCircuit::IterLogicCircuit(iter) => iter.get_otpins(),
            LogicCircuit::PinMap(pin_map) => pin_map.get_otpins(),
        }
    }

    fn get_otputs(&self) -> Vec<(OtPin, Bool)> {
        match self {
            LogicCircuit::Gate(gate) => gate.get_otputs(),
            LogicCircuit::MixLogicCircuit(mix) => mix.get_otputs(),
            LogicCircuit::IterLogicCircuit(iter) => iter.get_otputs(),
            LogicCircuit::PinMap(pin_map) => pin_map.get_otputs(),
        }
    }

    fn step(&mut self, inputs: Vec<(InPin, Bool)>) {
        match self {
            LogicCircuit::Gate(gate) => gate.step(inputs),
            LogicCircuit::MixLogicCircuit(mix) => mix.step(inputs),
            LogicCircuit::IterLogicCircuit(iter) => iter.step(inputs),
            LogicCircuit::PinMap(pin_map) => pin_map.step(inputs),
        }
    }

    fn as_graph_group(&self) -> Graph {
        match self {
            LogicCircuit::Gate(gate) => gate.as_graph_group(),
            LogicCircuit::MixLogicCircuit(mix) => mix.as_graph_group(),
            LogicCircuit::IterLogicCircuit(iter) => iter.as_graph_group(),
            LogicCircuit::PinMap(pin_map) => pin_map.as_graph_group(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GateKind {
    Cst,
    Not,
    And,
    Or,
    Br,
    Delay,
    End,
}

impl Display for GateKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GateKind::Cst => write!(f, "CST"),
            GateKind::Not => write!(f, "NOT"),
            GateKind::And => write!(f, "AND"),
            GateKind::Or => write!(f, "OR"),
            GateKind::Br => write!(f, "BR"),
            GateKind::Delay => write!(f, "DLY"),
            GateKind::End => write!(f, "END"),
        }
    }
}

fn get_inputs_from_map(inputs: &[(InPin, Bool)], inpin: &InPin) -> Bool {
    inputs
        .iter()
        .find(|(i, _)| i == inpin)
        .map(|(_, b)| *b)
        .unwrap_or(Bool::F)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Gate {
    kind: GateKind,
    state: Bool,
}

impl LogicCircuitTrait for Gate {
    fn kind(&self) -> Identifier {
        Identifier::System(self.kind.to_string())
    }

    fn get_inpins(&self) -> Vec<InPin> {
        match self.kind {
            GateKind::Cst => vec![],
            GateKind::Not | GateKind::Br | GateKind::Delay | GateKind::End => {
                vec!["IN".parse().unwrap()]
            }
            GateKind::And | GateKind::Or => {
                vec!["IN0".parse().unwrap(), "IN1".parse().unwrap()]
            }
        }
    }

    fn get_otpins(&self) -> Vec<OtPin> {
        match self.kind {
            GateKind::Cst | GateKind::Not | GateKind::Delay | GateKind::And | GateKind::Or => {
                vec!["OUT".parse().unwrap()]
            }
            GateKind::Br => vec!["OUT0".parse().unwrap(), "OUT1".parse().unwrap()],
            GateKind::End => vec![],
        }
    }

    fn get_otputs(&self) -> Vec<(OtPin, Bool)> {
        self.get_otpins()
            .into_iter()
            .map(|otpin| (otpin, self.state))
            .collect()
    }

    fn step(&mut self, inputs: Vec<(InPin, Bool)>) {
        match self.kind {
            GateKind::Cst => {}
            GateKind::Not => {
                self.state = !get_inputs_from_map(&inputs, &"IN".parse().unwrap());
            }
            GateKind::And => {
                self.state = get_inputs_from_map(&inputs, &"IN0".parse().unwrap())
                    & get_inputs_from_map(&inputs, &"IN1".parse().unwrap());
            }
            GateKind::Or => {
                self.state = get_inputs_from_map(&inputs, &"IN0".parse().unwrap())
                    | get_inputs_from_map(&inputs, &"IN1".parse().unwrap());
            }
            GateKind::Br => {
                self.state = get_inputs_from_map(&inputs, &"IN".parse().unwrap());
            }
            GateKind::Delay => {
                self.state = get_inputs_from_map(&inputs, &"IN".parse().unwrap());
            }
            GateKind::End => {
                self.state = get_inputs_from_map(&inputs, &"IN".parse().unwrap());
            }
        }
    }

    fn as_graph_group(&self) -> Graph {
        Graph {
            verts: vec![(self.kind(), LogicCircuit::Gate(self.clone()))],
            edges: vec![],
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PinMap {
    pub this: LogicCircuit,
    pub inpin_maps: Vec<(InPin, InPin)>, // access by first, actually mapped to second
    pub otpin_maps: Vec<(OtPin, OtPin)>, // same as above
}

impl LogicCircuitTrait for PinMap {
    fn kind(&self) -> Identifier {
        self.this.kind()
    }

    fn get_inpins(&self) -> Vec<InPin> {
        self.this
            .get_inpins()
            .into_iter()
            .map(|i| {
                if let Some((i1, _)) = self.inpin_maps.iter().find(|(_, i2)| i2 == &i) {
                    i1.clone()
                } else {
                    i
                }
            })
            .collect()
    }

    fn get_otpins(&self) -> Vec<OtPin> {
        self.this
            .get_otpins()
            .into_iter()
            .map(|o| {
                if let Some((o1, _)) = self.otpin_maps.iter().find(|(_, o2)| o2 == &o) {
                    o1.clone()
                } else {
                    o
                }
            })
            .collect()
    }

    fn get_otputs(&self) -> Vec<(OtPin, Bool)> {
        self.this
            .get_otputs()
            .into_iter()
            .map(|(o, b)| {
                if let Some((o1, _)) = self.otpin_maps.iter().find(|(_, o2)| o2 == &o) {
                    (o1.clone(), b)
                } else {
                    (o, b)
                }
            })
            .collect()
    }

    fn step(&mut self, inputs: Vec<(InPin, Bool)>) {
        let mut new_inputs = vec![];
        for (i, b) in inputs {
            if let Some((_, i2)) = self.inpin_maps.iter().find(|(i1, _)| i1 == &i) {
                new_inputs.push((i2.clone(), b));
            } else {
                new_inputs.push((i, b));
            }
        }
        self.this.step(new_inputs);
    }

    fn as_graph_group(&self) -> Graph {
        Graph {
            verts: vec![(self.kind(), self.this.clone())],
            edges: vec![],
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MixLogicCircuit {
    pub kind: Identifier,
    pub verts: Vec<(Identifier, LogicCircuit)>,
    pub edges: Vec<((Identifier, OtPin), (Identifier, InPin))>,
}

impl LogicCircuitTrait for MixLogicCircuit {
    fn kind(&self) -> Identifier {
        self.kind.clone()
    }

    fn get_inpins(&self) -> Vec<InPin> {
        self.verts
            .iter()
            .flat_map(|(s, g)| {
                g.get_inpins()
                    .into_iter()
                    .filter(move |i| self.edges.iter().all(|(_, (s2, i2))| s != s2 || i != i2))
                    .map(|i| concat_inpin(s.clone(), i))
            })
            .collect()
    }

    fn get_otpins(&self) -> Vec<OtPin> {
        self.verts
            .iter()
            .flat_map(|(s, g)| {
                g.get_otpins()
                    .into_iter()
                    .filter(move |o| self.edges.iter().all(|((s2, o2), _)| s != s2 || o != o2))
                    .map(|o| concat_otpin(s.clone(), o))
            })
            .collect()
    }

    fn get_otputs(&self) -> Vec<(OtPin, Bool)> {
        self.verts
            .iter()
            .flat_map(|(s, g)| {
                g.get_otputs()
                    .into_iter()
                    .filter(move |(o, _)| self.edges.iter().all(|((s2, o2), _)| s != s2 || o != o2))
                    .map(|(o, b)| (concat_otpin(s.clone(), o), b))
            })
            .collect()
    }

    fn step(&mut self, inputs: Vec<(InPin, Bool)>) {
        // inputs for each Logic Circuits (key by name)
        let mut new_inputs: HashMap<Identifier, Vec<(InPin, Bool)>> = HashMap::new();
        // initialize
        for (name, _) in &self.verts {
            new_inputs.insert(name.clone(), vec![]);
        }
        // after this, there is no new insertions to `new_inputs`

        // from other Logic Circuits
        // priority is high
        for (name, loc) in &self.verts {
            for (otpins, b) in loc.get_otputs() {
                if let Some((namei, i)) = self.edges.iter().find_map(|((name2, o), ni)| {
                    if name == name2 && o == &otpins {
                        Some(ni)
                    } else {
                        None
                    }
                }) {
                    new_inputs.get_mut(namei).unwrap().push((i.clone(), b));
                }
            }
        }

        // from inputs
        // priority is low => check if already used
        for (i, b) in inputs {
            let Some((name, rest)) = destruct_inpin(i) else {
                eprintln!("Invalid InPin");
                continue;
            };
            let Some(v) = new_inputs.get_mut(&name) else {
                eprintln!("Invalid Name {name}");
                continue;
            };
            if v.iter().all(|(i2, _)| i2 != &rest) {
                v.push((rest, b));
            }
        }

        for (name, inputs) in new_inputs {
            let lc = self
                .verts
                .iter_mut()
                .find_map(|(name2, lc)| if *name2 == name { Some(lc) } else { None })
                .unwrap(); // unwrap is safe because we initialized `new_inputs` with all names
            lc.step(inputs);
        }
    }

    fn as_graph_group(&self) -> Graph {
        Graph {
            verts: self.verts.clone(),
            edges: self.edges.clone(),
        }
    }
}

pub fn num_to_ident(n: usize) -> Identifier {
    Identifier::new_system(&format!("{n}"))
}

#[derive(Debug, Clone, PartialEq)]
pub struct IterLogicCircuit {
    pub kind: Identifier,
    // for extending `used`. push `init`` into `used` when needed
    // i.e. `used` can be thought as a infinity list of locigcirtuis
    // all of them are `init` except the finite number of them
    pub init: LogicCircuit,
    pub used: Vec<LogicCircuit>,
    pub next_edges: Vec<(OtPin, InPin)>, // edge from used[i] -> used[i+1]
    pub prev_edges: Vec<(OtPin, InPin)>, // edge from used[i] -> used[i-1]
}

impl LogicCircuitTrait for IterLogicCircuit {
    fn kind(&self) -> Identifier {
        self.kind.clone()
    }

    fn get_inpins(&self) -> Vec<InPin> {
        self.used
            .iter()
            .enumerate()
            .flat_map(|(n, g)| {
                g.get_inpins()
                    .into_iter()
                    .filter(|inpin| {
                        self.next_edges.iter().all(|(_, i2)| inpin != i2)
                            && self.prev_edges.iter().all(|(_, i2)| inpin != i2)
                    })
                    .map(move |inpin| concat_inpin(num_to_ident(n), inpin))
            })
            .collect()
    }

    fn get_otpins(&self) -> Vec<OtPin> {
        self.used
            .iter()
            .enumerate()
            .flat_map(|(n, g)| {
                g.get_otpins()
                    .into_iter()
                    .filter(|otpin| {
                        self.next_edges.iter().all(|(o2, _)| otpin != o2)
                            && self.prev_edges.iter().all(|(o2, _)| otpin != o2)
                    })
                    .map(move |otpin| concat_otpin(num_to_ident(n), otpin))
            })
            .collect()
    }

    fn get_otputs(&self) -> Vec<(OtPin, Bool)> {
        self.used
            .iter()
            .enumerate()
            .flat_map(|(n, g)| {
                g.get_otputs()
                    .into_iter()
                    .filter(|(otpin, _)| {
                        self.next_edges.iter().all(|(o2, _)| otpin != o2)
                            && self.prev_edges.iter().all(|(o2, _)| otpin != o2)
                    })
                    .map(move |(otpin, b)| (concat_otpin(num_to_ident(n), otpin), b))
            })
            .collect()
    }

    fn step(&mut self, inputs: Vec<(InPin, Bool)>) {
        // inputs for each Logic Circuits (key by index)
        // initialized
        let mut new_inputs: Vec<Vec<(InPin, Bool)>> = vec![vec![]; self.used.len() + 1];

        // from other Logic Circuits
        // priority is high
        for (num, lc) in self.used.iter().enumerate() {
            for (o, b) in lc.get_otputs() {
                // send inputs to next Logic Circuits
                if let Some(i) =
                    self.next_edges
                        .iter()
                        .find_map(|(o2, i)| if o == *o2 { Some(i) } else { None })
                {
                    new_inputs[num + 1].push((i.clone(), b));
                }
                // send inputs to previous Logic Circuits
                if let Some(i) =
                    self.prev_edges
                        .iter()
                        .find_map(|(o2, i)| if o == *o2 { Some(i) } else { None })
                {
                    new_inputs[num - 1].push((i.clone(), b));
                }
            }
        }

        // from inputs
        // priority is low => check if already used
        for (i, b) in inputs {
            let Some((num, rest)) = destruct_inpin(i) else {
                eprintln!("Invalid InPin");
                continue;
            };
            let Some(num) = num.to_usize() else {
                eprint!("ignored input {num} because not a number");
                continue;
            };
            let v = &mut new_inputs[num];
            if v.iter().all(|(i2, _)| i2 != &rest) {
                v.push((rest, b));
            }
        }

        // step each Logic Circuits
        for (num, inputs) in new_inputs.into_iter().enumerate() {
            if num != self.used.len() {
                self.used[num].step(inputs);
            } else {
                // if inputs is not all false, push init into used
                if inputs.iter().any(|(_, b)| *b == Bool::T) {
                    self.used.push(self.init.clone());
                    self.used[num].step(inputs);
                }
            }
        }
    }

    fn as_graph_group(&self) -> Graph {
        let edges = {
            let mut edges = vec![];
            for (i, _) in self.used.iter().enumerate() {
                if i < self.used.len() - 1 {
                    for (otpin, inpin) in &self.next_edges {
                        edges.push((
                            (num_to_ident(i), otpin.clone()),
                            (num_to_ident(i + 1), inpin.clone()),
                        ));
                    }
                }
                if i > 0 {
                    for (otpin, inpin) in &self.prev_edges {
                        edges.push((
                            (num_to_ident(i), otpin.clone()),
                            (num_to_ident(i - 1), inpin.clone()),
                        ));
                    }
                }
            }
            edges
        };
        Graph {
            verts: self
                .used
                .iter()
                .enumerate()
                .map(|(i, lc)| (num_to_ident(i), lc.clone()))
                .collect(),
            edges,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use utils::bool::Bool;

    #[test]
    fn test_gate() {
        let mut gate = Gate {
            kind: GateKind::And,
            state: Bool::F,
        };
        assert_eq!(
            gate.get_inpins(),
            vec!["IN0".parse().unwrap(), "IN1".parse().unwrap()]
        );
        assert_eq!(gate.get_otpins(), vec!["OUT".parse().unwrap()]);
        assert_eq!(gate.get_otputs(), vec![("OUT".parse().unwrap(), Bool::F)]);
        gate.step(vec![
            ("IN0".parse().unwrap(), Bool::T),
            ("IN1".parse().unwrap(), Bool::T),
        ]);
        assert_eq!(gate.state, Bool::T);
        assert_eq!(gate.get_otputs(), vec![("OUT".parse().unwrap(), Bool::T)]);
        gate.step(vec![
            ("IN0".parse().unwrap(), Bool::T),
            ("IN1".parse().unwrap(), Bool::F),
        ]);
        assert_eq!(gate.state, Bool::F);
        assert_eq!(gate.get_otputs(), vec![("OUT".parse().unwrap(), Bool::F)]);
    }
    #[test]
    fn test_pin_map() {
        let mut pin_map = PinMap {
            this: LogicCircuit::Gate(Gate {
                kind: GateKind::And,
                state: Bool::F,
            }),
            inpin_maps: vec![("I".parse().unwrap(), "IN0".parse().unwrap())],
            otpin_maps: vec![("O".parse().unwrap(), "OUT".parse().unwrap())],
        };
        assert_eq!(
            pin_map.get_inpins(),
            vec!["I".parse().unwrap(), "IN1".parse().unwrap()]
        );
        assert_eq!(pin_map.get_otpins(), vec!["O".parse().unwrap()]);
        assert_eq!(pin_map.get_otputs(), vec![("O".parse().unwrap(), Bool::F)]);

        pin_map.step(vec![("I".parse().unwrap(), Bool::T)]);
        assert_eq!(pin_map.get_otputs(), vec![("O".parse().unwrap(), Bool::F)]);
    }
    #[test]
    fn test_mix() {
        // graph like
        // A.IN0 ---> A <--- A.IN1
        //            |-- A.OUT == B.IN0 ---> B <--- B.IN1
        //                                    |--> B.OUT
        // B.OUT takes 2 step to be T if A.IN0 and A.IN1 are T
        // B.OUT takes 1 step to be T is B.IN1 is T
        let mut mix = MixLogicCircuit {
            kind: Identifier::new_user("MIX").unwrap(),
            verts: vec![
                (
                    Identifier::new_user("A").unwrap(),
                    LogicCircuit::Gate(Gate {
                        kind: GateKind::And,
                        state: Bool::F,
                    }),
                ),
                (
                    Identifier::new_user("B").unwrap(),
                    LogicCircuit::Gate(Gate {
                        kind: GateKind::Or,
                        state: Bool::F,
                    }),
                ),
            ],
            edges: vec![(
                (Identifier::new_user("A").unwrap(), "OUT".parse().unwrap()),
                (Identifier::new_user("B").unwrap(), "IN0".parse().unwrap()),
            )],
        };
        assert_eq!(
            mix.get_inpins(),
            vec![
                "A.IN0".parse().unwrap(),
                "A.IN1".parse().unwrap(),
                "B.IN1".parse().unwrap()
            ]
        );
        assert_eq!(mix.get_otpins(), vec!["B.OUT".parse().unwrap()]);
        assert_eq!(mix.get_otputs(), vec![("B.OUT".parse().unwrap(), Bool::F)]);

        mix.step(vec![("A.IN0".parse().unwrap(), Bool::T)]);
        assert_eq!(mix.get_otputs(), vec![("B.OUT".parse().unwrap(), Bool::F)]);

        mix.step(vec![("A.IN0".parse().unwrap(), Bool::T)]);
        assert_eq!(mix.get_otputs(), vec![("B.OUT".parse().unwrap(), Bool::F)]);

        // A.IN0 and A.IN1 are T
        mix.step(vec![
            ("A.IN0".parse().unwrap(), Bool::T),
            ("A.IN1".parse().unwrap(), Bool::T),
            // B.IN1 is F if not set
        ]);
        assert_eq!(mix.get_otputs(), vec![("B.OUT".parse().unwrap(), Bool::F)]);
        mix.step(vec![
            ("A.IN0".parse().unwrap(), Bool::F),
            ("A.IN1".parse().unwrap(), Bool::F),
            // B.IN1 is F if not set
        ]);
        assert_eq!(mix.get_otputs(), vec![("B.OUT".parse().unwrap(), Bool::T)]);
    }
}
