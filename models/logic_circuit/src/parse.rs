use crate::machine::*;
use crate::manipulation::{init_maps, List};
use anyhow::{bail, Result};
use either::Either;
use pest::{iterators::Pair, Parser};
use utils::identifier::Identifier;

#[derive(pest_derive::Parser)]
#[grammar = "logic_circuit.pest"]
struct Ps;

type GraphPin = (Identifier, Either<NamedPin, Identifier>);

#[derive(Debug)]
struct FingraphParse {
    name: Identifier,
    inpin: Vec<Identifier>,
    otpin: Vec<(Identifier, NamedPin)>,
    lcs: Vec<(Identifier, Identifier, Vec<GraphPin>)>,
}

fn conn_graph_parse(p: Pair<'_, Rule>) -> GraphPin {
    assert_eq!(p.as_rule(), Rule::conn_graph);
    let mut l = p.into_inner();
    let i: Identifier = Identifier::new(l.next().unwrap().as_str()).unwrap();
    let e = {
        let e = l.next().unwrap();
        assert_eq!(e.as_rule(), Rule::pin);
        let mut e = e.into_inner();
        let first = e.next().unwrap();
        let second = e.next();
        match second {
            Some(i) => Either::Left((
                Identifier::new(first.as_str()).unwrap(),
                Identifier::new(i.as_str()).unwrap(),
            )),
            None => Either::Right(Identifier::new(first.as_str()).unwrap()),
        }
    };
    (i, e)
}

fn otpin_graph_parse(p: Pair<'_, Rule>) -> (Identifier, NamedPin) {
    assert_eq!(p.as_rule(), Rule::otpin_graph);
    let mut l = p.into_inner();
    let o = l.next().unwrap();
    let n0 = l.next().unwrap();
    let o0 = l.next().unwrap();
    (
        Identifier::new(o.as_str()).unwrap(),
        (
            Identifier::new(n0.as_str()).unwrap(),
            Identifier::new(o0.as_str()).unwrap(),
        ),
    )
}

fn fingraph_parse(code: &str) -> FingraphParse {
    let lc = Ps::parse(Rule::fingraph, code).unwrap();
    let mut l = lc.into_iter().next().unwrap().into_inner();

    let name: Identifier = {
        let name = l.next().unwrap();
        assert_eq!(name.as_rule(), Rule::name);
        Identifier::new(name.as_str()).unwrap()
    };
    let inpin: Vec<Identifier> = {
        let inpins = l.next().unwrap();
        assert_eq!(inpins.as_rule(), Rule::in_graph);
        inpins
            .into_inner()
            .map(|p| {
                assert_eq!(p.as_rule(), Rule::name);
                Identifier::new(p.as_str()).unwrap()
            })
            .collect()
    };
    let otpin: Vec<(Identifier, NamedPin)> = {
        let otpins = l.next().unwrap();
        assert_eq!(otpins.as_rule(), Rule::ot_graph);
        otpins.into_inner().map(|p| otpin_graph_parse(p)).collect()
    };
    let lcs: Vec<(Identifier, Identifier, Vec<GraphPin>)> = {
        let mut v = vec![];
        for lcs in l {
            assert_eq!(lcs.as_rule(), Rule::lc_graph);
            let mut vs = lcs.into_inner();
            let name: Identifier = Identifier::new(vs.next().unwrap().as_str()).unwrap();
            let usename: Identifier = Identifier::new(vs.next().unwrap().as_str()).unwrap();
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
    next: Vec<(Pin, Pin)>,
    prev: Vec<(Pin, Pin)>,
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
        Identifier::new(name.as_str()).unwrap()
    };
    let initlc: Identifier = {
        let name = l.next().unwrap();
        assert_eq!(name.as_rule(), Rule::name);
        Identifier::new(name.as_str()).unwrap()
    };
    let next: Vec<(Pin, Pin)> = {
        let otpin = l.next().unwrap();
        assert_eq!(otpin.as_rule(), Rule::next_iter);
        otpin
            .into_inner()
            .map(|p| {
                assert_eq!(p.as_rule(), Rule::conn_iter);
                let (i, i0) = conn_iter_parse(p);
                (Identifier::new(i).unwrap(), Identifier::new(i0).unwrap())
            })
            .collect()
    };
    let prev: Vec<(Pin, Pin)> = {
        let otpin = l.next().unwrap();
        assert_eq!(otpin.as_rule(), Rule::prev_iter);
        otpin
            .into_inner()
            .map(|p| {
                assert_eq!(p.as_rule(), Rule::conn_iter);
                let (i, i0) = conn_iter_parse(p);
                (Identifier::new(i).unwrap(), Identifier::new(i0).unwrap())
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
                let mut inpin_maps: Vec<(Identifier, NamedPin)> = vec![];
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

                let graphlc =
                    LogicCircuit::new_mix(name.clone(), new_lcs, edges, inpin_maps, otpin_maps)?;
                maps.insert((name, graphlc));
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
    match maps.get(&Identifier::new("main").unwrap()) {
        Some(lc) => Ok(lc.clone()),
        None => bail!("not found main"),
    }
}

pub fn parse_main(code: &str) -> Result<LogicCircuit> {
    let mut maps: List = init_maps();
    parse(code, &mut maps)?;
    match maps.get(&Identifier::new("main").unwrap()) {
        Some(lc) => Ok(lc.clone()),
        None => bail!("not found main"),
    }
}
