use crate::machine::*;
use anyhow::Result;
use pest::Parser;
use utils::number::*;

#[derive(pest_derive::Parser)]
#[grammar = "counter_machine.pest"]
struct Ps;

pub fn parse(code: &str) -> Result<Code> {
    let code = Ps::parse(Rule::code, code)?;
    let v = code
        .into_iter()
        .map(|p| {
            let rule = p.as_rule();
            let mut l = p.into_inner();
            match rule {
                Rule::inc => {
                    let num = l.next().unwrap().as_str();
                    let num: Number = num.parse::<usize>().unwrap().into();
                    Operation::Inc(num.into())
                }
                Rule::dec => {
                    let num = l.next().unwrap().as_str();
                    let num: Number = num.parse::<usize>().unwrap().into();
                    Operation::Dec(num.into())
                }
                Rule::clr => {
                    let num = l.next().unwrap().as_str();
                    let num: Number = num.parse::<usize>().unwrap().into();
                    Operation::Clr(num.into())
                }
                Rule::cop => {
                    let num0 = l.next().unwrap().as_str();
                    let num0: Number = num0.parse::<usize>().unwrap().into();
                    let num1 = l.next().unwrap().as_str();
                    let num1: Number = num1.parse::<usize>().unwrap().into();
                    Operation::Copy(num0.into(), num1.into())
                }
                Rule::ifz => {
                    let num0 = l.next().unwrap().as_str();
                    let num0: Number = num0.parse::<usize>().unwrap().into();
                    let num1 = l.next().unwrap().as_str();
                    let num1: Number = num1.parse::<usize>().unwrap().into();
                    Operation::Ifz(num0.into(), num1.into())
                }
                _ => {
                    unreachable!()
                }
            }
        })
        .collect::<Vec<_>>();
    Ok(Code(v))
}
