use utils::identifier::Identifier;
use utils::number::Number;
use utils::TextCodec;

use crate::machine::*;

#[derive(Debug, Clone, PartialEq, Eq)]
enum Token {
    Ident(String),
    Number(String),
    Nop,
    If,
    Then,
    Else,
    While,
    LParen,
    RParen,
    LBrack,
    RBrack,
    Semi,
    Assign,
    Plus,
    Minus,
    Lt,
    Eq,
    Gt,
    OrOr,
    Bang,
}

pub fn parse_env(text: &str) -> Result<Environment, String> {
    let mut env = Environment::default();
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let Some((left, right)) = line.split_once('=') else {
            return Err(format!("Invalid env line: {line}"));
        };
        let name = Identifier::new(left.trim()).map_err(|e| e.to_string())?;
        let value = Number::parse(right.trim())?;
        env.set(name, value);
    }
    Ok(env)
}

impl TextCodec for Environment {
    fn parse(text: &str) -> Result<Self, String> {
        crate::manipulation::parse_env(text)
    }

    fn write_fmt(&self, f: &mut impl std::fmt::Write) -> std::fmt::Result {
        for (var, value) in &self.vars {
            writeln!(f, "{} = {}", var.as_str(), value.to_decimal_string())?;
        }
        Ok(())
    }
}

impl TextCodec for WhileCode {
    fn parse(text: &str) -> Result<Self, String> {
        let tokens = lex(text)?;
        let mut ps = Parser::new(tokens);
        let stmt = ps.parse_stmt()?;
        ps.expect_eof()?;
        Ok(WhileCode(stmt))
    }

    fn write_fmt(&self, f: &mut impl std::fmt::Write) -> std::fmt::Result {
        write!(f, "{}", stmt_to_text(&self.0))
    }
}

struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, pos: 0 }
    }

    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.pos)
    }

    fn next(&mut self) -> Option<Token> {
        let t = self.tokens.get(self.pos).cloned();
        if t.is_some() {
            self.pos += 1;
        }
        t
    }

    fn expect_eof(&self) -> Result<(), String> {
        if self.pos == self.tokens.len() {
            Ok(())
        } else {
            Err("Unexpected trailing tokens".to_string())
        }
    }

    fn eat(&mut self, tok: &Token) -> bool {
        if self.peek() == Some(tok) {
            self.pos += 1;
            true
        } else {
            false
        }
    }

    fn parse_stmt(&mut self) -> Result<Stmt, String> {
        let mut left = self.parse_stmt_single()?;
        while self.eat(&Token::Semi) {
            let right = self.parse_stmt_single()?;
            left = Stmt::Seq(Box::new(left), Box::new(right));
        }
        Ok(left)
    }

    fn parse_stmt_single(&mut self) -> Result<Stmt, String> {
        match self.peek() {
            Some(Token::Nop) => {
                self.next();
                Ok(Stmt::Nop)
            }
            Some(Token::If) => {
                self.next();
                let cond = self.parse_bexp()?;
                let body = self.parse_stmt_single()?;
                Ok(Stmt::If {
                    cond,
                    body: Box::new(body),
                })
            }
            Some(Token::While) => {
                self.next();
                let cond = self.parse_bexp()?;
                self.expect(Token::LBrack)?;
                let body = self.parse_stmt()?;
                self.expect(Token::RBrack)?;
                Ok(Stmt::While {
                    cond,
                    body: Box::new(body),
                })
            }
            Some(Token::LParen) => {
                self.next();
                let s = self.parse_stmt()?;
                self.expect(Token::RParen)?;
                Ok(s)
            }
            Some(Token::Ident(_)) => {
                let var = self.parse_ident()?;
                self.expect(Token::Assign)?;
                let expr = self.parse_aexp()?;
                Ok(Stmt::Assign { var, expr })
            }
            _ => Err("Invalid statement".to_string()),
        }
    }

    fn parse_aexp(&mut self) -> Result<AExp, String> {
        if self.eat(&Token::If) {
            let cond = self.parse_bexp()?;
            self.expect(Token::Then)?;
            let then_exp = self.parse_aexp()?;
            self.expect(Token::Else)?;
            let else_exp = self.parse_aexp()?;
            return Ok(AExp::IfThenElse {
                cond: Box::new(cond),
                then_exp: Box::new(then_exp),
                else_exp: Box::new(else_exp),
            });
        }

        let mut left = self.parse_aexp_atom()?;
        loop {
            let op = if self.eat(&Token::Plus) {
                Some(ABinOp::Add)
            } else if self.eat(&Token::Minus) {
                Some(ABinOp::Sub)
            } else {
                None
            };
            let Some(op) = op else {
                break;
            };
            let rhs = self.parse_aexp_atom()?;
            left = AExp::BinOp {
                lhs: Box::new(left),
                op,
                rhs: Box::new(rhs),
            };
        }
        Ok(left)
    }

    fn parse_aexp_atom(&mut self) -> Result<AExp, String> {
        match self.next() {
            Some(Token::Ident(s)) => Ok(AExp::Var(Identifier::new(&s).map_err(|e| e.to_string())?)),
            Some(Token::Number(n)) => Ok(AExp::Num(Number::parse(&n)?)),
            Some(Token::LParen) => {
                let exp = self.parse_aexp()?;
                self.expect(Token::RParen)?;
                Ok(exp)
            }
            _ => Err("Invalid aexp".to_string()),
        }
    }

    fn parse_bexp(&mut self) -> Result<BExp, String> {
        let mut left = self.parse_bexp_not()?;
        while self.eat(&Token::OrOr) {
            let rhs = self.parse_bexp_not()?;
            left = BExp::Or(Box::new(left), Box::new(rhs));
        }
        Ok(left)
    }

    fn parse_bexp_not(&mut self) -> Result<BExp, String> {
        if self.eat(&Token::Bang) {
            return Ok(BExp::Not(Box::new(self.parse_bexp_not()?)));
        }
        self.parse_bexp_atom()
    }

    fn parse_bexp_atom(&mut self) -> Result<BExp, String> {
        if self.eat(&Token::LParen) {
            let b = self.parse_bexp()?;
            self.expect(Token::RParen)?;
            return Ok(b);
        }
        let lhs = self.parse_aexp()?;
        let rel = if self.eat(&Token::Lt) {
            RelOp::Lt
        } else if self.eat(&Token::Eq) {
            RelOp::Eq
        } else if self.eat(&Token::Gt) {
            RelOp::Gt
        } else {
            return Err("Expected relation operator".to_string());
        };
        let rhs = self.parse_aexp()?;
        Ok(BExp::Rel {
            lhs: Box::new(lhs),
            rel,
            rhs: Box::new(rhs),
        })
    }

    fn parse_ident(&mut self) -> Result<Identifier, String> {
        match self.next() {
            Some(Token::Ident(s)) => Identifier::new(&s).map_err(|e| e.to_string()),
            _ => Err("Expected identifier".to_string()),
        }
    }

    fn expect(&mut self, tok: Token) -> Result<(), String> {
        if self.next() == Some(tok) {
            Ok(())
        } else {
            Err("Unexpected token".to_string())
        }
    }
}

fn lex(text: &str) -> Result<Vec<Token>, String> {
    let mut tokens = Vec::new();
    let mut chars = text.chars().peekable();
    while let Some(&ch) = chars.peek() {
        if ch.is_whitespace() {
            chars.next();
            continue;
        }
        if ch.is_ascii_digit() {
            let mut s = String::new();
            while let Some(&c) = chars.peek() {
                if c.is_ascii_digit() {
                    s.push(c);
                    chars.next();
                } else {
                    break;
                }
            }
            tokens.push(Token::Number(s));
            continue;
        }
        if ch.is_ascii_alphabetic() || ch == '_' {
            let mut s = String::new();
            while let Some(&c) = chars.peek() {
                if c.is_ascii_alphanumeric() || c == '_' || c == '-' {
                    s.push(c);
                    chars.next();
                } else {
                    break;
                }
            }
            tokens.push(match s.as_str() {
                "Nop" => Token::Nop,
                "if" => Token::If,
                "then" => Token::Then,
                "else" => Token::Else,
                "while" => Token::While,
                _ => Token::Ident(s),
            });
            continue;
        }
        match ch {
            '(' => {
                chars.next();
                tokens.push(Token::LParen);
            }
            ')' => {
                chars.next();
                tokens.push(Token::RParen);
            }
            '[' => {
                chars.next();
                tokens.push(Token::LBrack);
            }
            ']' => {
                chars.next();
                tokens.push(Token::RBrack);
            }
            ';' => {
                chars.next();
                tokens.push(Token::Semi);
            }
            '+' => {
                chars.next();
                tokens.push(Token::Plus);
            }
            '-' => {
                chars.next();
                tokens.push(Token::Minus);
            }
            '<' => {
                chars.next();
                tokens.push(Token::Lt);
            }
            '>' => {
                chars.next();
                tokens.push(Token::Gt);
            }
            '=' => {
                chars.next();
                tokens.push(Token::Eq);
            }
            '!' => {
                chars.next();
                tokens.push(Token::Bang);
            }
            ':' => {
                chars.next();
                if chars.next() == Some('=') {
                    tokens.push(Token::Assign);
                } else {
                    return Err("Expected '=' after ':'".to_string());
                }
            }
            '|' => {
                chars.next();
                if chars.next() == Some('|') {
                    tokens.push(Token::OrOr);
                } else {
                    return Err("Expected '|' after '|'".to_string());
                }
            }
            _ => return Err(format!("Unexpected character: {ch}")),
        }
    }
    Ok(tokens)
}

fn aexp_to_text(exp: &AExp) -> String {
    match exp {
        AExp::Var(v) => v.as_str().to_string(),
        AExp::Num(n) => n.to_decimal_string(),
        AExp::BinOp { lhs, op, rhs } => {
            let op = match op {
                ABinOp::Add => "+",
                ABinOp::Sub => "-",
            };
            format!("({} {} {})", aexp_to_text(lhs), op, aexp_to_text(rhs))
        }
        AExp::IfThenElse {
            cond,
            then_exp,
            else_exp,
        } => format!(
            "if {} then {} else {}",
            bexp_to_text(cond),
            aexp_to_text(then_exp),
            aexp_to_text(else_exp)
        ),
    }
}

fn bexp_to_text(exp: &BExp) -> String {
    match exp {
        BExp::Rel { lhs, rel, rhs } => {
            let rel = match rel {
                RelOp::Lt => "<",
                RelOp::Eq => "=",
                RelOp::Gt => ">",
            };
            format!("({} {} {})", aexp_to_text(lhs), rel, aexp_to_text(rhs))
        }
        BExp::Or(a, b) => format!("({} || {})", bexp_to_text(a), bexp_to_text(b)),
        BExp::Not(b) => format!("!{}", bexp_to_text(b)),
    }
}

pub fn stmt_to_text(stmt: &Stmt) -> String {
    match stmt {
        Stmt::Nop => "Nop".to_string(),
        Stmt::Assign { var, expr } => format!("{} := {}", var.as_str(), aexp_to_text(expr)),
        Stmt::Seq(a, b) => format!("{} ; {}", stmt_to_text(a), stmt_to_text(b)),
        Stmt::If { cond, body } => format!("if {} {}", bexp_to_text(cond), stmt_to_text(body)),
        Stmt::While { cond, body } => {
            format!("while {} [ {} ]", bexp_to_text(cond), stmt_to_text(body))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use utils::{Machine, StepResult};

    #[test]
    fn while_code_roundtrip() {
        let text = r#"
while x < 3 [
  if !(x = 1) y := if x < 2 then x + 1 else x - 1 ;
  x := x + 1
]
"#;
        let parsed = WhileCode::parse(text).unwrap();
        let printed = parsed.print();
        let reparsed = WhileCode::parse(&printed).unwrap();
        assert_eq!(parsed, reparsed);
    }

    #[test]
    fn smallstep_runs() {
        let code = WhileCode::parse("x := 0 ; while x < 3 [ x := x + 1 ]").unwrap();
        let env = Environment::default();
        let mut machine = WhileMachine::make(code, env).unwrap();
        for _ in 0..100 {
            match machine.step(()).unwrap() {
                StepResult::Continue { next, .. } => machine = next,
                StepResult::Halt { output, .. } => {
                    let x = output
                        .vars
                        .iter()
                        .find(|(k, _)| k.as_str() == "x")
                        .map(|(_, v)| v.clone())
                        .unwrap_or_default();
                    assert_eq!(x, Number::from(3usize));
                    return;
                }
            }
        }
        panic!("did not halt");
    }
}
