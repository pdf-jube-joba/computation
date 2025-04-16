use crate::machine::*;
use anyhow::{bail, Ok, Result};
use either::Either;
use pest::{iterators::Pair, Parser};
use utils::bool::Bool;

#[derive(pest_derive::Parser)]
#[grammar = "logic_circuit.pest"]
struct Ps;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct List(pub Vec<(Name, LoC)>);

impl<T> From<T> for List
where
    T: IntoIterator<Item = (Name, LoC)>,
{
    fn from(value: T) -> Self {
        List(value.into_iter().collect())
    }
}

impl List {
    pub fn get(&self, name: &Name) -> Option<&LoC> {
        self.0
            .iter()
            .find_map(|v| if &v.0 == name { Some(&v.1) } else { None })
    }
    pub fn insert(&mut self, name_loc: (Name, LoC)) -> Option<()> {
        if self.get(&name_loc.0).is_none() {
            self.0.push(name_loc);
            Some(())
        } else {
            None
        }
    }
}

// contains fundamental gate
pub fn init_maps() -> List {
    vec![
        ("NOT-T".into(), LoC::notgate(Bool::T)),
        ("NOT-F".into(), LoC::notgate(Bool::F)),
        ("AND-T".into(), LoC::andgate(Bool::T)),
        ("AND-F".into(), LoC::andgate(Bool::F)),
        ("OR-T".into(), LoC::orgate(Bool::T)),
        ("OR-F".into(), LoC::orgate(Bool::F)),
        ("CST-T".into(), LoC::cstgate(Bool::T)),
        ("CST-F".into(), LoC::cstgate(Bool::F)),
        ("BR-T".into(), LoC::brgate(Bool::T)),
        ("BR-F".into(), LoC::brgate(Bool::F)),
        ("END".into(), LoC::endgate()),
        ("DLY-T".into(), LoC::delaygate(Bool::T)),
        ("DLY-F".into(), LoC::delaygate(Bool::F)),
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
                    // なんか書いてたらいらなくなってしまった。
                    inpin: _,
                    otpin,
                    lcs,
                } = fingraph_parse(lc.as_str());
                eprintln!("{name}");
                let mut new_lcs = vec![];
                let mut edges = vec![];
                let mut inpins = vec![];
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
                            Either::Right(i) => inpins.push((i, (lcname.clone(), inpin))),
                        }
                    }
                }
                // let mut v = vec![];
                // let mut e = vec![];
                // for (lcname, usename, inout) in lcs {
                //     v.push((lcname.clone(), c.clone()));
                //     for (i, (n, o)) in inout {
                //         e.push(((n.clone(), o.clone()), (lcname.clone(), i.clone())));
                //     }
                // }
                let graphlc = LoC::new_graph(name.clone(), new_lcs, edges, inpins, otpin)?;
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
                let iterlc = LoC::new_iter(name.clone(), initlc.clone(), next, prev)?;
                maps.insert((name, iterlc));
            }
            _ => {
                assert_eq!(lc.as_str(), "");
            }
        }
    }
    Ok(())
}

pub fn parse_main_with_maps(code: &str, mut maps: List) -> Result<LoC> {
    parse(code, &mut maps)?;
    match maps.get(&"main".into()) {
        Some(lc) => Ok(lc.clone()),
        None => bail!("not found main"),
    }
}

pub fn parse_main(code: &str) -> Result<LoC> {
    let mut maps: List = init_maps();
    parse(code, &mut maps)?;
    match maps.get(&"main".into()) {
        Some(lc) => Ok(lc.clone()),
        None => bail!("not found main"),
    }
}

type Pin = (InPin, Either<(Name, OtPin), InPin>);

#[derive(Debug)]
struct FingraphParse {
    name: Name,
    #[allow(unused)]
    inpin: Vec<InPin>,
    otpin: Vec<(OtPin, (Name, OtPin))>,
    lcs: Vec<(Name, Name, Vec<Pin>)>,
}

fn conn_graph_parse(p: Pair<'_, Rule>) -> Pin {
    assert_eq!(p.as_rule(), Rule::conn_graph);
    let mut l = p.into_inner();
    let i = l.next().unwrap().as_str().into();
    let e = {
        let e = l.next().unwrap();
        assert_eq!(e.as_rule(), Rule::pin);
        let mut e = e.into_inner();
        let first = e.next().unwrap();
        let second = e.next();
        match second {
            Some(i) => Either::Left((first.as_str().into(), i.as_str().into())),
            None => Either::Right(first.as_str().into()),
        }
    };
    (i, e)
}

fn otpin_graph_parse(p: Pair<'_, Rule>) -> (OtPin, (Name, OtPin)) {
    assert_eq!(p.as_rule(), Rule::otpin_graph);
    let mut l = p.into_inner();
    let o = l.next().unwrap();
    let n0 = l.next().unwrap();
    let o0 = l.next().unwrap();
    (o.as_str().into(), (n0.as_str().into(), o0.as_str().into()))
}

fn fingraph_parse(code: &str) -> FingraphParse {
    let lc = Ps::parse(Rule::fingraph, code).unwrap();
    let mut l = lc.into_iter().next().unwrap().into_inner();

    let name: Name = {
        let name = l.next().unwrap();
        assert_eq!(name.as_rule(), Rule::name);
        name.as_str().into()
    };
    let inpin: Vec<InPin> = {
        let inpins = l.next().unwrap();
        assert_eq!(inpins.as_rule(), Rule::in_graph);
        inpins
            .into_inner()
            .map(|p| {
                assert_eq!(p.as_rule(), Rule::name);
                p.as_str().into()
            })
            .collect()
    };
    let otpin: Vec<(OtPin, (Name, OtPin))> = {
        let inpins = l.next().unwrap();
        assert_eq!(inpins.as_rule(), Rule::ot_graph);
        inpins.into_inner().map(|p| otpin_graph_parse(p)).collect()
    };
    let lcs: Vec<(Name, Name, Vec<Pin>)> = {
        let mut v = vec![];
        for lcs in l {
            assert_eq!(lcs.as_rule(), Rule::lc_graph);
            let mut vs = lcs.into_inner();
            let name: Name = vs.next().unwrap().as_str().into();
            let usename: Name = vs.next().unwrap().as_str().into();
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
    name: Name,
    initlc: Name,
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
