use anyhow::{anyhow, bail, Result};
use either::Either;
use std::{collections::HashMap, fmt::Display};
use utils::{bool::Bool, number::*};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct InPin(String);

impl From<String> for InPin {
    fn from(value: String) -> Self {
        InPin(value)
    }
}

impl From<&str> for InPin {
    fn from(value: &str) -> Self {
        InPin(value.to_string())
    }
}

impl Display for InPin {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct OtPin(String);

impl From<String> for OtPin {
    fn from(value: String) -> Self {
        OtPin(value)
    }
}

impl From<&str> for OtPin {
    fn from(value: &str) -> Self {
        OtPin(value.to_string())
    }
}

impl Display for OtPin {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Gate {
    Cst {
        state: Bool,
    },
    Not {
        state: Bool,
        input: Bool,
    },
    And {
        state: Bool,
        input0: Bool,
        input1: Bool,
    },
    Or {
        state: Bool,
        input0: Bool,
        input1: Bool,
    },
    Br {
        state: Bool,
        input: Bool,
    },
    Delay {
        input: Bool,
        state: Bool,
    },
    End {
        input: Bool,
    },
}

impl Gate {
    pub fn state(&self) -> &Bool {
        match self {
            Gate::Cst { state } => state,
            Gate::Not { state, input: _ } => state,
            Gate::Br { state, input: _ } => state,
            Gate::End { input } => input,
            Gate::And {
                state,
                input0: _,
                input1: _,
            } => state,
            Gate::Or {
                state,
                input0: _,
                input1: _,
            } => state,
            Gate::Delay { input: _, state } => state,
        }
    }
    pub fn get_input(&self, input_name: &InPin) -> Option<&Bool> {
        match (self, input_name.0.as_str()) {
            (Gate::Not { state: _, input }, "IN") => Some(input),
            (Gate::Br { state: _, input }, "IN") => Some(input),
            (Gate::End { input }, "IN") => Some(input),
            (
                Gate::And {
                    state: _,
                    input0,
                    input1: _,
                },
                "IN0",
            ) => Some(input0),
            (
                Gate::And {
                    state: _,
                    input0: _,
                    input1,
                },
                "IN1",
            ) => Some(input1),
            (
                Gate::Or {
                    state: _,
                    input0,
                    input1: _,
                },
                "IN0",
            ) => Some(input0),
            (
                Gate::Or {
                    state: _,
                    input0: _,
                    input1,
                },
                "IN1",
            ) => Some(input1),
            (Gate::Delay { input, state: _ }, "IN") => Some(input),
            _ => None,
        }
    }
    fn getmut_input(&mut self, inpin: &InPin) -> Option<&mut Bool> {
        match (self, inpin.0.as_str()) {
            (Gate::Not { state: _, input }, "IN") => Some(input),
            (Gate::Br { state: _, input }, "IN") => Some(input),
            (Gate::End { input }, "IN") => Some(input),
            (
                Gate::And {
                    state: _,
                    input0,
                    input1: _,
                },
                "IN0",
            ) => Some(input0),
            (
                Gate::And {
                    state: _,
                    input0: _,
                    input1,
                },
                "IN1",
            ) => Some(input1),
            (
                Gate::Or {
                    state: _,
                    input0,
                    input1: _,
                },
                "IN0",
            ) => Some(input0),
            (
                Gate::Or {
                    state: _,
                    input0: _,
                    input1,
                },
                "IN1",
            ) => Some(input1),
            (Gate::Delay { input, state: _ }, "IN") => Some(input),
            _ => None,
        }
    }
    pub fn get_output(&self, otpin: &OtPin) -> Option<&Bool> {
        match (self, otpin.0.as_str()) {
            (Gate::Not { state, input: _ }, "OUT") => Some(state),
            (Gate::Cst { state }, "OUT") => Some(state),
            (Gate::Br { state, input: _ }, "OUT0") => Some(state),
            (Gate::Br { state, input: _ }, "OUT1") => Some(state),
            (Gate::End { input: _ }, _) => None,
            (
                Gate::And {
                    state,
                    input0: _,
                    input1: _,
                },
                "OUT",
            ) => Some(state),
            (
                Gate::Or {
                    state,
                    input0: _,
                    input1: _,
                },
                "OUT",
            ) => Some(state),
            (Gate::Delay { input: _, state }, "OUT") => Some(state),
            _ => None,
        }
    }
    pub fn next(&mut self) {
        match self {
            Gate::Not { state, input } => {
                *state = !*input;
            }
            Gate::Br { state, input } => {
                *state = *input;
            }
            Gate::Delay { input, state } => {
                *state = *input;
            }
            Gate::And {
                state,
                input0,
                input1,
            } => {
                *state = input0.and(*input1);
            }
            Gate::Or {
                state,
                input0,
                input1,
            } => {
                *state = input0.or(*input1);
            }
            _ => {}
        }
    }
    pub fn name(&self) -> String {
        match self {
            Gate::Not { state: _, input: _ } => "not".to_owned(),
            Gate::And {
                state: _,
                input0: _,
                input1: _,
            } => "and".to_owned(),
            Gate::Or {
                state: _,
                input0: _,
                input1: _,
            } => "or ".to_owned(),
            Gate::Cst { state } => format!("cst{state}"),
            Gate::Br { state: _, input: _ } => "br".to_owned(),
            Gate::End { input: _ } => "end".to_owned(),
            Gate::Delay { input: _, state: _ } => "dly".to_owned(),
        }
    }
    pub fn get_inpins(&self) -> Vec<(InPin, Bool)> {
        match self {
            Gate::Not { state: _, input }
            | Gate::Br { state: _, input }
            | Gate::Delay { input, state: _ }
            | Gate::End { input } => vec![("IN".into(), *input)],
            Gate::And {
                state: _,
                input0,
                input1,
            }
            | Gate::Or {
                state: _,
                input0,
                input1,
            } => vec![("IN0".into(), *input0), ("IN1".into(), *input1)],
            Gate::Cst { state: _ } => vec![],
        }
    }
    pub fn get_otpins(&self) -> Vec<(OtPin, Bool)> {
        match self {
            Gate::Not { state, input: _ }
            | Gate::Delay { input: _, state }
            | Gate::And {
                state,
                input0: _,
                input1: _,
            }
            | Gate::Or {
                state,
                input0: _,
                input1: _,
            }
            | Gate::Cst { state } => vec![("OUT".into(), *state)],
            Gate::Br { state, input: _ } => vec![("OUT0".into(), *state), ("OUT1".into(), *state)],
            Gate::End { input: _ } => vec![],
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Name(String);

impl From<String> for Name {
    fn from(value: String) -> Self {
        Name(value)
    }
}

impl From<&str> for Name {
    fn from(value: &str) -> Self {
        Name(value.to_string())
    }
}

impl Display for Name {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FinGraph {
    pub lcs: Vec<(Name, LoC)>,
    pub edges: Vec<((Name, OtPin), (Name, InPin))>,
    pub input: Vec<(InPin, (Name, InPin))>,
    pub otput: Vec<(OtPin, (Name, OtPin))>,
}

impl FinGraph {
    fn new(
        mut lcs: Vec<(Name, LoC)>,
        edges: Vec<((Name, OtPin), (Name, InPin))>,
        input: Vec<(InPin, (Name, InPin))>,
        otput: Vec<(OtPin, (Name, OtPin))>,
    ) -> Result<Self> {
        let mut unused_inpins: HashMap<Name, Vec<(InPin, bool)>> = HashMap::new();
        let mut unused_otpins: HashMap<Name, Vec<(OtPin, bool)>> = HashMap::new();

        for (n, lc) in lcs.iter() {
            let inpins = unused_inpins.entry(n.clone()).or_default();
            inpins.extend(lc.get_inpins().iter().map(|i| (i.0.clone(), true)));

            let otpins = unused_otpins.entry(n.clone()).or_default();
            otpins.extend(lc.get_otpins().iter().map(|o| (o.0.clone(), true)));
        }

        let mut check_used_name_and_inpin = |(name, inpin): &(Name, InPin)| -> Result<()> {
            let Some(inpins) = unused_inpins.get_mut(name) else {
                bail!("not found lc named {name}")
            };
            let Some(b) = inpins
                .iter_mut()
                .find_map(|(i, b)| if i == inpin { Some(b) } else { None })
            else {
                bail!("not found inpin: {inpin} in name: {name}")
            };
            if !*b {
                bail!("already used name: {name} inpin: {inpin}")
            }
            *b = false;
            Ok(())
        };

        let mut check_used_name_and_otpin = |(name, otpin): &(Name, OtPin)| -> Result<()> {
            let Some(otpins) = unused_otpins.get_mut(name) else {
                bail!("not found lc named {name}")
            };
            let Some(b) = otpins
                .iter_mut()
                .find_map(|(o, b)| if o == otpin { Some(b) } else { None })
            else {
                bail!("not found otpin: {otpin} in name: {name}")
            };
            if !*b {
                bail!("already used name: {name} otpin: {otpin}")
            }
            *b = false;
            Ok(())
        };

        for (no, ni) in edges.iter() {
            check_used_name_and_otpin(no)?;
            check_used_name_and_inpin(ni)?;
        }
        for ni in input.iter() {
            check_used_name_and_inpin(&ni.1)?;
        }
        for no in otput.iter() {
            check_used_name_and_otpin(&no.1)?;
        }

        for (n, v) in unused_inpins {
            if let Some((i, _)) = v.into_iter().find(|(_, b)| *b) {
                bail!("unused inpins in name: {n} inpin: {i}")
            }
        }

        for (n, v) in unused_otpins {
            if let Some((o, _)) = v.into_iter().find(|(_, b)| *b) {
                bail!("unused inpins in name: {n} otpin: {o}")
            }
        }

        for (no, ni) in edges.iter() {
            let ob: Bool = *lcs
                .iter_mut()
                .find_map(|(n, lc)| if n == &no.0 { Some(lc) } else { None })
                .unwrap()
                .get_otput(&no.1)
                .unwrap();
            let ib: &mut Bool = lcs
                .iter_mut()
                .find_map(|(n, lc)| if n == &ni.0 { Some(lc) } else { None })
                .unwrap()
                .getmut_input(&ni.1)
                .unwrap();
            *ib = ob;
        }

        Ok(Self {
            lcs,
            edges,
            input,
            otput,
        })
    }
    pub fn get_input(&self, inpin: &InPin) -> Option<&Bool> {
        let (_, (name, inpin)) = self.input.iter().find(|(i, _)| i == inpin)?;
        let (_, lc) = self.lcs.iter().find(|(name2, _)| name2 == name)?;
        lc.get_input(inpin)
    }
    pub fn getmut_input(&mut self, inpin: &InPin) -> Option<&mut Bool> {
        let (_, (name, inpin)) = self.input.iter_mut().find(|(i, _)| i == inpin)?;
        let (_, lc) = self.lcs.iter_mut().find(|(name2, _)| name2 == name)?;
        lc.getmut_input(inpin)
    }
    pub fn get_otput(&self, otpin: &OtPin) -> Option<&Bool> {
        let (_, (name, otpin)) = self.otput.iter().find(|(i, _)| i == otpin)?;
        let (_, lc) = self.lcs.iter().find(|(name2, _)| name2 == name)?;
        lc.get_otput(otpin)
    }
    pub fn getmut_lc(&mut self, name: &Name) -> Option<&mut LoC> {
        let (_, lc) = self.lcs.iter_mut().find(|(n, _)| name == n)?;
        Some(lc)
    }
    pub fn get_lc(&self, name: &Name) -> Option<&LoC> {
        let (_, lc) = self.lcs.iter().find(|(n, _)| name == n)?;
        Some(lc)
    }
    pub fn next(&mut self) {
        for (_, lc) in &mut self.lcs {
            lc.next();
        }
        // lc 同士の整合性
        for ((n0, o), (n1, i)) in self.edges.clone() {
            let lco = *self.get_lc(&n0).unwrap().get_otput(&o).unwrap();
            let lci = self.getmut_lc(&n1).unwrap().getmut_input(&i).unwrap();
            *lci = lco;
        }
    }
    pub fn get_lc_names(&self) -> Vec<Name> {
        self.lcs.iter().map(|(n, _)| n.clone()).collect()
    }
    pub fn get_inpins(&self) -> Vec<(InPin, Bool)> {
        self.input
            .iter()
            .map(|i| (i.0.clone(), *self.get_input(&i.0).unwrap()))
            .collect()
    }
    pub fn get_inpins_of_lc(&self, name: &Name) -> Option<Vec<(InPin, Bool)>> {
        let lc = self.get_lc(name)?;
        Some(lc.get_inpins())
    }
    pub fn get_otpins(&self) -> Vec<(OtPin, Bool)> {
        self.otput
            .iter()
            .map(|o| (o.0.clone(), *self.get_otput(&o.0).unwrap()))
            .collect()
    }
    pub fn get_otpins_of_lc(&self, name: &Name) -> Option<Vec<(OtPin, Bool)>> {
        let lc = self.get_lc(name)?;
        Some(lc.get_otpins())
    }
    pub fn edges(&self) -> &Vec<((Name, OtPin), (Name, InPin))> {
        &self.edges
    }
    pub fn get_inpin_to_lc_inpin(&self, inpin: &InPin) -> Option<(Name, InPin)> {
        self.input
            .iter()
            .find(|(i, _)| i == inpin)
            .map(|v| v.1.clone())
    }
    pub fn get_otpin_to_lc_otpin(&self, otpin: &OtPin) -> Option<(Name, OtPin)> {
        self.otput
            .iter()
            .find(|(o, _)| o == otpin)
            .map(|v| v.1.clone())
    }
    pub fn get_lc_inpins(&self, name: &Name) -> Vec<(InPin, Name, OtPin, Bool)> {
        self.edges
            .iter()
            .filter_map(|((no, o), (ni, i))| {
                if name == ni {
                    let s = self.get_input(i).unwrap();
                    Some((i.clone(), no.clone(), o.clone(), *s))
                } else {
                    None
                }
            })
            .collect()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Iter {
    lc_init: Box<LoC>,
    lc_extended: Vec<LoC>,
    next_edges: Vec<(OtPin, InPin)>,
    prev_edges: Vec<(OtPin, InPin)>,
}

impl Iter {
    fn new(
        lc: LoC,
        next_edges: Vec<(OtPin, InPin)>,
        prev_edges: Vec<(OtPin, InPin)>,
    ) -> Result<Self> {
        let mut unused_inpin: Vec<(InPin, bool)> =
            lc.get_inpins().into_iter().map(|i| (i.0, true)).collect();
        let mut unused_otpin: Vec<(OtPin, bool)> =
            lc.get_otpins().into_iter().map(|o| (o.0, true)).collect();
        for (otpin, inpin) in next_edges.iter() {
            let Some((_, b)) = unused_otpin.iter_mut().find(|(o, _)| o == otpin) else {
                bail!("not found otpin: {otpin}");
            };
            if !*b {
                bail!("already used otpin: {otpin}");
            }
            *b = false;

            let Some((_, b)) = unused_inpin.iter_mut().find(|(i, _)| i == inpin) else {
                bail!("not found inpin: {inpin}");
            };
            if !*b {
                bail!("already used inpin: {inpin}");
            }
            *b = false;
        }

        for (i, b) in unused_inpin {
            if b {
                bail!("unused inpin: {i}");
            }
        }

        for (o, b) in unused_otpin {
            if b {
                bail!("unused otpin: {o}");
            }
        }

        Ok(Self {
            lc_init: Box::new(lc.clone()),
            lc_extended: vec![lc],
            next_edges: next_edges.into_iter().collect(),
            prev_edges: prev_edges.into_iter().collect(),
        })
    }
    pub fn get_input(&self, inpin: &InPin) -> Option<&Bool> {
        let inpin = self.get_inpins().into_iter().find(|i| i.0 == *inpin)?;
        self.lc_extended[0].get_input(&inpin.0)
    }
    pub fn getmut_input(&mut self, inpin: &InPin) -> Option<&mut Bool> {
        let inpin = self.get_inpins().into_iter().find(|i| i.0 == *inpin)?;
        self.lc_extended[0].getmut_input(&inpin.0)
    }
    pub fn get_otput(&self, otpin: &OtPin) -> Option<&Bool> {
        let otpin = self.get_otpins().into_iter().find(|o| o.0 == *otpin)?;
        self.lc_extended[0].get_otput(&otpin.0)
    }
    pub fn getmut_lc(&mut self, n: Number) -> Option<&mut LoC> {
        let n: usize = n.into();
        self.lc_extended.get_mut(n)
    }
    fn get_lc(&self, n: Number) -> Option<&LoC> {
        let n: usize = n.into();
        self.lc_extended.get(n)
    }
    pub fn get_lcs(&self) -> &Vec<LoC> {
        &self.lc_extended
    }
    pub fn next(&mut self) {
        for l in &mut self.lc_extended {
            l.next();
        }
        let n = self.lc_extended.len();
        // next との整合性
        self.lc_extended.push(self.lc_init.as_ref().clone());
        let mut b = true;
        for (o, i) in self.next_edges.iter() {
            for l in 0..n {
                let o = *self.lc_extended[l].get_otput(o).unwrap();
                let i = self.lc_extended[l + 1].getmut_input(i).unwrap();
                *i = o;
                if l == n - 1 && o == Bool::T {
                    b = false;
                }
            }
        }
        if b {
            self.lc_extended.pop();
        }

        // prev との整合性
        for (o, i) in self.prev_edges.iter() {
            for l in 1..n {
                let o = *self.lc_extended[l].get_otput(o).unwrap();
                let i = self.lc_extended[l - 1].getmut_input(i).unwrap();
                *i = o;
            }
        }
    }
    pub fn get_inpins(&self) -> Vec<(InPin, Bool)> {
        self.next_edges
            .iter()
            .map(|(_, i)| (i.clone(), *self.get_input(&i).unwrap()))
            .collect()
    }
    pub fn get_otpins(&self) -> Vec<(OtPin, Bool)> {
        self.prev_edges
            .iter()
            .map(|(o, _)| (o.clone(), *self.get_otput(&o).unwrap()))
            .collect()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LoC {
    Gate(Gate),
    FinGraph(Name, Box<FinGraph>),
    Iter(Name, Iter),
}

type Path = Vec<Either<Name, Number>>;
pub fn into_inpin_path(str: &str) -> Path {
    let p: Vec<_> = str
        .split('.')
        .map(|s| match s.parse::<usize>() {
            Ok(n) => Either::Right(n.into()),
            Err(_) => Either::Left(s.into()),
        })
        .collect();
    p
}

impl LoC {
    pub fn notgate(b: Bool) -> LoC {
        LoC::Gate(Gate::Not {
            state: b,
            input: Bool::F,
        })
    }
    pub fn andgate(b: Bool) -> LoC {
        LoC::Gate(Gate::And {
            state: b,
            input0: Bool::F,
            input1: Bool::F,
        })
    }
    pub fn orgate(b: Bool) -> LoC {
        LoC::Gate(Gate::Or {
            state: b,
            input0: Bool::F,
            input1: Bool::F,
        })
    }
    pub fn cstgate(b: Bool) -> LoC {
        LoC::Gate(Gate::Cst { state: b })
    }
    pub fn brgate(b: Bool) -> LoC {
        LoC::Gate(Gate::Br {
            state: b,
            input: Bool::F,
        })
    }
    pub fn endgate() -> LoC {
        LoC::Gate(Gate::End { input: Bool::F })
    }
    pub fn delaygate(b: Bool) -> LoC {
        LoC::Gate(Gate::Delay {
            input: Bool::F,
            state: b,
        })
    }
    pub fn new_graph(
        name: Name,
        lcs: Vec<(Name, LoC)>,
        edges: Vec<((Name, OtPin), (Name, InPin))>,
        input: Vec<(InPin, (Name, InPin))>,
        output: Vec<(OtPin, (Name, OtPin))>,
    ) -> Result<Self> {
        Ok(LoC::FinGraph(
            name,
            Box::new(FinGraph::new(lcs, edges, input, output)?),
        ))
    }
    pub fn new_iter(
        name: Name,
        lc: LoC,
        next_edges: Vec<(OtPin, InPin)>,
        prev_edges: Vec<(OtPin, InPin)>,
    ) -> Result<Self> {
        Ok(LoC::Iter(name, Iter::new(lc, next_edges, prev_edges)?))
    }
    pub fn get_name(&self) -> Name {
        match self {
            LoC::Gate(gate) => gate.name().into(),
            LoC::FinGraph(name, _) => name.clone(),
            LoC::Iter(name, _) => name.clone(),
        }
    }
    pub fn get_input(&self, inpin: &InPin) -> Option<&Bool> {
        match self {
            LoC::Gate(gate) => gate.get_input(inpin),
            LoC::FinGraph(_, fingraph) => fingraph.get_input(inpin),
            LoC::Iter(_, iter) => iter.get_input(inpin),
        }
    }
    pub fn getmut_input(&mut self, inpin: &InPin) -> Option<&mut Bool> {
        match self {
            LoC::Gate(gate) => gate.getmut_input(inpin),
            LoC::FinGraph(_, fingraph) => fingraph.getmut_input(inpin),
            LoC::Iter(_, iter) => iter.getmut_input(inpin),
        }
    }
    pub fn get_otput(&self, otpin: &OtPin) -> Option<&Bool> {
        match self {
            LoC::Gate(gate) => gate.get_output(otpin),
            LoC::FinGraph(_, fingraph) => fingraph.get_otput(otpin),
            LoC::Iter(_, iter) => iter.get_otput(otpin),
        }
    }
    pub fn getmut_lc_from_path(&mut self, path: &Path) -> Option<&mut LoC> {
        let mut lc = self;
        for name in path {
            match (lc, name) {
                (LoC::FinGraph(_, fingraph), Either::Left(name)) => {
                    lc = fingraph.getmut_lc(name)?;
                }
                (LoC::Iter(_, iter), Either::Right(num)) => {
                    lc = iter.getmut_lc(num.clone())?;
                }
                _ => {
                    return None;
                }
            }
        }
        Some(lc)
    }
    pub fn get_lc_from_path(&self, path: &Path) -> Option<&LoC> {
        let mut lc = self;
        for name in path {
            match (lc, name) {
                (LoC::FinGraph(_, fingraph), Either::Left(name)) => {
                    lc = fingraph.get_lc(name)?;
                }
                (LoC::Iter(_, iter), Either::Right(num)) => {
                    lc = iter.get_lc(num.clone())?;
                }
                _ => {
                    return None;
                }
            }
        }
        Some(lc)
    }
    pub fn get_state_of_gate_from_path(&self, path: &Path) -> Option<&Bool> {
        let lc = self.get_lc_from_path(path)?;
        let LoC::Gate(gate) = lc else {
            return None;
        };
        Some(gate.state())
    }
    pub fn next(&mut self) {
        match self {
            LoC::Gate(gate) => gate.next(),
            LoC::FinGraph(_, fingraph) => fingraph.next(),
            LoC::Iter(_, iter) => iter.next(),
        }
    }
    pub fn get_inpins(&self) -> Vec<(InPin, Bool)> {
        match self {
            LoC::Gate(gate) => gate.get_inpins(),
            LoC::FinGraph(_, fingraph) => fingraph.get_inpins(),
            LoC::Iter(_, iter) => iter.get_inpins(),
        }
    }
    pub fn get_otpins(&self) -> Vec<(OtPin, Bool)> {
        match self {
            LoC::Gate(gate) => gate.get_otpins(),
            LoC::FinGraph(_, fingraph) => fingraph.get_otpins(),
            LoC::Iter(_, iter) => iter.get_otpins(),
        }
    }
    pub fn take_fingraph(self) -> Option<FinGraph> {
        match self {
            LoC::FinGraph(name, fingraph) => Some(*fingraph),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn gate_test() {
        let mut notgate = Gate::Not {
            state: Bool::F,
            input: Bool::F,
        };
        notgate.next();
        eprintln!("{notgate:?}");
    }
    #[test]
    fn rsratch() {
        let rs = LoC::new_graph(
            "RS-latch".into(),
            vec![
                ("O0".into(), LoC::orgate(Bool::T)),
                ("N0".into(), LoC::notgate(Bool::F)),
                ("B0".into(), LoC::brgate(Bool::F)),
                ("O1".into(), LoC::orgate(Bool::F)),
                ("N1".into(), LoC::notgate(Bool::T)),
                ("B1".into(), LoC::brgate(Bool::T)),
            ],
            vec![
                (("O0".into(), "OUT".into()), ("N0".into(), "IN".into())),
                (("O1".into(), "OUT".into()), ("N1".into(), "IN".into())),
                (("N0".into(), "OUT".into()), ("B0".into(), "IN".into())),
                (("N1".into(), "OUT".into()), ("B1".into(), "IN".into())),
                (("B0".into(), "OUT1".into()), ("O1".into(), "IN1".into())),
                (("B1".into(), "OUT1".into()), ("O0".into(), "IN1".into())),
            ],
            vec![
                ("R".into(), ("O0".into(), "IN0".into())),
                ("S".into(), ("O1".into(), "IN0".into())),
            ],
            vec![
                ("Q".into(), ("B0".into(), "OUT0".into())),
                ("nQ".into(), ("B1".into(), "OUT0".into())),
            ],
        );
        let mut rs = rs.unwrap();

        let a = rs.get_inpins();
        assert_eq!(a, vec![("R".into(), Bool::F), ("S".into(), Bool::F)]);

        let t = |lc: &mut LoC| loop {
            let lc_prev = lc.clone();
            lc.next();
            if lc_prev == *lc {
                break;
            }
        };

        let rsp = rs.clone();
        rs.next();
        assert_eq!(rsp, rs);

        let r = rs.getmut_input(&"R".into()).unwrap();
        *r = Bool::T;
        t(&mut rs);
        println!("---");

        let r = rs.getmut_input(&"R".into()).unwrap();
        *r = Bool::F;
        let r = rs.getmut_input(&"S".into()).unwrap();
        *r = Bool::T;
        t(&mut rs);
    }
    #[test]
    fn inf_dff() {}
}
