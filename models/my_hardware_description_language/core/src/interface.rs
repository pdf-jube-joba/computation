use crate::machine::*;
use anyhow::{bail, Ok, Result};
use pest::{iterators::Pair, Parser};

const BOOL_T: &str = "#true";
const BOOL_F: &str = "#false";

#[derive(pest_derive::Parser)]
#[grammar = "my_HDL.pest"]
struct MyParser;

pub fn parse_value_rule(rule: Pair<Rule>) -> Result<Value> {
    if !matches!(rule.as_rule(), Rule::value) {
        bail!("rule is not value")
    }
    let l = rule.into_inner().next().unwrap();
    match l.as_rule() {
        Rule::bool_value => match l.into_inner().next().unwrap().as_rule() {
            Rule::true_bool => Ok(Value::Bit(utils::bool::Bool::T)),
            Rule::false_bool => Ok(Value::Bit(utils::bool::Bool::F)),
            _ => unreachable!(),
        },
        Rule::array_value => {
            let v = l
                .into_inner()
                .map(|p| parse_value_rule(p))
                .collect::<Result<Vec<_>>>()?;
            Ok(Value::Array(v))
        }
        Rule::struct_value => {
            let mut vc = vec![];
            let mut l = l.into_inner();
            while let Some(n) = l.next() {
                let name = n.as_str().to_string();
                let value = l.next().unwrap();
                let value = parse_value_rule(value)?;
                vc.push((name, value));
            }
            if let Some(value) = Value::new_strct(vc) {
                Ok(value)
            } else {
                bail!("field 名がかぶってそう")
            }
        }
        Rule::enum_value => {
            let name = l.into_inner().next().unwrap().as_str().to_string();
            Ok(Value::new_enume(name))
        }
        _ => unreachable!("parse を考えるとありえない"),
    }
}
