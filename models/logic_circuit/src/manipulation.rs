use std::collections::{HashMap, HashSet};

use crate::machine::*;
use anyhow::{bail, Result};
use either::Either;
use pest::Parser;

#[derive(pest_derive::Parser)]
#[grammar = "logic_circuit.pest"]
struct Ps;

fn parse(code: &str) -> Result<LoC> {
    let lcs = Ps::parse(Rule::lcs, code)?;
    let mut maps: HashMap<String, LoC> = HashMap::new();
    for lc in lcs {
        // A pair is a combination of the rule which matched and a span of input
        match lc.as_rule() {
            Rule::fingraph => {
                let mut cont = lc.into_inner();
                let name: Name = {
                    let name = cont.next().unwrap();
                    assert_eq!(name.as_rule(), Rule::name);
                    name.as_str().into()
                };
                let indesc: Vec<(InPin, (Name, InPin))> = {
                    let indesc = cont.next().unwrap();
                    assert_eq!(indesc.as_rule(), Rule::in_desc);
                    let mut v = vec![];
                    for i in indesc.into_inner() {
                        assert_eq!(i.as_rule(), Rule::inout_connect);
                        let mut i = i.into_inner();
                        let (v0, v1, v2) = (
                            i.next().unwrap().as_str().into(),
                            i.next().unwrap().as_str().into(),
                            i.next().unwrap().as_str().into(),
                        );
                        v.push((v0, (v1, v2)));
                    }
                    eprintln!("{v:?}");
                    v
                };
                let otdesc: Vec<(OtPin, (Name, OtPin))> = {
                    let otdesc = cont.next().unwrap();
                    assert_eq!(otdesc.as_rule(), Rule::out_desc);
                    let mut v = vec![];
                    for i in otdesc.into_inner() {
                        assert_eq!(i.as_rule(), Rule::inout_connect);
                        let mut i = i.into_inner();
                        let (v0, v1, v2) = (
                            i.next().unwrap().as_str().into(),
                            i.next().unwrap().as_str().into(),
                            i.next().unwrap().as_str().into(),
                        );
                        v.push((v0, (v1, v2)));
                    }
                    eprintln!("{v:?}");
                    v
                };
                let lc_descs: Vec<(Name, Name, Vec<(InPin, (Name, OtPin))>)> = cont
                    .map(|p| {
                        assert_eq!(p.as_rule(), Rule::lc_desc);
                        let mut cont = p.into_inner();
                        let name: Name = {
                            let name = cont.next().unwrap();
                            assert_eq!(name.as_rule(), Rule::name);
                            name.as_str().into()
                        };
                        let usename: Name = {
                            let name = cont.next().unwrap();
                            assert_eq!(name.as_rule(), Rule::name);
                            name.as_str().into()
                        };
                        let v = {
                            let mut v = vec![];
                            for i in cont {
                                assert_eq!(i.as_rule(), Rule::inout_connect);
                                let mut i = i.into_inner();
                                let (v0, v1, v2) = (
                                    i.next().unwrap().as_str().into(),
                                    i.next().unwrap().as_str().into(),
                                    i.next().unwrap().as_str().into(),
                                );
                                v.push((v0, (v1, v2)));
                            }
                            eprintln!("{v:?}");
                            v
                        };
                        (name, usename, v)
                    })
                    .collect();
            }
            Rule::iterator => {
                let name = lc.into_inner().next().unwrap();
                eprintln!("{:?}", name.as_rule());
            }
            _ => unreachable!(),
        }
    }
    bail!("hello")
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn f() {
        let s = "graph: DFF {
            in {A=b.c, d=g.f }
            out {a=b.c}
            hello, DFF, I=O.I
          }";
        let c = parse(s);
    }
}
