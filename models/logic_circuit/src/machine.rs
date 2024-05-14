use anyhow::{anyhow, bail, Result};
use either::Either;
use pest::pratt_parser::Op;
use std::str::FromStr;
use std::{
    collections::{HashMap, HashSet},
    fmt::Display,
    ops::Neg,
};
use utils::number::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Bool {
    T,
    F,
}

impl Neg for Bool {
    type Output = Bool;
    fn neg(self) -> Self::Output {
        match self {
            Bool::T => Bool::F,
            Bool::F => Bool::T,
        }
    }
}

impl Bool {
    pub fn and(self, other: Self) -> Self {
        match (self, other) {
            (Bool::T, Bool::T) => Bool::T,
            _ => Bool::F,
        }
    }
    pub fn or(self, other: Self) -> Self {
        match (self, other) {
            (Bool::F, Bool::F) => Bool::F,
            _ => Bool::T,
        }
    }
}

impl FromStr for Bool {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "T" => Ok(Bool::T),
            "F" => Ok(Bool::F),
            _ => Err(anyhow!("fail to parse {s}")),
        }
    }
}

impl Display for Bool {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Bool::T => write!(f, "T"),
            Bool::F => write!(f, "F"),
        }
    }
}

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
    fn next(&mut self) {
        match self {
            Gate::Not { state, input } => {
                *state = input.neg();
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
            Gate::Br { state: _, input: _ } => "br ".to_owned(),
            Gate::End { input: _ } => "end".to_owned(),
            Gate::Delay { input: _, state: _ } => "dly".to_owned(),
        }
    }
    pub fn get_all_input_name(&self) -> Vec<InPin> {
        match self {
            Gate::Not { state: _, input: _ }
            | Gate::Br { state: _, input: _ }
            | Gate::Delay { input: _, state: _ } => vec!["IN".into()],
            Gate::And {
                state: _,
                input0: _,
                input1: _,
            }
            | Gate::Or {
                state: _,
                input0: _,
                input1: _,
            } => vec!["IN0".into(), "IN1".into()],
            _ => vec![],
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
    pub name: Name,
    pub lcs: HashMap<Name, LoC>,
    pub edges: HashSet<((Name, OtPin), (Name, InPin))>,
    pub input: HashMap<InPin, (Name, InPin)>,
    pub output: HashMap<OtPin, (Name, OtPin)>,
}

impl FinGraph {
    fn new(
        name: Name,
        lcs: Vec<(Name, LoC)>,
        edges: Vec<((Name, OtPin), (Name, InPin))>,
        input: Vec<(InPin, (Name, InPin))>,
        output: Vec<(OtPin, (Name, OtPin))>,
    ) -> Result<Self> {
        let mut lcs: HashMap<Name, LoC> = lcs.into_iter().map(|(a, b)| (a.clone(), b)).collect();
        let mut new_edges: HashSet<((Name, OtPin), (Name, InPin))> = HashSet::new();
        let mut new_input = HashMap::new();
        let mut new_output = HashMap::new();
        for ((n0, o), (n1, i)) in edges {
            let Some(n0_lc) = lcs.get(&n0) else {
                bail!("fail {name} not found name {n0}")
            };
            let Some(&ob) = n0_lc.get_output(&o) else {
                bail!("fail {name} not found outpin {o} in {n0}")
            };

            let Some(n1_lc) = lcs.get_mut(&n1) else {
                bail!("fail {name} not found name {n1}")
            };

            let Some(ib) = n1_lc.getmut_input(&i) else {
                bail!("fail {name} not found inpin {i} in {n1}")
            };
            *ib = ob;
            new_edges.insert(((n0.clone(), o.clone()), (n1.clone(), i.clone())));
        }
        for (i, (n, i0)) in input {
            let Some(nlc) = lcs.get(&n) else {
                bail!("fail {name} not found {n}")
            };
            if nlc.get_input(&i0).is_none() {
                bail!("fail {name} not found inpin {i0} in {n}")
            }
            new_input.insert(i, (n, i0));
        }
        for (o, (n, o0)) in output {
            let Some(nlc) = lcs.get(&n) else {
                bail!("fail {name} not found {n}")
            };
            if nlc.get_output(&o0).is_none() {
                bail!("fail {name} not found inpin {o0} in {n}")
            }
            new_output.insert(o, (n, o0));
        }
        Ok(Self {
            name,
            lcs,
            edges: new_edges,
            input: new_input
                .into_iter()
                .map(|(a, (b, c))| (a.clone(), (b.clone(), c.clone())))
                .collect(),
            output: new_output
                .into_iter()
                .map(|(a, (b, c))| (a.clone(), (b.clone(), c.clone())))
                .collect(),
        })
    }
    pub fn get_input(&self, inpin: &InPin) -> Option<&Bool> {
        let (name, inpin) = self.input.get(inpin)?;
        let lc = self.lcs.get(name)?;
        lc.get_input(inpin)
    }
    fn getmut_input(&mut self, inpin: &InPin) -> Option<&mut Bool> {
        let (name, inpin) = self.input.get(inpin)?;
        let lc = self.lcs.get_mut(name)?;
        lc.getmut_input(inpin)
    }
    pub fn get_output(&self, otpin: &OtPin) -> Option<&Bool> {
        let (name, otpin) = self.output.get(otpin)?;
        let lc = self.lcs.get(name)?;
        lc.get_output(otpin)
    }
    pub fn getmut_lc(&mut self, name: &Name) -> Option<&mut LoC> {
        self.lcs.get_mut(name)
    }
    pub fn get_lc(&self, name: &Name) -> Option<&LoC> {
        self.lcs.get(name)
    }
    fn next(&mut self) {
        for lc in self.lcs.values_mut() {
            lc.next();
        }
        for ((n0, o), (n1, i)) in self.edges.clone() {
            let lco = *self.get_lc(&n0).unwrap().get_output(&o).unwrap();
            let lci = self.getmut_lc(&n1).unwrap().getmut_input(&i).unwrap();
            *lci = lco;
        }
    }
    pub fn get_lc_inouts(&self, name: &Name) -> Option<(&LoC, Vec<(InPin, (Name, OtPin), Bool)>)> {
        let lc = self.lcs.get(name)?;
        let mut inout = vec![];
        for ((n0, o0), (n1, i1)) in self.edges.iter() {
            if n1 == name {
                let lc = self.get_lc(name).unwrap();
                let b = *lc.get_input(i1).unwrap();
                inout.push((i1.clone(), (n0.clone(), o0.clone()), b));
            }
        }
        Some((lc, inout))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Iter {
    pub name: Name,
    pub lc_init: Box<LoC>,
    pub lc_extended: Vec<LoC>,
    pub next_edges: HashSet<(OtPin, InPin)>,
    pub prev_edges: HashSet<(OtPin, InPin)>,
    pub input: HashMap<InPin, InPin>,
    pub otput: HashMap<OtPin, OtPin>,
}

impl Iter {
    fn new(
        name: Name,
        lc: LoC,
        next_edges: Vec<(OtPin, InPin)>,
        prev_edges: Vec<(OtPin, InPin)>,
        input: Vec<(InPin, InPin)>,
        otput: Vec<(OtPin, OtPin)>,
    ) -> Result<Self> {
        Ok(Self {
            name,
            lc_init: Box::new(lc.clone()),
            lc_extended: vec![lc],
            next_edges: next_edges.into_iter().collect(),
            prev_edges: prev_edges.into_iter().collect(),
            input: input.into_iter().collect(),
            otput: otput.into_iter().collect(),
        })
    }
    pub fn get_input(&self, inpin: &InPin) -> Option<&Bool> {
        let inpin = self.input.get(inpin)?;
        self.lc_extended[0].get_input(inpin)
    }
    fn getmut_input(&mut self, inpin: &InPin) -> Option<&mut Bool> {
        let inpin = self.input.get_mut(inpin)?;
        self.lc_extended[0].getmut_input(inpin)
    }
    pub fn get_otput(&self, otpin: &OtPin) -> Option<&Bool> {
        let otpin = self.otput.get(otpin)?;
        self.lc_extended[0].get_output(otpin)
    }
    pub fn getmut_lc(&mut self, n: Number) -> Option<&mut LoC> {
        let n: usize = n.into();
        self.lc_extended.get_mut(n)
    }
    fn get_lc(&self, n: Number) -> Option<&LoC> {
        let n: usize = n.into();
        self.lc_extended.get(n)
    }
    fn next(&mut self) {
        for l in &mut self.lc_extended {
            l.next();
        }
        let n = self.lc_extended.len();
        // next との整合性
        self.lc_extended.push(self.lc_init.as_ref().clone());
        let mut b = true;
        for (o, i) in self.next_edges.iter() {
            for l in 0..n {
                let o = *self.lc_extended[l].get_output(o).unwrap();
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
                let o = *self.lc_extended[l].get_output(o).unwrap();
                let i = self.lc_extended[l - 1].getmut_input(i).unwrap();
                *i = o;
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LoC {
    Gate(Gate),
    FinGraph(Box<FinGraph>),
    Iter(Iter),
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
        Ok(LoC::FinGraph(Box::new(FinGraph::new(
            name, lcs, edges, input, output,
        )?)))
    }
    pub fn new_iter(
        name: Name,
        lc: LoC,
        next_edges: Vec<(OtPin, InPin)>,
        prev_edges: Vec<(OtPin, InPin)>,
        input: Vec<(InPin, InPin)>,
        otput: Vec<(OtPin, OtPin)>,
    ) -> Result<Self> {
        Ok(LoC::Iter(Iter::new(
            name, lc, next_edges, prev_edges, input, otput,
        )?))
    }
    pub fn name(&self) -> String {
        match self {
            LoC::Gate(gate) => gate.name(),
            LoC::FinGraph(fingraph) => fingraph.name.to_string(),
            LoC::Iter(iter) => iter.name.to_string(),
        }
    }
    pub fn get_input(&self, inpin: &InPin) -> Option<&Bool> {
        match self {
            LoC::Gate(gate) => gate.get_input(inpin),
            LoC::FinGraph(fingraph) => fingraph.get_input(inpin),
            LoC::Iter(iter) => iter.get_input(inpin),
        }
    }
    pub fn getmut_input(&mut self, inpin: &InPin) -> Option<&mut Bool> {
        match self {
            LoC::Gate(gate) => gate.getmut_input(inpin),
            LoC::FinGraph(fingraph) => fingraph.getmut_input(inpin),
            LoC::Iter(iter) => iter.getmut_input(inpin),
        }
    }
    pub fn get_output(&self, otpin: &OtPin) -> Option<&Bool> {
        match self {
            LoC::Gate(gate) => gate.get_output(otpin),
            LoC::FinGraph(fingraph) => fingraph.get_output(otpin),
            LoC::Iter(iter) => iter.get_otput(otpin),
        }
    }
    pub fn getmut_lc_from_path(&mut self, path: &Path) -> Option<&mut LoC> {
        let mut lc = self;
        for name in path {
            match (lc, name) {
                (LoC::FinGraph(fingraph), Either::Left(name)) => {
                    lc = fingraph.getmut_lc(name)?;
                }
                (LoC::Iter(iter), Either::Right(num)) => {
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
                (LoC::FinGraph(fingraph), Either::Left(name)) => {
                    lc = fingraph.get_lc(name)?;
                }
                (LoC::Iter(iter), Either::Right(num)) => {
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
            LoC::FinGraph(fingraph) => fingraph.next(),
            LoC::Iter(iter) => iter.next(),
        }
    }
    pub fn get_all_input_name(&self) -> Vec<InPin> {
        match self {
            LoC::Gate(gate) => gate.get_all_input_name(),
            LoC::FinGraph(fingraph) => fingraph.input.keys().cloned().collect(),
            LoC::Iter(iter) => iter.input.keys().cloned().collect(),
        }
    }
}

pub fn print_format(lc: &LoC) {
    fn print_format(lc: &LoC) -> Vec<String> {
        match lc {
            LoC::Gate(gate) => {
                vec![]
            }
            LoC::FinGraph(fingraph) => {
                let FinGraph {
                    name,
                    lcs,
                    edges,
                    input,
                    output,
                } = fingraph.as_ref();
                let mut lines = vec![];
                lines.push(format!("fingraph:{name}"));

                let mut l_in = "i...".to_string();
                l_in.extend(input.iter().map(|(i, (n0, i0))| {
                    format!("{i}={n0}.{i0}:{}, ", fingraph.get_input(i).unwrap())
                }));
                lines.push(l_in);

                let mut l_ot = "o...".to_string();
                l_ot.extend(output.iter().map(|(o, (n0, o0))| {
                    format!("{o}={n0}.{o0}:{}, ", fingraph.get_output(o).unwrap())
                }));
                lines.push(l_ot);
                let edges = {
                    let mut new_edges: HashMap<_, Vec<(InPin, (Name, OtPin))>> = HashMap::new();
                    for ((n0, o0), (n1, i1)) in edges.iter() {
                        new_edges.entry(n1).or_default();
                        let io = new_edges.get_mut(n1).unwrap();
                        io.push((i1.clone(), (n0.clone(), o0.clone())));
                    }
                    new_edges
                };
                for (name, lc) in lcs.iter() {
                    let ins = edges.get(name).map_or("not found".to_string(), |l| {
                        l.iter()
                            .map(|(i, (n, o))| {
                                format!("{i}={n}.{o}:{}, ", lc.get_input(i).unwrap())
                            })
                            .collect()
                    });
                    lines.push(format!("  {name}, {}, {ins}", lc.name()));
                    for s in print_format(lc) {
                        let s = format!("  {s}");
                        lines.push(s);
                    }
                }
                lines
            }
            LoC::Iter(iter) => {
                let Iter {
                    name,
                    lc_init,
                    lc_extended,
                    next_edges,
                    prev_edges,
                    input,
                    otput,
                } = iter;
                let mut lines = vec![];
                lines.push(format!("iterator:{name}"));

                let mut l_in = "i...".to_string();
                l_in.extend(
                    input
                        .iter()
                        .map(|(i, i0)| format!("{i}={i0}:{}, ", iter.get_input(i).unwrap())),
                );
                lines.push(l_in);

                let mut o_in = "o...".to_string();
                o_in.extend(
                    otput
                        .iter()
                        .map(|(o, o0)| format!("{o}={o0}:{}, ", iter.get_otput(o).unwrap())),
                );
                lines.push(o_in);

                lines
            }
        }
    }
    for l in print_format(lc) {
        eprintln!("{l}")
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
            "RS".into(),
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

        let a = rs.get_all_input_name();
        assert_eq!(a, vec!["R".into(), "S".into()]);

        print_format(&rs);

        let t = |lc: &mut LoC| loop {
            let lc_prev = lc.clone();
            lc.next();
            if lc_prev == *lc {
                break;
            }
            print_format(lc);
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
