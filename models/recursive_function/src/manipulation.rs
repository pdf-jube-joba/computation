use pest::{iterators::Pair, Parser};
use std::collections::HashMap;

use crate::machine::RecursiveFunctions;

#[derive(pest_derive::Parser)]
#[grammar = "parse.pest"]
pub struct Ps;

fn parse_func(
    pair: Pair<Rule>,
    map: &HashMap<String, RecursiveFunctions>,
) -> Result<RecursiveFunctions, String> {
    debug_assert!(pair.as_rule() == Rule::func);
    let mut inner = pair.into_inner();
    let p = inner.next().unwrap();
    match p.as_rule() {
        Rule::zero => {
            debug_assert!(p.as_str() == "ZERO");
            Ok(RecursiveFunctions::zero())
        }
        Rule::succ => {
            debug_assert!(p.as_str() == "SUCC");
            Ok(RecursiveFunctions::succ())
        }
        Rule::proj => {
            let mut inner = p.into_inner();
            let length = inner.next().unwrap().as_str().parse::<usize>().unwrap();
            let number = inner.next().unwrap().as_str().parse::<usize>().unwrap();
            RecursiveFunctions::projection(length, number)
        }
        Rule::comp => {
            let mut ps = p.into_inner();
            let outer = parse_func(ps.next().unwrap(), map)?;
            let inner_funcs: Vec<RecursiveFunctions> = ps
                .map(|pair| {
                    debug_assert!(pair.as_rule() == Rule::func);
                    parse_func(pair, map)
                })
                .collect::<Result<_, _>>()?;
            RecursiveFunctions::composition(outer, inner_funcs)
        }
        Rule::prim => {
            let mut inner = p.into_inner();
            let zero = parse_func(inner.next().unwrap(), map)?;
            let succ = parse_func(inner.next().unwrap(), map)?;
            RecursiveFunctions::primitive_recursion(zero, succ)
        }
        Rule::muop => {
            let mut inner = p.into_inner();
            let muop = parse_func(inner.next().unwrap(), map)?;
            RecursiveFunctions::muoperator(muop)
        }
        Rule::name => {
            let name = p.as_str().to_string();
            if let Some(func) = map.get(&name) {
                return Ok(func.clone());
            }
            Err(format!("Function {name} not found"))
        }
        _ => {
            unreachable!("??? {}", p.as_str());
        }
    }
}

fn parse_let_statement(
    pair: Pair<Rule>,
    map: &HashMap<String, RecursiveFunctions>,
) -> Result<(String, RecursiveFunctions), String> {
    debug_assert!(pair.as_rule() == Rule::let_statement);
    let mut inner = pair.into_inner();
    let name = inner.next().unwrap().as_str().to_string();
    let func = parse_func(inner.next().unwrap(), map)?;
    if map.contains_key(&name) {
        return Err(format!("Function {name} already exists"));
    }
    Ok((name, func))
}

pub fn parse(str: &str) -> Result<RecursiveFunctions, String> {
    let mut map = HashMap::new();
    let mut pairs = Ps::parse(Rule::program, str).map_err(|err| format!("{err}"))?;
    let mut pairs = pairs.next().unwrap().into_inner();

    while pairs.peek().unwrap().as_rule() == Rule::let_statement {
        let pair = pairs.next().unwrap();
        let (name, func) = parse_let_statement(pair, &map)?;
        map.insert(name, func);
    }

    let main_func = parse_func(pairs.next().unwrap(), &map)?;

    Ok(main_func)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn parse_test_simple() {
        let code = "ZERO";
        let func = parse(code).unwrap();
        assert_eq!(func, RecursiveFunctions::zero());

        let code = "SUCC";
        let func = parse(code).unwrap();
        assert_eq!(func, RecursiveFunctions::succ());

        let code = "PROJ[3, 0]";
        let func = parse(code).unwrap();
        assert_eq!(func, RecursiveFunctions::projection(3, 0).unwrap());
    }
    #[test]
    fn parse_test_rec() {
        let code = "COMP[SUCC: (ZERO)]";
        let func = parse(code).unwrap();
        assert_eq!(
            func,
            RecursiveFunctions::composition(
                RecursiveFunctions::succ(),
                vec![RecursiveFunctions::zero()],
            )
            .unwrap()
        );

        let code = "PRIM[z: ZERO s: PROJ[2, 0] ]";
        let func = parse(code).unwrap();
        assert_eq!(
            func,
            RecursiveFunctions::primitive_recursion(
                RecursiveFunctions::zero(),
                RecursiveFunctions::projection(2, 0).unwrap()
            )
            .unwrap()
        );

        let code = "MUOP[SUCC]";
        let func = parse(code).unwrap();
        assert_eq!(
            func,
            RecursiveFunctions::muoperator(RecursiveFunctions::succ()).unwrap()
        );
    }
    #[test]
    fn parse_test_rec2() {
        let code = "MUOP[MUOP[PROJ[2, 0]]]";
        let func = parse(code).unwrap();
        assert_eq!(
            func,
            RecursiveFunctions::muoperator(
                RecursiveFunctions::muoperator(RecursiveFunctions::projection(2, 0).unwrap())
                    .unwrap()
            )
            .unwrap()
        );

        let code = "COMP[PROJ[2, 0]: (MUOP[SUCC], MUOP[SUCC])]";
        let func = parse(code).unwrap();
        assert_eq!(
            func,
            RecursiveFunctions::composition(
                RecursiveFunctions::projection(2, 0).unwrap(),
                vec![
                    RecursiveFunctions::muoperator(RecursiveFunctions::succ()).unwrap(),
                    RecursiveFunctions::muoperator(RecursiveFunctions::succ()).unwrap()
                ]
            )
            .unwrap()
        );
    }
    #[test]
    fn parse_test_with_name() {
        let code = "let f = ZERO.\nf";
        let func = parse(code).unwrap();
        assert_eq!(func, RecursiveFunctions::zero());

        let code = "let f = ZERO.\nlet g = SUCC.\nf";
        let func = parse(code).unwrap();
        assert_eq!(func, RecursiveFunctions::zero());

        let code = "let f = PROJ[2,1].\nlet g = SUCC.\nCOMP[f: (g, g)]";
        let func = parse(code).unwrap();
        assert_eq!(
            func,
            RecursiveFunctions::composition(
                RecursiveFunctions::projection(2, 1).unwrap(),
                vec![RecursiveFunctions::succ(), RecursiveFunctions::succ()]
            )
            .unwrap()
        );
    }
}
