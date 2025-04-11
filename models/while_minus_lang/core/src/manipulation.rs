use crate::machine::{WhileLanguage, WhileStatement};
use anyhow::Result;
use pest::{iterators::Pair, Parser};
use utils::variable::Var;

#[derive(pest_derive::Parser)]
#[grammar = "while_minus_language.pest"]
struct Ps;

pub fn parse_name(ps: Pair<Rule>) -> Var {
    let name = ps.as_str();
    name.into()
}

pub fn parse_one_statement(code: &str) -> Result<WhileStatement> {
    let mut code = Ps::parse(Rule::statement, code)?;
    let p = code.next().unwrap();
    let rule = p.as_rule();
    let mut l = p.into_inner();
    let statement = match rule {
        Rule::inc_statement => {
            // take one var
            let var = l.next().unwrap();
            let var: Var = parse_name(var);
            WhileStatement::inc(var)
        }
        Rule::dec_statement => {
            // take one var
            let var = l.next().unwrap();
            let var: Var = parse_name(var);
            WhileStatement::dec(var)
        }
        Rule::clr_statement => {
            // take one var
            let var = l.next().unwrap();
            let var: Var = parse_name(var);
            WhileStatement::clr(var)
        }
        Rule::cpy_statement => {
            // take two var
            let var0 = l.next().unwrap();
            let var0: Var = parse_name(var0);
            let var1 = l.next().unwrap();
            let var1: Var = parse_name(var1);
            WhileStatement::cpy(var0, var1)
        }
        Rule::while_statement => {
            // while `var` { statements* }
            let var = l.next().unwrap();
            let var: Var = parse_name(var);
            let mut statements = vec![];
            for statement in l {
                assert!(statement.as_rule() == Rule::statement);
                let statement = parse_one_statement(statement.as_str()).unwrap();
                statements.push(statement);
            }
            WhileStatement::while_not_zero(var, statements)
        }
        _ => {
            unreachable!()
        }
    };
    Ok(statement)
}

pub fn program(code: &str) -> Result<WhileLanguage> {
    let code = Ps::parse(Rule::program, code)?;
    let mut statements = vec![];
    for p in code {
        assert!(p.as_rule() == Rule::statement);
        let statement = parse_one_statement(p.as_str()).unwrap();
        statements.push(statement);
    }
    Ok(WhileLanguage::new(statements))
}
