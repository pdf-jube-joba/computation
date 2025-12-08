use crate::machine::*;
use anyhow::{bail, Ok, Result};
use either::Either;
use pest::{iterators::Pair, Parser};
use utils::{alphabet::Identifier, bool::Bool};

#[derive(pest_derive::Parser)]
#[grammar = "logic_circuit.pest"]
struct Ps;

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
    Identifier::new_user(name).unwrap()
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

pub fn parse(code: &str, maps: &mut List) -> Result<()> {
    let lcs = Ps::parse(Rule::lcs, code)?;
    for lc in lcs {
        match lc.as_rule() {
            Rule::fingraph => {
                let FingraphParse {
                    name,
                    inpin: inpins,
                    otpin: otpin_maps,
                    lcs,
                } = fingraph_parse(lc.as_str());
                eprintln!("{name}");
                let mut new_lcs = vec![];
                let mut edges = vec![];
                let mut inpin_maps = vec![];
                // let mut otpins = vec![];
                for (lcname, usename, inout) in lcs {
                    let Some(c) = maps.get(&usename) else {
                        bail!("not found name {usename}");
                    };
                    new_lcs.push((lcname.clone(), c.clone()));
                    for (inpin, out) in inout {
                        match out {
                            Either::Left((name, otpin)) => {
                                edges.push(((name, otpin), (lcname.clone(), inpin)));
                            }
                            Either::Right(i) => {
                                if !inpins.contains(&i) {
                                    bail!("not found inpin {i}");
                                }
                                inpin_maps.push((i, (lcname.clone(), inpin)));
                            }
                        }
                    }
                }

                let graphlc = LogicCircuit::new_mix(name.clone(), new_lcs, edges)?;
                // use core::pinmap
                let pinmap = LogicCircuit::new_pin_map(
                    graphlc,
                    inpin_maps
                        .into_iter()
                        .map(|(i, (n, ni))| (i, concat_inpin(n, ni)))
                        .collect(),
                    otpin_maps
                        .into_iter()
                        .map(|(o, (n, no))| (o, concat_otpin(n, no)))
                        .collect(),
                )?;
                maps.insert((name, pinmap));
            }
            Rule::iterator => {
                let IterParse {
                    name,
                    initlc,
                    next,
                    prev,
                } = iter_parse(lc.as_str());
                eprintln!("{name}");
                let Some(initlc) = maps.get(&initlc) else {
                    bail!("not found name {initlc}");
                };
                let iterlc = LogicCircuit::new_iter(name.clone(), initlc.clone(), next, prev)?;
                maps.insert((name, iterlc));
            }
            _ => {
                assert_eq!(lc.as_str(), "");
            }
        }
    }
    Ok(())
}

pub fn parse_main_with_maps(code: &str, mut maps: List) -> Result<LogicCircuit> {
    parse(code, &mut maps)?;
    match maps.get(&Identifier::new_user("main").unwrap()) {
        Some(lc) => Ok(lc.clone()),
        None => bail!("not found main"),
    }
}

pub fn parse_main(code: &str) -> Result<LogicCircuit> {
    let mut maps: List = init_maps();
    parse(code, &mut maps)?;
    match maps.get(&Identifier::new_user("main").unwrap()) {
        Some(lc) => Ok(lc.clone()),
        None => bail!("not found main"),
    }
}

type Pin = (InPin, Either<(Identifier, OtPin), InPin>);

#[derive(Debug)]
struct FingraphParse {
    name: Identifier,
    inpin: Vec<InPin>,
    otpin: Vec<(OtPin, (Identifier, OtPin))>,
    lcs: Vec<(Identifier, Identifier, Vec<Pin>)>,
}

fn conn_graph_parse(p: Pair<'_, Rule>) -> Pin {
    assert_eq!(p.as_rule(), Rule::conn_graph);
    let mut l = p.into_inner();
    let i: InPin = l.next().unwrap().as_str().parse().unwrap();
    let e = {
        let e = l.next().unwrap();
        assert_eq!(e.as_rule(), Rule::pin);
        let mut e = e.into_inner();
        let first = e.next().unwrap();
        let second = e.next();
        match second {
            Some(i) => Either::Left((
                Identifier::new_user(first.as_str()).unwrap(),
                i.as_str().parse().unwrap(),
            )),
            None => Either::Right(first.as_str().parse().unwrap()),
        }
    };
    (i, e)
}

fn otpin_graph_parse(p: Pair<'_, Rule>) -> (OtPin, (Identifier, OtPin)) {
    assert_eq!(p.as_rule(), Rule::otpin_graph);
    let mut l = p.into_inner();
    let o = l.next().unwrap();
    let n0 = l.next().unwrap();
    let o0 = l.next().unwrap();
    (
        o.as_str().parse().unwrap(),
        (
            Identifier::new_user(n0.as_str()).unwrap(),
            o0.as_str().parse().unwrap(),
        ),
    )
}

fn fingraph_parse(code: &str) -> FingraphParse {
    let lc = Ps::parse(Rule::fingraph, code).unwrap();
    let mut l = lc.into_iter().next().unwrap().into_inner();

    let name: Identifier = {
        let name = l.next().unwrap();
        assert_eq!(name.as_rule(), Rule::name);
        Identifier::new_user(name.as_str()).unwrap()
    };
    let inpin: Vec<InPin> = {
        let inpins = l.next().unwrap();
        assert_eq!(inpins.as_rule(), Rule::in_graph);
        inpins
            .into_inner()
            .map(|p| {
                assert_eq!(p.as_rule(), Rule::name);
                p.as_str().parse().unwrap()
            })
            .collect()
    };
    let otpin: Vec<(OtPin, (Identifier, OtPin))> = {
        let otpins = l.next().unwrap();
        assert_eq!(otpins.as_rule(), Rule::ot_graph);
        otpins.into_inner().map(|p| otpin_graph_parse(p)).collect()
    };
    let lcs: Vec<(Identifier, Identifier, Vec<Pin>)> = {
        let mut v = vec![];
        for lcs in l {
            assert_eq!(lcs.as_rule(), Rule::lc_graph);
            let mut vs = lcs.into_inner();
            let name: Identifier = Identifier::new_user(vs.next().unwrap().as_str()).unwrap();
            let usename: Identifier = Identifier::new_user(vs.next().unwrap().as_str()).unwrap();
            let ve: Vec<_> = vs.map(|p| conn_graph_parse(p)).collect();
            v.push((name, usename, ve))
        }
        v
    };
    FingraphParse {
        name,
        inpin,
        otpin,
        lcs,
    }
}

#[derive(Debug)]
struct IterParse {
    name: Identifier,
    initlc: Identifier,
    next: Vec<(OtPin, InPin)>,
    prev: Vec<(OtPin, InPin)>,
}

fn iter_parse<'a>(code: &'a str) -> IterParse {
    let conn_iter_parse = |p: Pair<'a, Rule>| -> (&'a str, &'a str) {
        assert_eq!(p.as_rule(), Rule::conn_iter);
        let mut v = p.into_inner();
        let o = v.next().unwrap().as_str();
        let i = v.next().unwrap().as_str();
        (o, i)
    };
    let lc = Ps::parse(Rule::iterator, code).unwrap();
    let mut l = lc.into_iter().next().unwrap().into_inner();
    let name: Identifier = {
        let name = l.next().unwrap();
        assert_eq!(name.as_rule(), Rule::name);
        Identifier::new_user(name.as_str()).unwrap()
    };
    let initlc: Identifier = {
        let name = l.next().unwrap();
        assert_eq!(name.as_rule(), Rule::name);
        Identifier::new_user(name.as_str()).unwrap()
    };
    let next: Vec<(OtPin, InPin)> = {
        let otpin = l.next().unwrap();
        assert_eq!(otpin.as_rule(), Rule::next_iter);
        otpin
            .into_inner()
            .map(|p| {
                assert_eq!(p.as_rule(), Rule::conn_iter);
                let (i, i0) = conn_iter_parse(p);
                (i.parse().unwrap(), i0.parse().unwrap())
            })
            .collect()
    };
    let prev: Vec<(OtPin, InPin)> = {
        let otpin = l.next().unwrap();
        assert_eq!(otpin.as_rule(), Rule::prev_iter);
        otpin
            .into_inner()
            .map(|p| {
                assert_eq!(p.as_rule(), Rule::conn_iter);
                let (i, i0) = conn_iter_parse(p);
                (i.parse().unwrap(), i0.parse().unwrap())
            })
            .collect()
    };
    IterParse {
        name,
        initlc,
        next,
        prev,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn f() {
        let s = "graph: main {
            in {A, d}
            out {a=b.c}
            A, AND-T {}
          }";
        let _c = parse_main(s).unwrap();
    }
}
