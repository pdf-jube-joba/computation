use anyhow::{anyhow, bail, Result};
use either::Either;
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
            _ => Err(anyhow!("a")),
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

type InPin = String;
type OtPin = String;

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
    End {
        input: Bool,
    },
}

impl Gate {
    fn state(&self) -> &Bool {
        match self {
            Gate::Cst { state } => state,
            Gate::Not { state, input } => state,
            Gate::Br { state, input } => state,
            Gate::End { input } => input,
            Gate::And {
                state,
                input0,
                input1,
            } => state,
            Gate::Or {
                state,
                input0,
                input1,
            } => state,
        }
    }
    fn get_input(&self, input_name: InPin) -> Option<&Bool> {
        match (self, input_name.as_str()) {
            (Gate::Not { state, input }, "IN") => Some(input),
            (Gate::Br { state, input }, "IN") => Some(input),
            (Gate::End { input }, "IN") => Some(input),
            (
                Gate::And {
                    state,
                    input0,
                    input1,
                },
                "IN0",
            ) => Some(input0),
            (
                Gate::And {
                    state,
                    input0,
                    input1,
                },
                "IN1",
            ) => Some(input1),
            (
                Gate::Or {
                    state,
                    input0,
                    input1,
                },
                "IN0",
            ) => Some(input0),
            (
                Gate::Or {
                    state,
                    input0,
                    input1,
                },
                "IN1",
            ) => Some(input1),
            _ => None,
        }
    }
    fn getmut_input(&mut self, input_name: InPin) -> Option<&mut Bool> {
        match (self, input_name.as_str()) {
            (Gate::Not { state, input }, "IN") => Some(input),
            (Gate::Br { state, input }, "IN") => Some(input),
            (Gate::End { input }, "IN") => Some(input),
            (
                Gate::And {
                    state,
                    input0,
                    input1,
                },
                "IN0",
            ) => Some(input0),
            (
                Gate::And {
                    state,
                    input0,
                    input1,
                },
                "IN1",
            ) => Some(input1),
            (
                Gate::Or {
                    state,
                    input0,
                    input1,
                },
                "IN0",
            ) => Some(input0),
            (
                Gate::Or {
                    state,
                    input0,
                    input1,
                },
                "IN1",
            ) => Some(input1),
            _ => None,
        }
    }
    fn get_output(&self, output_name: &OtPin) -> Option<&Bool> {
        match (self, output_name.as_str()) {
            (Gate::Not { state, input }, "OUT") => Some(state),
            (Gate::Cst { state }, "OUT") => Some(state),
            (Gate::Br { state, input }, "OUT0") => Some(state),
            (Gate::Br { state, input }, "OUT1") => Some(state),
            (Gate::End { input }, _) => None,
            (
                Gate::And {
                    state,
                    input0,
                    input1,
                },
                "OUT",
            ) => Some(state),
            (
                Gate::Or {
                    state,
                    input0,
                    input1,
                },
                "OUT",
            ) => Some(state),
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
    fn name(&self) -> String {
        match self {
            Gate::Not { state, input } => "not".to_owned(),
            Gate::And {
                state,
                input0,
                input1,
            } => "and".to_owned(),
            Gate::Or {
                state,
                input0,
                input1,
            } => "or ".to_owned(),
            Gate::Cst { state } => format!("cst{state}"),
            Gate::Br { state, input } => "br ".to_owned(),
            Gate::End { input } => "end".to_owned(),
        }
    }
}

type Name = String;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FinGraph {
    name: String,
    lcs: HashMap<Name, LoC>,
    edges: HashSet<((Name, OtPin), (Name, InPin))>,
    input: HashMap<InPin, (Name, InPin)>,
    output: HashMap<OtPin, (Name, OtPin)>,
}

impl FinGraph {
    fn new(
        name: String,
        lcs: Vec<(Name, LoC)>,
        edges: Vec<((Name, OtPin), (Name, InPin))>,
        input: Vec<(InPin, (Name, InPin))>,
        output: Vec<(OtPin, (Name, OtPin))>,
    ) -> Result<Self> {
        let mut lcs: HashMap<Name, LoC> = lcs.into_iter().collect();
        let mut new_edges: HashSet<((Name, InPin), (Name, OtPin))> = HashSet::new();
        let mut new_input = HashMap::new();
        let mut new_output = HashMap::new();
        for ((n0, o), (n1, i)) in edges {
            let Some(n0_lc) = lcs.get(&n0) else {
                bail!("fail {name} not found name {n0}")
            };
            let Some(&ob) = n0_lc.get_output(o.clone()) else {
                bail!("fail {name} not found outpin {o} in {n0}")
            };

            let Some(n1_lc) = lcs.get_mut(&n1) else {
                bail!("fail {name} not found name {n1}")
            };

            let Some(ib) = n1_lc.getmut_input(i.clone()) else {
                bail!("fail {name} not found inpin {i} in {n1}")
            };
            *ib = ob;
            new_edges.insert(((n0, o), (n1, i)));
        }
        for (i, (n, i0)) in input {
            let Some(nlc) = lcs.get(&n) else {
                bail!("fail {name} not found {n}")
            };
            if nlc.get_input(i0.clone()).is_none() {
                bail!("fail {name} not found inpin {i0} in {n}")
            }
            new_input.insert(i, (n, i0));
        }
        for (o, (n, o0)) in output {
            let Some(nlc) = lcs.get(&n) else {
                bail!("fail {name} not found {n}")
            };
            if nlc.get_output(o0.clone()).is_none() {
                bail!("fail {name} not found inpin {o0} in {n}")
            }
            new_output.insert(o, (n, o0));
        }
        Ok(Self {
            name,
            lcs,
            edges: new_edges,
            input: new_input,
            output: new_output,
        })
    }
    fn get_input(&self, inpin: InPin) -> Option<&Bool> {
        let (name, inpin) = self.input.get(&inpin)?;
        let lc = self.lcs.get(name)?;
        lc.get_input(inpin.to_string())
    }
    fn getmut_input(&mut self, inpin: InPin) -> Option<&mut Bool> {
        let (name, inpin) = self.input.get(&inpin)?;
        let lc = self.lcs.get_mut(name)?;
        lc.getmut_input(inpin.to_string())
    }
    fn get_output(&self, otpin: OtPin) -> Option<&Bool> {
        let (name, otpin) = self.output.get(&otpin)?;
        let lc = self.lcs.get(name)?;
        lc.get_output(otpin.to_string())
    }
    fn getmut_lc(&mut self, name: Name) -> Option<&mut LoC> {
        self.lcs.get_mut(&name)
    }
    fn get_lc(&self, name: Name) -> Option<&LoC> {
        self.lcs.get(&name)
    }
    fn next(&mut self) {
        for lc in self.lcs.values_mut() {
            lc.next();
        }
        for ((n0, o), (n1, i)) in self.edges.clone() {
            let lco = *self.get_lc(n0).unwrap().get_output(o).unwrap();
            let lci = self.getmut_lc(n1).unwrap().getmut_input(i).unwrap();
            *lci = lco;
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Iter {
    name: String,
    lc_init: Box<LoC>,
    lc_extended: Vec<LoC>,
    next_edges: HashSet<(OtPin, InPin)>,
    prev_edges: HashSet<(OtPin, InPin)>,
    input: HashMap<InPin, InPin>,
    otput: HashMap<OtPin, OtPin>,
}

impl Iter {
    fn new(
        name: String,
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
    fn get_input(&self, inpin: InPin) -> Option<&Bool> {
        let inpin = self.input.get(&inpin)?.to_owned();
        self.lc_extended[0].get_input(inpin)
    }
    fn getmut_input(&mut self, inpin: InPin) -> Option<&mut Bool> {
        let inpin = self.input.get_mut(&inpin)?.to_owned();
        self.lc_extended[0].getmut_input(inpin)
    }
    fn get_otput(&self, otpin: OtPin) -> Option<&Bool> {
        let otpin = self.otput.get(&otpin)?.to_owned();
        self.lc_extended[0].get_output(otpin)
    }
    fn getmut_lc(&mut self, n: Number) -> Option<&mut LoC> {
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
                let o = *self.lc_extended[l].get_output(o.clone()).unwrap();
                let i = self.lc_extended[l + 1].getmut_input(i.clone()).unwrap();
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
                let o = *self.lc_extended[l].get_output(o.clone()).unwrap();
                let i = self.lc_extended[l - 1].getmut_input(i.clone()).unwrap();
                *i = o;
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LoC {
    Gate(Gate),
    FinGraph(FinGraph),
    Iter(Iter),
}

type Path = Vec<Either<Name, Number>>;
fn into_inpin_path(str: &str) -> Path {
    let mut p: Vec<_> = str
        .split(".")
        .map(|s| match s.parse::<usize>() {
            Ok(n) => Either::Right(n.into()),
            Err(_) => Either::Left(s.to_string()),
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
    pub fn end() -> LoC {
        LoC::Gate(Gate::End { input: Bool::F })
    }
    pub fn new_graph(
        name: String,
        lcs: Vec<(Name, LoC)>,
        edges: Vec<((Name, OtPin), (Name, InPin))>,
        input: Vec<(InPin, (Name, InPin))>,
        output: Vec<(OtPin, (Name, OtPin))>,
    ) -> Result<Self> {
        Ok(LoC::FinGraph(FinGraph::new(
            name, lcs, edges, input, output,
        )?))
    }
    pub fn new_iter(
        name: String,
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
            LoC::FinGraph(fingraph) => fingraph.name.to_owned(),
            LoC::Iter(iter) => iter.name.to_owned(),
        }
    }
    pub fn get_input(&self, inpin: InPin) -> Option<&Bool> {
        match self {
            LoC::Gate(gate) => gate.get_input(inpin),
            LoC::FinGraph(fingraph) => fingraph.get_input(inpin),
            LoC::Iter(iter) => iter.get_input(inpin),
        }
    }
    pub fn getmut_input(&mut self, inpin: InPin) -> Option<&mut Bool> {
        match self {
            LoC::Gate(gate) => gate.getmut_input(inpin),
            LoC::FinGraph(fingraph) => fingraph.getmut_input(inpin),
            LoC::Iter(iter) => iter.getmut_input(inpin),
        }
    }
    pub fn get_output(&self, otpin: OtPin) -> Option<&Bool> {
        match self {
            LoC::Gate(gate) => gate.get_output(&otpin),
            LoC::FinGraph(fingraph) => fingraph.get_output(otpin),
            LoC::Iter(iter) => iter.get_otput(otpin),
        }
    }
    pub fn getmut_lc_from_path(&mut self, path: Path) -> Option<&mut LoC> {
        let mut lc = self;
        for name in path {
            match (lc, name) {
                (LoC::FinGraph(fingraph), Either::Left(name)) => {
                    lc = fingraph.getmut_lc(name)?;
                }
                (LoC::Iter(iter), Either::Right(num)) => {
                    lc = iter.getmut_lc(num)?;
                }
                _ => {
                    return None;
                }
            }
        }
        Some(lc)
    }
    pub fn get_lc_from_path(&self, path: Path) -> Option<&LoC> {
        let mut lc = self;
        for name in path {
            match (lc, name) {
                (LoC::FinGraph(fingraph), Either::Left(name)) => {
                    lc = fingraph.get_lc(name)?;
                }
                (LoC::Iter(iter), Either::Right(num)) => {
                    lc = iter.get_lc(num)?;
                }
                _ => {
                    return None;
                }
            }
        }
        Some(lc)
    }
    pub fn get_state_of_gate_from_path(&self, path: Path) -> Option<&Bool> {
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
}

pub fn print_format(lc: &LoC) {
    fn print_format(lc: &LoC) -> Vec<String> {
        match lc {
            LoC::Gate(gate) => {
                // match gate {
                //     Gate::Cst { state } => vec![format!("cst {state} in:/")],
                //     Gate::Not { state, input } => vec![format!("not {state} in:{input}")],
                //     Gate::End { input } => vec![format!("end / in:{input}")],
                //     Gate::Br { state, input } => vec![format!("bra {state} in:{input}")],
                //     Gate::And {
                //         state,
                //         input0,
                //         input1,
                //     } => vec![format!("and {state} in:{input0} {input1}")],
                //     Gate::Or {
                //         state,
                //         input0,
                //         input1,
                //     } => vec![format!("or  {state} in:{input0} {input1}")],
                // },
                vec![]
            }
            LoC::FinGraph(fingraph) => {
                let FinGraph {
                    name,
                    lcs,
                    edges,
                    input,
                    output,
                } = fingraph;
                let mut lines = vec![];
                lines.push(format!("fingraph:{name}"));
                let mut l_in = "i...".to_string();
                l_in.extend(input.iter().map(|(i, (n0, i0))| {
                    format!("{i}={n0}.{i0}:{}, ", fingraph.get_input(i.clone()).unwrap())
                }));
                lines.push(l_in);

                let mut l_ot = "o...".to_string();
                l_ot.extend(output.iter().map(|(o, (n0, o0))| {
                    format!(
                        "{o}={n0}.{o0}:{}, ",
                        fingraph.get_output(o.clone()).unwrap()
                    )
                }));
                lines.push(l_ot);
                let edges = {
                    let mut new_edges: HashMap<_, Vec<(InPin, (Name, OtPin))>> = HashMap::new();
                    for ((n0, o0), (n1, i1)) in edges.iter() {
                        new_edges.entry(n1).or_default();
                        let io = new_edges.get_mut(n1).unwrap();
                        io.push((i1.to_string(), (n0.to_string(), o0.to_string())));
                    }
                    new_edges
                };
                for (name, lc) in lcs.iter() {
                    let ins = edges.get(name).map_or("not found".to_string(), |l| {
                        l.iter()
                            .map(|(i, (n, o))| {
                                format!("{i}={n}.{o}:{}, ", lc.get_input(i.clone()).unwrap())
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
                vec![]
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
            "RS".to_string(),
            vec![
                ("O0".to_string(), LoC::orgate(Bool::T)),
                ("N0".to_string(), LoC::notgate(Bool::F)),
                ("B0".to_string(), LoC::brgate(Bool::F)),
                ("O1".to_string(), LoC::orgate(Bool::F)),
                ("N1".to_string(), LoC::notgate(Bool::T)),
                ("B1".to_string(), LoC::brgate(Bool::T)),
            ],
            vec![
                (("O0", "OUT"), ("N0", "IN")),
                (("O1", "OUT"), ("N1", "IN")),
                (("N0", "OUT"), ("B0", "IN")),
                (("N1", "OUT"), ("B1", "IN")),
                (("B0", "OUT1"), ("O1", "IN1")),
                (("B1", "OUT1"), ("O0", "IN1")),
            ]
            .into_iter()
            .map(|((n0, o), (n1, i))| {
                ((n0.to_owned(), o.to_owned()), (n1.to_owned(), i.to_owned()))
            })
            .collect::<Vec<_>>(),
            vec![("R", ("O0", "IN0")), ("S", ("O1", "IN0"))]
                .into_iter()
                .map(|(i, (n0, i0))| (i.to_owned(), (n0.to_owned(), i0.to_owned())))
                .collect::<Vec<_>>(),
            vec![("Q", ("B0", "OUT0")), ("nQ", ("B1", "OUT0"))]
                .into_iter()
                .map(|(o, (n0, o0))| (o.to_owned(), (n0.to_owned(), o0.to_owned())))
                .collect::<Vec<_>>(),
        );
        let mut rs = rs.unwrap();

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

        let r = rs.getmut_input("R".to_string()).unwrap();
        *r = Bool::T;
        t(&mut rs);
        println!("---");

        let r = rs.getmut_input("R".to_string()).unwrap();
        *r = Bool::F;
        let r = rs.getmut_input("S".to_string()).unwrap();
        *r = Bool::T;
        t(&mut rs);
    }
}
