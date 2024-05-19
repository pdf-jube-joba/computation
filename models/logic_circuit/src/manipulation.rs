use std::collections::HashMap;

use crate::machine::*;
use anyhow::{bail, Ok, Result};
use pest::{iterators::Pair, Parser};

#[derive(pest_derive::Parser)]
#[grammar = "logic_circuit.pest"]
struct Ps;

// contains fundamental gate
pub fn init_maps() -> HashMap<Name, LoC> {
    let mut maps = HashMap::new();
    maps.insert("NOT-T".into(), LoC::notgate(Bool::T));
    maps.insert("NOT-F".into(), LoC::notgate(Bool::F));
    maps.insert("AND-T".into(), LoC::andgate(Bool::T));
    maps.insert("AND-F".into(), LoC::andgate(Bool::F));
    maps.insert("OR-T".into(), LoC::orgate(Bool::T));
    maps.insert("OR-F".into(), LoC::orgate(Bool::F));
    maps.insert("CST-T".into(), LoC::cstgate(Bool::T));
    maps.insert("CST-F".into(), LoC::cstgate(Bool::F));
    maps.insert("BR-T".into(), LoC::brgate(Bool::T));
    maps.insert("BR-F".into(), LoC::brgate(Bool::F));
    maps.insert("END".into(), LoC::endgate());
    maps.insert("DLY-T".into(), LoC::delaygate(Bool::T));
    maps.insert("DLY-F".into(), LoC::delaygate(Bool::F));
    maps
}

pub fn parse(code: &str, maps: &mut HashMap<Name, LoC>) -> Result<()> {
    let lcs = Ps::parse(Rule::lcs, code)?;
    for lc in lcs {
        match lc.as_rule() {
            Rule::fingraph => {
                let FingraphParse {
                    name,
                    inpin,
                    otpin,
                    lcs,
                } = fingraph_parse(lc.as_str());
                eprintln!("{name}");
                let mut v = vec![];
                let mut e = vec![];
                for (lcname, usename, inout) in lcs {
                    let Some(c) = maps.get(&usename) else {
                        bail!("not found name {usename}");
                    };
                    v.push((lcname.clone(), c.clone()));
                    for (i, (n, o)) in inout {
                        e.push(((n.clone(), o.clone()), (lcname.clone(), i.clone())));
                    }
                }
                let graphlc = LoC::new_graph(name.clone(), v, e, inpin, otpin)?;
                maps.insert(name, graphlc);
            }
            Rule::iterator => {
                let IterParse {
                    name,
                    initlc,
                    inpin,
                    otpin,
                    next,
                    prev,
                } = iter_parse(lc.as_str());
                eprintln!("{name}");
                let Some(initlc) = maps.get(&initlc) else {
                    bail!("not found name {initlc}");
                };
                let iterlc = LoC::new_iter(name.clone(), initlc.clone(), next, prev, inpin, otpin)?;
                maps.insert(name, iterlc);
            }
            _ => {
                assert_eq!(lc.as_str(), "");
            }
        }
    }
    Ok(())
}

pub fn parse_main_with_maps(code: &str, init_maps: &HashMap<Name, LoC>) -> Result<LoC> {
    let mut maps = init_maps.clone();
    parse(code, &mut maps)?;
    match maps.get(&"main".into()) {
        Some(lc) => Ok(lc.clone()),
        None => bail!("not found main"),
    }
}

pub fn parse_main(code: &str) -> Result<LoC> {
    let mut maps: HashMap<Name, LoC> = init_maps();
    parse(code, &mut maps)?;
    match maps.get(&"main".into()) {
        Some(lc) => Ok(lc.clone()),
        None => bail!("not found main"),
    }
}

#[derive(Debug)]
struct FingraphParse {
    name: Name,
    inpin: Vec<(InPin, (Name, InPin))>,
    otpin: Vec<(OtPin, (Name, OtPin))>,
    lcs: Vec<(Name, Name, Vec<(InPin, (Name, OtPin))>)>,
}

fn fingraph_parse<'a>(code: &'a str) -> FingraphParse {
    let lc = Ps::parse(Rule::fingraph, code).unwrap();
    let mut l = lc.into_iter().next().unwrap().into_inner();
    let conn_graph_parse = |p: Pair<'a, Rule>| -> (&'a str, &'a str, &'a str) {
        assert_eq!(p.as_rule(), Rule::conn_graph);
        let mut v = p.into_inner();
        let i = v.next().unwrap().as_str();
        let n0 = v.next().unwrap().as_str();
        let i0 = v.next().unwrap().as_str();
        (i, n0, i0)
    };
    let name: Name = {
        let name = l.next().unwrap();
        assert_eq!(name.as_rule(), Rule::name);
        name.as_str().into()
    };
    let inpin: Vec<(InPin, (Name, InPin))> = {
        let inpins = l.next().unwrap();
        assert_eq!(inpins.as_rule(), Rule::in_graph);
        inpins
            .into_inner()
            .map(|p| {
                assert_eq!(p.as_rule(), Rule::conn_graph);
                let (i, n0, i0) = conn_graph_parse(p);
                (i.into(), (n0.into(), i0.into()))
            })
            .collect()
    };
    let otpin: Vec<(OtPin, (Name, OtPin))> = {
        let inpins = l.next().unwrap();
        assert_eq!(inpins.as_rule(), Rule::ot_graph);
        inpins
            .into_inner()
            .map(|p| {
                assert_eq!(p.as_rule(), Rule::conn_graph);
                let (o, n0, o0) = conn_graph_parse(p);
                (o.into(), (n0.into(), o0.into()))
            })
            .collect()
    };
    let lcs: Vec<(Name, Name, Vec<(InPin, (Name, OtPin))>)> = {
        let mut v = vec![];
        for lcs in l {
            assert_eq!(lcs.as_rule(), Rule::lc_graph);
            let mut vs = lcs.into_inner();
            let name: Name = vs.next().unwrap().as_str().into();
            let usename: Name = vs.next().unwrap().as_str().into();
            let ve: Vec<(InPin, (Name, OtPin))> = vs
                .map(|p| {
                    assert_eq!(p.as_rule(), Rule::conn_graph);
                    let (i, n, o) = conn_graph_parse(p);
                    (i.into(), (n.into(), o.into()))
                })
                .collect();
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
    name: Name,
    initlc: Name,
    inpin: Vec<(InPin, InPin)>,
    otpin: Vec<(OtPin, OtPin)>,
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
    let name: Name = {
        let name = l.next().unwrap();
        assert_eq!(name.as_rule(), Rule::name);
        name.as_str().into()
    };
    let initlc: Name = {
        let name = l.next().unwrap();
        assert_eq!(name.as_rule(), Rule::name);
        name.as_str().into()
    };
    let inpin: Vec<(InPin, InPin)> = {
        let inpin = l.next().unwrap();
        assert_eq!(inpin.as_rule(), Rule::in_iter);
        inpin
            .into_inner()
            .map(|p| {
                assert_eq!(p.as_rule(), Rule::conn_iter);
                let (i, i0) = conn_iter_parse(p);
                (i.into(), i0.into())
            })
            .collect()
    };
    let otpin: Vec<(OtPin, OtPin)> = {
        let otpin = l.next().unwrap();
        assert_eq!(otpin.as_rule(), Rule::ot_iter);
        otpin
            .into_inner()
            .map(|p| {
                assert_eq!(p.as_rule(), Rule::conn_iter);
                let (i, i0) = conn_iter_parse(p);
                (i.into(), i0.into())
            })
            .collect()
    };
    let next: Vec<(OtPin, InPin)> = {
        let otpin = l.next().unwrap();
        assert_eq!(otpin.as_rule(), Rule::next_iter);
        otpin
            .into_inner()
            .map(|p| {
                assert_eq!(p.as_rule(), Rule::conn_iter);
                let (i, i0) = conn_iter_parse(p);
                (i.into(), i0.into())
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
                (i.into(), i0.into())
            })
            .collect()
    };
    IterParse {
        name,
        initlc,
        inpin,
        otpin,
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
            in {A=b.c, d=g.f }
            out {a=b.c}
            A, AND-T,
          }";
        let _c = parse_main(s).unwrap();
    }
}
