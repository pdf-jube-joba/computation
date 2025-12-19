use crate::machine::Command;
use anyhow::Result;
use pest::{Parser, iterators::Pair};
use utils::{number::Number, variable::VarStr as Var};

#[derive(pest_derive::Parser)]
#[grammar = "goto_lang.pest"]
struct Ps;

pub fn parse_name(ps: Pair<Rule>) -> Var {
    debug_assert!(ps.as_rule() == Rule::name);
    let name = ps.as_str();
    Var::new(name)
}

pub fn parse_one_statement(ps: Pair<Rule>) -> Result<Command> {
    debug_assert!(ps.as_rule() == Rule::statement);
    let mut ps = ps.into_inner();
    let p = ps.next().unwrap();
    let statement = match p.as_rule() {
        Rule::inc_statement => {
            let mut p = p.into_inner();
            // take one var
            let var = p.next().unwrap();
            let var: Var = parse_name(var);
            Command::Inc(var)
        }
        Rule::dec_statement => {
            let mut p = p.into_inner();

            // take one var
            let var = p.next().unwrap();
            let var: Var = parse_name(var);
            Command::Dec(var)
        }
        Rule::clr_statement => {
            let mut p = p.into_inner();

            // take one var
            let var = p.next().unwrap();
            let var: Var = parse_name(var);
            Command::Clr(var)
        }
        Rule::cpy_statement => {
            let mut p = p.into_inner();

            // take two var
            let var0 = p.next().unwrap();
            let var0: Var = parse_name(var0);
            let var1 = p.next().unwrap();
            let var1: Var = parse_name(var1);
            Command::Cpy(var0, var1)
        }
        Rule::ifnz_statement => {
            let mut p = p.into_inner();

            // take var and number
            let var = p.next().unwrap();
            let var: Var = parse_name(var);
            let num = p.next().unwrap();
            let num: usize = num.as_str().parse().unwrap();
            let num: Number = num.into();
            Command::Ifnz(var, num)
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

pub fn program(code: &str) -> Result<Vec<Command>> {
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

pub fn program_read_to_end(code: &str) -> Result<Vec<Command>> {
    let mut code = Ps::parse(Rule::program_read_to_end, code)?;
    let code = code.next().unwrap();
    let mut code = code.into_inner();
    let p = code.next().unwrap();
    assert!(p.as_rule() == Rule::program);

    program(p.as_str())
}

pub fn parse_env(ps: Pair<Rule>) -> Result<Vec<(Var, Number)>> {
    debug_assert!(ps.as_rule() == Rule::env);
    let mut env = vec![];
    for p in ps.into_inner() {
        debug_assert!(p.as_rule() == Rule::env_one);
        let mut p = p.into_inner();
        let name = p.next().unwrap();
        let name: Var = parse_name(name);
        let number = p.next().unwrap();
        let number: usize = number.as_str().parse()?;
        let number: Number = number.into();
        env.push((name, number));
    }
    Ok(env)
}

pub fn env_read_to_end(code: &str) -> Result<Vec<(Var, Number)>> {
    let mut code = Ps::parse(Rule::env, code)?;
    let code = code.next().unwrap();
    parse_env(code)
}

#[cfg(test)]
mod tests {
    use utils::{Machine, TextCodec};

    fn print_env(env: &crate::machine::Environment) -> String {
        let mut s = String::new();
        for (var, num) in &env.env {
            s.push_str(&format!("{} = {} ", var.as_str(), num.print()));
        }
        s
    }

    use crate::machine::Code;

    use super::*;
    #[test]
    fn test_parse_env() {
        let code = "x = 10 y = 20 z = 30";
        let mut ps = Ps::parse(Rule::env, code).unwrap();
        let ps = ps.next().unwrap();
        let env = parse_env(ps).unwrap();
        assert_eq!(env.len(), 3);
        assert_eq!(env[0].0.as_str(), "x");
        assert_eq!(env[0].1, 10.into());
        assert_eq!(env[1].0.as_str(), "y");
        assert_eq!(env[1].1, 20.into());
        assert_eq!(env[2].0.as_str(), "z");
        assert_eq!(env[2].1, 30.into());
    }
    #[test]
    fn test_parse_code() {
        let code = "
        inc x
        dec y
        clr z
        cpy x <- y
        ifnz z : 0
        ";
        let commands = program_read_to_end(code).unwrap();
        assert_eq!(commands.len(), 5);
        match &commands[0] {
            Command::Inc(v) => assert_eq!(v.as_str(), "x"),
            _ => panic!("unexpected command"),
        }
        match &commands[1] {
            Command::Dec(v) => assert_eq!(v.as_str(), "y"),
            _ => panic!("unexpected command"),
        }
        match &commands[2] {
            Command::Clr(v) => assert_eq!(v.as_str(), "z"),
            _ => panic!("unexpected command"),
        }
        match &commands[3] {
            Command::Cpy(v1, v2) => {
                assert_eq!(v1.as_str(), "x");
                assert_eq!(v2.as_str(), "y");
            }
            _ => panic!("unexpected command"),
        }
        match &commands[4] {
            Command::Ifnz(v, n) => {
                assert_eq!(v.as_str(), "z");
                assert_eq!(*n, 0.into());
            }
            _ => panic!("unexpected command"),
        }
    }
    #[test]
    fn test2() {
        let code = "
cpy y2 <- y
inc z
dec y2
ifnz y2 : 1
dec x
ifnz x : 0
";
        let commands = program_read_to_end(code).unwrap();
        assert_eq!(commands.len(), 6);
        for c in &commands {
            println!("{:?}", c);
        }

        let env = "x = 2 y = 3 z = 0";
        let env = env_read_to_end(env).unwrap();
        for (v, n) in &env {
            println!("{} = {}", v.as_str(), n.print());
        }

        println!("--- Running Program ---");

        let mut program = crate::machine::Program {
            commands: Code(commands),
            pc: 0.into(),
            env: crate::machine::Environment::from(env),
        };

        for _ in 0..100 {
            let _ = program.step(());
            println!(
                "pc: {}, env: {}",
                program.pc.print(),
                print_env(&program.env)
            );
        }
    }
}
