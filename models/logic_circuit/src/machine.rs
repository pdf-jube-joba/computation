use anyhow::{bail, Result};
use std::{
    collections::{HashMap, HashSet},
    fmt::Display,
};
use utils::{bool::Bool, identifier::Identifier};

pub type Pin = Identifier;
pub type NamedPin = (Identifier, Identifier);

fn pin_name(pin: &NamedPin) -> &Identifier {
    &pin.1
}

fn make_pin(name: Identifier, pin: Identifier) -> NamedPin {
    (name, pin)
}

#[derive(Debug, Clone, PartialEq)]
pub struct Graph {
    pub verts: Vec<(Identifier, LogicCircuit)>,
    pub edges: Vec<(NamedPin, NamedPin)>,
    pub inpins_map: Vec<(NamedPin, NamedPin)>,
    pub otpins_map: Vec<(NamedPin, NamedPin)>,
}

pub trait LogicCircuitTrait {
    fn kind(&self) -> Identifier;
    fn get_inpins(&self) -> Vec<NamedPin>;
    fn get_otpins(&self) -> Vec<NamedPin>;
    fn get_otputs(&self) -> Vec<(NamedPin, Bool)>;
    fn step(&mut self, inputs: Vec<(NamedPin, Bool)>);
    fn as_graph_group(&self) -> Graph;
}

#[derive(Debug, Clone, PartialEq)]
pub enum LogicCircuit {
    Gate(Gate),
    MixLogicCircuit(Box<MixLogicCircuit>),
    IterLogicCircuit(Box<IterLogicCircuit>),
}

impl LogicCircuit {
    pub fn state_as_gate(&self) -> Option<Bool> {
        if let LogicCircuit::Gate(gate) = self {
            Some(gate.state)
        } else {
            None
        }
    }
    pub fn new_gate(kind: GateKind, state: Bool) -> LogicCircuit {
        LogicCircuit::Gate(Gate { kind, state })
    }
    pub fn new_mix(
        kind: Identifier,
        verts: Vec<(Identifier, LogicCircuit)>,
        edges: Vec<(NamedPin, NamedPin)>,
        inpin_maps: Vec<(Identifier, NamedPin)>,
        otpin_maps: Vec<(Identifier, NamedPin)>,
    ) -> Result<LogicCircuit> {
        // prepare all inputs and outputs for each Logic Circuit
        let mut maps: HashMap<Identifier, (HashSet<Pin>, HashSet<Pin>)> = HashMap::new();
        // initialize
        for (name, lc) in &verts {
            if maps.contains_key(name) {
                bail!("duplicate name {name:?}");
            }
            maps.insert(
                name.clone(),
                (
                    lc.get_inpins().into_iter().map(|(_, p)| p).collect(),
                    lc.get_otpins().into_iter().map(|(_, p)| p).collect(),
                ),
            );
        }

        // check if all edges are in verts
        // and no overlap
        for (no, ni) in &edges {
            let Some((_, otpins)) = maps.get_mut(&no.0) else {
                bail!("edge {no:?} not in verts");
            };
            if !otpins.remove(&no.1) {
                bail!("edge {no:?} not in verts");
            }

            let Some((inpins, _)) = maps.get_mut(&ni.0) else {
                bail!("edge {ni:?} not in verts");
            };
            if !inpins.remove(&ni.1) {
                bail!("edge {ni:?} not in verts");
            }
        }
        Ok(LogicCircuit::MixLogicCircuit(Box::new(MixLogicCircuit {
            kind,
            verts,
            edges,
            inpin_maps,
            otpin_maps,
        })))
    }
    pub fn new_iter(
        kind: Identifier,
        init: LogicCircuit,
        next_edges: Vec<(Pin, Pin)>,
        prev_edges: Vec<(Pin, Pin)>,
    ) -> Result<LogicCircuit> {
        Ok(LogicCircuit::IterLogicCircuit(Box::new(IterLogicCircuit {
            kind,
            init,
            used: vec![],
            next_edges,
            prev_edges,
            inpin_maps: vec![],
            otpin_maps: vec![],
        })))
    }
}

impl LogicCircuitTrait for LogicCircuit {
    fn kind(&self) -> Identifier {
        match self {
            LogicCircuit::Gate(gate) => gate.kind(),
            LogicCircuit::MixLogicCircuit(mix) => mix.kind(),
            LogicCircuit::IterLogicCircuit(iter) => iter.kind(),
        }
    }

    fn get_inpins(&self) -> Vec<NamedPin> {
        match self {
            LogicCircuit::Gate(gate) => gate.get_inpins(),
            LogicCircuit::MixLogicCircuit(mix) => mix.get_inpins(),
            LogicCircuit::IterLogicCircuit(iter) => iter.get_inpins(),
        }
    }

    fn get_otpins(&self) -> Vec<NamedPin> {
        match self {
            LogicCircuit::Gate(gate) => gate.get_otpins(),
            LogicCircuit::MixLogicCircuit(mix) => mix.get_otpins(),
            LogicCircuit::IterLogicCircuit(iter) => iter.get_otpins(),
        }
    }

    fn get_otputs(&self) -> Vec<(NamedPin, Bool)> {
        match self {
            LogicCircuit::Gate(gate) => gate.get_otputs(),
            LogicCircuit::MixLogicCircuit(mix) => mix.get_otputs(),
            LogicCircuit::IterLogicCircuit(iter) => iter.get_otputs(),
        }
    }

    fn step(&mut self, inputs: Vec<(NamedPin, Bool)>) {
        match self {
            LogicCircuit::Gate(gate) => gate.step(inputs),
            LogicCircuit::MixLogicCircuit(mix) => mix.step(inputs),
            LogicCircuit::IterLogicCircuit(iter) => iter.step(inputs),
        }
    }

    fn as_graph_group(&self) -> Graph {
        match self {
            LogicCircuit::Gate(gate) => gate.as_graph_group(),
            LogicCircuit::MixLogicCircuit(mix) => mix.as_graph_group(),
            LogicCircuit::IterLogicCircuit(iter) => iter.as_graph_group(),
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

fn get_inputs_from_map(inputs: &[(NamedPin, Bool)], inpin: &Pin) -> Bool {
    inputs
        .iter()
        .find(|(i, _)| pin_name(i) == inpin)
        .map(|(_, b)| *b)
        .unwrap_or(Bool::F)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Gate {
    pub(crate) kind: GateKind,
    pub(crate) state: Bool,
}

impl LogicCircuitTrait for Gate {
    fn kind(&self) -> Identifier {
        Identifier::new(self.kind.to_string()).unwrap()
    }

    fn get_inpins(&self) -> Vec<NamedPin> {
        let name = self.kind();
        match self.kind {
            GateKind::Cst => vec![],
            GateKind::Not | GateKind::Br | GateKind::Delay | GateKind::End => {
                vec![make_pin(name, Identifier::new("IN").unwrap())]
            }
            GateKind::And | GateKind::Or => {
                vec![
                    make_pin(name.clone(), Identifier::new("IN0").unwrap()),
                    make_pin(name, Identifier::new("IN1").unwrap()),
                ]
            }
        }
    }

    fn get_otpins(&self) -> Vec<NamedPin> {
        let name = self.kind();
        match self.kind {
            GateKind::Cst | GateKind::Not | GateKind::Delay | GateKind::And | GateKind::Or => {
                vec![make_pin(name, Identifier::new("OUT").unwrap())]
            }
            GateKind::Br => vec![
                make_pin(name.clone(), Identifier::new("OUT0").unwrap()),
                make_pin(name, Identifier::new("OUT1").unwrap()),
            ],
            GateKind::End => vec![],
        }
    }

    fn get_otputs(&self) -> Vec<(NamedPin, Bool)> {
        self.get_otpins()
            .into_iter()
            .map(|otpin| (otpin, self.state))
            .collect()
    }

    fn step(&mut self, inputs: Vec<(NamedPin, Bool)>) {
        match self.kind {
            GateKind::Cst => {}
            GateKind::Not => {
                self.state = !get_inputs_from_map(&inputs, &Identifier::new("IN").unwrap());
            }
            GateKind::And => {
                self.state = get_inputs_from_map(&inputs, &Identifier::new("IN0").unwrap())
                    & get_inputs_from_map(&inputs, &Identifier::new("IN1").unwrap());
            }
            GateKind::Or => {
                self.state = get_inputs_from_map(&inputs, &Identifier::new("IN0").unwrap())
                    | get_inputs_from_map(&inputs, &Identifier::new("IN1").unwrap());
            }
            GateKind::Br => {
                self.state = get_inputs_from_map(&inputs, &Identifier::new("IN").unwrap());
            }
            GateKind::Delay => {
                self.state = get_inputs_from_map(&inputs, &Identifier::new("IN").unwrap());
            }
            GateKind::End => {
                self.state = get_inputs_from_map(&inputs, &Identifier::new("IN").unwrap());
            }
        }
    }

    fn as_graph_group(&self) -> Graph {
        Graph {
            verts: vec![(self.kind(), LogicCircuit::Gate(self.clone()))],
            edges: vec![],
            inpins_map: self
                .get_inpins()
                .into_iter()
                .map(|i| (i.clone(), i))
                .collect(),
            otpins_map: self
                .get_otpins()
                .into_iter()
                .map(|o| (o.clone(), o))
                .collect(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MixLogicCircuit {
    pub kind: Identifier,
    pub verts: Vec<(Identifier, LogicCircuit)>,
    pub edges: Vec<(NamedPin, NamedPin)>,
    pub inpin_maps: Vec<(Identifier, NamedPin)>,
    pub otpin_maps: Vec<(Identifier, NamedPin)>,
}

impl MixLogicCircuit {
    fn usable_inpins(&self) -> Vec<NamedPin> {
        self.verts
            .iter()
            .flat_map(|(s, g)| {
                g.get_inpins()
                    .into_iter()
                    .map(|(_, pin)| pin)
                    .map(|pin| make_pin(s.clone(), pin))
                    .filter(move |pin| self.edges.iter().all(|(_, i2)| pin != i2))
            })
            .collect()
    }
    fn usable_otpins(&self) -> Vec<NamedPin> {
        self.verts
            .iter()
            .flat_map(|(s, g)| {
                g.get_otpins()
                    .into_iter()
                    .map(|(_, pin)| pin)
                    .map(|pin| make_pin(s.clone(), pin))
                    .filter(move |pin| self.edges.iter().all(|(o2, _)| pin != o2))
            })
            .collect()
    }

    fn map_internal_inpin(&self, internal: &NamedPin) -> NamedPin {
        if let Some((external, _)) = self
            .inpin_maps
            .iter()
            .find(|(_, target)| target == internal)
        {
            make_pin(self.kind.clone(), external.clone())
        } else {
            internal.clone()
        }
    }

    fn map_internal_otpin(&self, internal: &NamedPin) -> NamedPin {
        if let Some((external, _)) = self
            .otpin_maps
            .iter()
            .find(|(_, target)| target == internal)
        {
            make_pin(self.kind.clone(), external.clone())
        } else {
            internal.clone()
        }
    }

    fn resolve_input(&self, input: NamedPin) -> Option<NamedPin> {
        if input.0 == self.kind {
            self.inpin_maps
                .iter()
                .find(|(external, _)| external == &input.1)
                .map(|(_, target)| target.clone())
        } else {
            Some(input)
        }
    }
}

impl LogicCircuitTrait for MixLogicCircuit {
    fn kind(&self) -> Identifier {
        self.kind.clone()
    }

    fn get_inpins(&self) -> Vec<NamedPin> {
        self.usable_inpins()
            .into_iter()
            .map(|pin| self.map_internal_inpin(&pin))
            .collect()
    }

    fn get_otpins(&self) -> Vec<NamedPin> {
        self.usable_otpins()
            .into_iter()
            .map(|pin| self.map_internal_otpin(&pin))
            .collect()
    }

    fn get_otputs(&self) -> Vec<(NamedPin, Bool)> {
        self.verts
            .iter()
            .flat_map(|(s, g)| {
                g.get_otputs()
                    .into_iter()
                    .map(|(o, b)| (make_pin(s.clone(), pin_name(&o).clone()), b))
                    .filter(move |(o, _)| self.edges.iter().all(|(from, _)| from != o))
                    .map(|(o, b)| (self.map_internal_otpin(&o), b))
            })
            .collect()
    }

    fn step(&mut self, inputs: Vec<(NamedPin, Bool)>) {
        // inputs for each Logic Circuits (key by name)
        let mut new_inputs: HashMap<Identifier, Vec<(NamedPin, Bool)>> = HashMap::new();
        // initialize
        for (name, _) in &self.verts {
            new_inputs.insert(name.clone(), vec![]);
        }
        // after this, there is no new insertions to `new_inputs`

        // from other Logic Circuits
        // priority is high
        for (name, loc) in &self.verts {
            for (otpins, b) in loc.get_otputs() {
                let output = make_pin(name.clone(), pin_name(&otpins).clone());
                if let Some((_, i)) =
                    self.edges
                        .iter()
                        .find_map(|(o, ni)| if o == &output { Some((o, ni)) } else { None })
                {
                    new_inputs.get_mut(&i.0).unwrap().push((i.clone(), b));
                }
            }
        }

        // from inputs
        // priority is low => check if already used
        for (i, b) in inputs {
            let Some(internal) = self.resolve_input(i) else {
                eprintln!("Invalid InPin");
                continue;
            };
            let Some(v) = new_inputs.get_mut(&internal.0) else {
                eprintln!("Invalid Name {}", internal.0);
                continue;
            };
            if v.iter().all(|(i2, _)| i2 != &internal) {
                v.push((internal, b));
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
            inpins_map: self
                .usable_inpins()
                .into_iter()
                .map(|i| (self.map_internal_inpin(&i), i))
                .collect(),
            otpins_map: self
                .usable_otpins()
                .into_iter()
                .map(|o| (self.map_internal_otpin(&o), o))
                .collect(),
        }
    }
}

pub fn num_to_ident(n: usize) -> Identifier {
    Identifier::new(format!("_{n}_")).unwrap()
}

#[derive(Debug, Clone, PartialEq)]
pub struct IterLogicCircuit {
    pub kind: Identifier,
    // for extending `used`. push `init`` into `used` when needed
    // i.e. `used` can be thought as a infinity list of locigcirtuis
    // all of them are `init` except the finite number of them
    pub init: LogicCircuit,
    pub used: Vec<LogicCircuit>,
    pub next_edges: Vec<(Pin, Pin)>, // edge from used[i] -> used[i+1]
    pub prev_edges: Vec<(Pin, Pin)>, // edge from used[i] -> used[i-1]
    pub inpin_maps: Vec<(Identifier, NamedPin)>,
    pub otpin_maps: Vec<(Identifier, NamedPin)>,
}

impl IterLogicCircuit {
    fn usable_inpins(&self) -> Vec<(usize, Pin)> {
        self.used
            .iter()
            .enumerate()
            .flat_map(|(n, g)| {
                g.get_inpins()
                    .into_iter()
                    .map(|(_, pin)| pin)
                    .filter(move |pin| {
                        (n == 0 || self.next_edges.iter().all(|(_, i2)| pin != i2))
                            && self.prev_edges.iter().all(|(_, i2)| pin != i2)
                    })
                    .map(move |pin| (n, pin))
            })
            .collect()
    }
    fn usable_otpins(&self) -> Vec<(usize, Pin)> {
        self.used
            .iter()
            .enumerate()
            .flat_map(|(n, g)| {
                g.get_otpins()
                    .into_iter()
                    .map(|(_, pin)| pin)
                    .filter(move |pin| {
                        (n == 0 || self.prev_edges.iter().all(|(o2, _)| pin != o2))
                            && self.next_edges.iter().all(|(o2, _)| pin != o2)
                    })
                    .map(move |pin| (n, pin))
            })
            .collect()
    }

    fn map_internal_inpin(&self, internal: &NamedPin) -> NamedPin {
        if let Some((external, _)) = self
            .inpin_maps
            .iter()
            .find(|(_, target)| target == internal)
        {
            make_pin(self.kind.clone(), external.clone())
        } else {
            internal.clone()
        }
    }

    fn map_internal_otpin(&self, internal: &NamedPin) -> NamedPin {
        if let Some((external, _)) = self
            .otpin_maps
            .iter()
            .find(|(_, target)| target == internal)
        {
            make_pin(self.kind.clone(), external.clone())
        } else {
            internal.clone()
        }
    }

    fn resolve_input(&self, input: NamedPin) -> Option<NamedPin> {
        if input.0 == self.kind {
            self.inpin_maps
                .iter()
                .find(|(external, _)| external == &input.1)
                .map(|(_, target)| target.clone())
        } else {
            Some(input)
        }
    }
}

impl LogicCircuitTrait for IterLogicCircuit {
    fn kind(&self) -> Identifier {
        self.kind.clone()
    }

    fn get_inpins(&self) -> Vec<NamedPin> {
        self.usable_inpins()
            .into_iter()
            .map(|(n, i)| self.map_internal_inpin(&make_pin(num_to_ident(n), i)))
            .collect()
    }

    fn get_otpins(&self) -> Vec<NamedPin> {
        self.usable_otpins()
            .into_iter()
            .map(|(n, o)| self.map_internal_otpin(&make_pin(num_to_ident(n), o)))
            .collect()
    }

    fn get_otputs(&self) -> Vec<(NamedPin, Bool)> {
        self.used
            .iter()
            .enumerate()
            .flat_map(|(n, g)| {
                g.get_otputs()
                    .into_iter()
                    .map(|(o, b)| (make_pin(num_to_ident(n), pin_name(&o).clone()), b))
                    .filter(move |(o, _)| {
                        (n == 0 || self.prev_edges.iter().all(|(o2, _)| &o.1 != o2))
                            && self.next_edges.iter().all(|(o2, _)| &o.1 != o2)
                    })
                    .map(|(o, b)| (self.map_internal_otpin(&o), b))
                    .collect::<Vec<_>>()
            })
            .collect()
    }

    fn step(&mut self, inputs: Vec<(NamedPin, Bool)>) {
        // inputs for each Logic Circuits (key by index)
        // initialized
        let mut new_inputs: Vec<Vec<(NamedPin, Bool)>> = vec![vec![]; self.used.len() + 1];

        // from other Logic Circuits
        // priority is high
        for (num, lc) in self.used.iter().enumerate() {
            for (o, b) in lc.get_otputs() {
                let output_pin = pin_name(&o);
                // send inputs to next Logic Circuits
                if let Some(i) =
                    self.next_edges
                        .iter()
                        .find_map(|(o2, i)| if o2 == output_pin { Some(i) } else { None })
                {
                    new_inputs[num + 1].push((make_pin(num_to_ident(num + 1), i.clone()), b));
                }
                // send inputs to previous Logic Circuits
                if let Some(i) =
                    self.prev_edges
                        .iter()
                        .find_map(|(o2, i)| if o2 == output_pin { Some(i) } else { None })
                {
                    new_inputs[num - 1].push((make_pin(num_to_ident(num - 1), i.clone()), b));
                }
            }
        }

        // from inputs
        // priority is low => check if already used
        for (i, b) in inputs {
            let Some(internal) = self.resolve_input(i) else {
                eprintln!("Invalid InPin");
                continue;
            };
            let Ok(num) = internal.0.as_ref().parse::<usize>() else {
                eprint!("ignored input {} because not a number", internal.0);
                continue;
            };
            let v = &mut new_inputs[num];
            if v.iter().all(|(i2, _)| i2 != &internal) {
                v.push((internal, b));
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
                            make_pin(num_to_ident(i), otpin.clone()),
                            make_pin(num_to_ident(i + 1), inpin.clone()),
                        ));
                    }
                }
                if i > 0 {
                    for (otpin, inpin) in &self.prev_edges {
                        edges.push((
                            make_pin(num_to_ident(i), otpin.clone()),
                            make_pin(num_to_ident(i - 1), inpin.clone()),
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
            inpins_map: self
                .usable_inpins()
                .into_iter()
                .map(|(n, i)| {
                    let internal = make_pin(num_to_ident(n), i);
                    let external = self.map_internal_inpin(&internal);
                    (external, internal)
                })
                .collect(),
            otpins_map: self
                .usable_otpins()
                .into_iter()
                .map(|(n, o)| {
                    let internal = make_pin(num_to_ident(n), o);
                    let external = self.map_internal_otpin(&internal);
                    (external, internal)
                })
                .collect(),
        }
    }
}
