use crate::machine::Command;
use utils::identifier::Identifier as Var;
use utils::number::Number;
use utils::{TextCodec, Token as LexToken, lex};

pub fn program(code: &str) -> Result<Vec<Command>, String> {
    let tokens = lex(code).map_err(|e| e.to_string())?;
    let mut parser = Parser::new(tokens);
    let mut statements = Vec::new();

    while !parser.is_eof() {
        statements.push(parser.parse_statement()?);
        parser.expect_symbol(';')?;
    }

    Ok(statements)
}

pub fn program_read_to_end(code: &str) -> Result<Vec<Command>, String> {
    program(code)
}

pub fn env_read_to_end(code: &str) -> Result<Vec<(Var, Number)>, String> {
    let tokens = lex(code).map_err(|e| e.to_string())?;
    let mut parser = Parser::new(tokens);
    let mut env = Vec::new();

    while !parser.is_eof() {
        let name = parser.parse_name()?;
        parser.expect_symbol('=')?;
        let number = parser.parse_number()?;
        env.push((name, number));
        parser.eat_symbol(';');
    }

    Ok(env)
}

struct Parser {
    tokens: Vec<LexToken>,
    pos: usize,
}

impl Parser {
    fn new(tokens: Vec<LexToken>) -> Self {
        let tokens = tokens
            .into_iter()
            .filter(|token| !matches!(token, LexToken::Whitespace(_) | LexToken::Comment(_)))
            .collect();
        Self { tokens, pos: 0 }
    }

    fn is_eof(&self) -> bool {
        self.pos >= self.tokens.len()
    }

    fn peek(&self) -> Option<&LexToken> {
        self.tokens.get(self.pos)
    }

    fn next(&mut self) -> Option<LexToken> {
        let token = self.tokens.get(self.pos).cloned();
        if token.is_some() {
            self.pos += 1;
        }
        token
    }

    fn error_here(&self, message: impl Into<String>) -> String {
        match self.peek() {
            Some(token) => format!("{} near {:?}", message.into(), token),
            None => format!("{} at end of input", message.into()),
        }
    }

    fn peek_ident(&self, word: &str) -> bool {
        matches!(self.peek(), Some(LexToken::Ident(found)) if found == word)
    }

    fn eat_ident(&mut self, word: &str) -> bool {
        if self.peek_ident(word) {
            self.pos += 1;
            true
        } else {
            false
        }
    }

    fn peek_symbol(&self, ch: char) -> bool {
        matches!(self.peek(), Some(LexToken::Symbol(found)) if *found == ch)
    }

    fn eat_symbol(&mut self, ch: char) -> bool {
        if self.peek_symbol(ch) {
            self.pos += 1;
            true
        } else {
            false
        }
    }

    fn expect_symbol(&mut self, ch: char) -> Result<(), String> {
        if self.eat_symbol(ch) {
            Ok(())
        } else {
            Err(self.error_here(format!("expected symbol '{ch}'")))
        }
    }

    fn parse_name(&mut self) -> Result<Var, String> {
        match self.next() {
            Some(LexToken::Ident(name)) => Var::new(name).map_err(|e| e.to_string()),
            _ => Err(self.error_here("expected identifier")),
        }
    }

    fn parse_number(&mut self) -> Result<Number, String> {
        match self.next() {
            Some(LexToken::Number(n)) => Number::parse(&n),
            _ => Err(self.error_here("expected number")),
        }
    }

    fn parse_statement(&mut self) -> Result<Command, String> {
        if self.eat_ident("inc") {
            return Ok(Command::Inc(self.parse_name()?));
        }
        if self.eat_ident("dec") {
            return Ok(Command::Dec(self.parse_name()?));
        }
        if self.eat_ident("clr") {
            return Ok(Command::Clr(self.parse_name()?));
        }
        if self.eat_ident("cpy") {
            let dst = self.parse_name()?;
            self.expect_symbol('<')?;
            self.expect_symbol('-')?;
            let src = self.parse_name()?;
            return Ok(Command::Cpy(dst, src));
        }
        if self.eat_ident("ifnz") {
            let var = self.parse_name()?;
            self.expect_symbol(':')?;
            let num = self.parse_number()?;
            return Ok(Command::Ifnz(var, num));
        }
        Err(self.error_here("expected statement"))
    }
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
        let code = "x = 10; y = 20; z = 30;";
        let env = env_read_to_end(code).unwrap();
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
        inc x;
        dec y;
        clr z;
        cpy x <- y;
        ifnz z : 0;
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
cpy y2 <- y;
inc z;
dec y2;
ifnz y2 : 1;
dec x;
ifnz x : 0;
";
        let commands = program_read_to_end(code).unwrap();
        assert_eq!(commands.len(), 6);
        for c in &commands {
            println!("{:?}", c);
        }

        let env = "x = 2; y = 3; z = 0;";
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
            match program.step(()).unwrap() {
                utils::StepResult::Continue { next, output: () } => {
                    program = next;
                }
                utils::StepResult::Halt { output } => {
                    print_env(&output);
                    break;
                }
            }
        }
    }
}
