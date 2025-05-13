use crate::machine::WhileStatement;
use anyhow::Result;
use pest::{iterators::Pair, Parser};
use utils::variable::Var;

#[derive(pest_derive::Parser)]
#[grammar = "while_minus_language.pest"]
struct Ps;

pub fn parse_name(ps: Pair<Rule>) -> Var {
    debug_assert!(ps.as_rule() == Rule::name);
    let name = ps.as_str();
    name.into()
}

pub fn parse_one_statement(ps: Pair<Rule>) -> Result<WhileStatement> {
    debug_assert!(ps.as_rule() == Rule::statement);
    let mut ps = ps.into_inner();
    let p = ps.next().unwrap();
    let statement = match p.as_rule() {
        Rule::inc_statement => {
            let mut p = p.into_inner();
            // take one var
            let var = p.next().unwrap();
            let var: Var = parse_name(var);
            WhileStatement::inc(var)
        }
        Rule::dec_statement => {
            let mut p = p.into_inner();

            // take one var
            let var = p.next().unwrap();
            let var: Var = parse_name(var);
            WhileStatement::dec(var)
        }
        Rule::clr_statement => {
            let mut p = p.into_inner();

            // take one var
            let var = p.next().unwrap();
            let var: Var = parse_name(var);
            WhileStatement::clr(var)
        }
        Rule::cpy_statement => {
            let mut p = p.into_inner();

            // take two var
            let var0 = p.next().unwrap();
            let var0: Var = parse_name(var0);
            let var1 = p.next().unwrap();
            let var1: Var = parse_name(var1);
            WhileStatement::cpy(var0, var1)
        }
        Rule::while_statement => {
            let mut p = p.into_inner();

            // while `var``
            let var = p.next().unwrap();
            let var: Var = parse_name(var);
            WhileStatement::while_not_zero(var)
        }
        Rule::while_end => {
            let _ = p.into_inner();
            WhileStatement::while_end()
        }
        _ => {
            return Err(anyhow::anyhow!(
                "unreachable {} {:?}",
                p.as_str(),
                p.as_rule()
            ));
        }
    };
    Ok(statement)
}

pub fn program(code: &str) -> Result<Vec<WhileStatement>> {
    let mut code = Ps::parse(Rule::program, code)?;
    let code = code.next().unwrap();
    let code = code.into_inner();
    let mut statements = vec![];
    for p in code {
        let statement = parse_one_statement(p)?;
        statements.push(statement);
    }
    Ok(statements)
}

pub fn program_read_to_end(code: &str) -> Result<Vec<WhileStatement>> {
    let mut code = Ps::parse(Rule::program_read_to_end, code)?;
    let code = code.next().unwrap();
    let mut code = code.into_inner();
    let p = code.next().unwrap();
    assert!(p.as_rule() == Rule::program);

    program(p.as_str())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn parse_simple() {
        // empty code is acceptable
        let code = "";
        assert!(program(code).is_ok());
        assert!(program_read_to_end(code).is_ok());

        // one line with no \n
        let code = "inc x";
        assert!(program(code).is_ok());
        assert!(program_read_to_end(code).is_ok());

        // one line with \n
        let code = "inc x \n";
        assert!(program(code).is_ok());
        assert!(program_read_to_end(code).is_ok());

        // two line
        let code = "inc x\n inc x";
        assert!(program(code).is_ok());
        assert!(program_read_to_end(code).is_ok());

        // one line start with space
        let code = " inc x";
        assert!(program(code).is_ok());
        assert!(program_read_to_end(code).is_ok());
    }
    #[test]
    fn parse_comment() {
        // code start with empty line
        let code = "\n\n\n";
        let statements = program(code).unwrap();
        assert_eq!(statements.len(), 0);

        // code with empty line in the middle of lines
        let code = "inc x\n\n\ninc x\n";
        let statements = program(code).unwrap();
        assert_eq!(statements.len(), 2);
        assert_eq!(statements[0], WhileStatement::inc("x".into()));
        assert_eq!(statements[1], WhileStatement::inc("x".into()));

        // code start with comment
        let code = "// this is a comment\n";
        let statements = program(code).unwrap();
        assert_eq!(statements.len(), 0);

        // code with comment at the end
        let code = "inc x // this is a comment\n";
        let statements = program(code).unwrap();
        assert_eq!(statements.len(), 1);
        assert_eq!(statements[0], WhileStatement::inc("x".into()));
    }
    #[test]
    fn parse_test_each_stmt() {
        // test each statement

        // inc
        let code = "inc x\n";
        let statements = program(code).unwrap();
        assert_eq!(statements.len(), 1);
        assert_eq!(statements[0], WhileStatement::inc("x".into()));

        // dec
        let code = "dec x\n";
        let statements = program(code).unwrap();
        assert_eq!(statements.len(), 1);
        assert_eq!(statements[0], WhileStatement::dec("x".into()));

        // clr
        let code = "clr x\n";
        let statements = program(code).unwrap();
        assert_eq!(statements.len(), 1);
        assert_eq!(statements[0], WhileStatement::clr("x".into()));

        // cpy
        let code = "cpy x <- y\n";
        let statements = program(code).unwrap();
        assert_eq!(statements.len(), 1);
        assert_eq!(statements[0], WhileStatement::cpy("x".into(), "y".into()));

        // while
        let code = "while_nz x {\n";
        let statements = program(code).unwrap();
        assert_eq!(statements.len(), 1);
        assert_eq!(statements[0], WhileStatement::while_not_zero("x".into()));

        // while end
        let code = "}\n";
        let statements = program(code).unwrap();
        assert_eq!(statements.len(), 1);
        assert_eq!(statements[0], WhileStatement::while_end());
    }
    #[test]
    fn parse_test() {
        let code = "inc x\n\
                    dec y\n\
                    cpy x  <- y\n\
                    while_nz x {\n\
                        inc x\n\
                        dec y\n\
                    }\n";
        let statements = program(code).unwrap();
        assert_eq!(statements.len(), 7);
        assert_eq!(statements[0], WhileStatement::inc("x".into()));
        assert_eq!(statements[1], WhileStatement::dec("y".into()));
        assert_eq!(statements[2], WhileStatement::cpy("x".into(), "y".into()));
        assert_eq!(statements[3], WhileStatement::while_not_zero("x".into()));
        assert_eq!(statements[4], WhileStatement::inc("x".into()));
        assert_eq!(statements[5], WhileStatement::dec("y".into()));
        assert_eq!(statements[6], WhileStatement::while_end());
    }
    #[test]
    fn parse_fail_test() {
        let code = "i";
        // result is ok
        // because pest does not consume the input
        let result = program(code);
        assert!(result.is_ok());

        // result is error
        // because EOI(end of input) can not be consumed
        let result = program_read_to_end(code);
        assert!(result.is_err());
    }
}
