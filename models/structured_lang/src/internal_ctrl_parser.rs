use utils::identifier::Identifier;
use utils::number::Number;
use utils::TextCodec;

use crate::internal_ctrl::{
    ABinOp, AExp, Atom, BExp, Environment, InternalCtrlCode, RelOp, Stmt,
};

#[derive(Debug, Clone, PartialEq, Eq)]
enum Token {
    Ident(String),
    Number(String),
    Static,
    Nop,
    If,
    Break,
    Continue,
    Loop,
    LParen,
    RParen,
    LBrace,
    RBrace,
    Semi,
    Colon,
    Comma,
    Assign,
    Arrow,
    Plus,
    Minus,
    Lt,
    Eq,
    Gt,
}

impl TextCodec for Environment {
    fn parse(text: &str) -> Result<Self, String> {
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

    fn write_fmt(&self, f: &mut impl std::fmt::Write) -> std::fmt::Result {
        for (var, value) in &self.vars {
            writeln!(f, "{} = {}", var.as_str(), value.to_decimal_string())?;
        }
        Ok(())
    }
}

impl TextCodec for InternalCtrlCode {
    fn parse(text: &str) -> Result<Self, String> {
        let mut parser = Parser::new(lex(text)?);
        parser.parse_program()
    }

    fn write_fmt(&self, f: &mut impl std::fmt::Write) -> std::fmt::Result {
        write!(f, "static")?;
        if !self.statics.is_empty() {
            write!(
                f,
                " {}",
                self.statics
                    .iter()
                    .map(|x| x.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            )?;
        }
        writeln!(f, "; {}", stmt_to_text(&self.body))
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
        let tok = self.tokens.get(self.pos).cloned();
        if tok.is_some() {
            self.pos += 1;
        }
        tok
    }

    fn expect(&mut self, token: Token) -> Result<(), String> {
        if self.next() == Some(token) {
            Ok(())
        } else {
            Err("Unexpected token".to_string())
        }
    }

    fn eat(&mut self, token: &Token) -> bool {
        if self.peek() == Some(token) {
            self.pos += 1;
            true
        } else {
            false
        }
    }

    fn parse_program(&mut self) -> Result<InternalCtrlCode, String> {
        self.expect(Token::Static)?;
        let mut statics = Vec::new();
        if self.peek() != Some(&Token::Semi) {
            statics.push(self.parse_ident()?);
            while self.eat(&Token::Comma) {
                statics.push(self.parse_ident()?);
            }
        }
        self.expect(Token::Semi)?;
        let body = self.parse_stmt()?;
        if self.pos != self.tokens.len() {
            return Err("Unexpected trailing tokens".to_string());
        }
        Ok(InternalCtrlCode { statics, body })
    }

    fn parse_stmt(&mut self) -> Result<Stmt, String> {
        let mut left = self.parse_stmt_single()?;
        while self.eat(&Token::Semi) {
            if self.peek().is_none() || self.peek() == Some(&Token::RBrace) {
                break;
            }
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
                let body = self.parse_block()?;
                Ok(Stmt::If {
                    cond,
                    body: Box::new(body),
                })
            }
            Some(Token::Break) => {
                self.next();
                let label = self.parse_label()?;
                let value = self.parse_ident()?;
                Ok(Stmt::Break { label, value })
            }
            Some(Token::Continue) => {
                self.next();
                let label = self.parse_label()?;
                Ok(Stmt::Continue { label })
            }
            Some(Token::Loop) => {
                self.next();
                let label = self.parse_label()?;
                self.expect(Token::LParen)?;
                let body = self.parse_stmt()?;
                self.expect(Token::RParen)?;
                self.expect(Token::Arrow)?;
                let out = self.parse_ident()?;
                Ok(Stmt::Loop {
                    label,
                    body: Box::new(body),
                    out,
                })
            }
            Some(Token::LBrace) => self.parse_block(),
            Some(Token::Ident(_)) => {
                let var = self.parse_ident()?;
                self.expect(Token::Assign)?;
                let expr = self.parse_aexp()?;
                Ok(Stmt::Assign { var, expr })
            }
            _ => Err("Invalid statement".to_string()),
        }
    }

    fn parse_block(&mut self) -> Result<Stmt, String> {
        self.expect(Token::LBrace)?;
        let stmt = self.parse_stmt()?;
        self.expect(Token::RBrace)?;
        Ok(stmt)
    }

    fn parse_aexp(&mut self) -> Result<AExp, String> {
        let lhs = self.parse_atom()?;
        if let Some(Token::Plus | Token::Minus) = self.peek() {
            let op = match self.next() {
                Some(Token::Plus) => ABinOp::Add,
                Some(Token::Minus) => ABinOp::Sub,
                _ => unreachable!(),
            };
            let rhs = self.parse_atom()?;
            Ok(AExp::Bin { lhs, op, rhs })
        } else {
            Ok(AExp::Atom(lhs))
        }
    }

    fn parse_bexp(&mut self) -> Result<BExp, String> {
        let lhs = self.parse_atom()?;
        let rel = match self.next() {
            Some(Token::Lt) => RelOp::Lt,
            Some(Token::Eq) => RelOp::Eq,
            Some(Token::Gt) => RelOp::Gt,
            _ => return Err("Expected relation operator".to_string()),
        };
        let rhs = self.parse_atom()?;
        Ok(BExp { lhs, rel, rhs })
    }

    fn parse_atom(&mut self) -> Result<Atom, String> {
        match self.next() {
            Some(Token::Ident(name)) => Ok(Atom::Var(Identifier::new(&name).map_err(|e| e.to_string())?)),
            Some(Token::Number(num)) => Ok(Atom::Imm(Number::parse(&num)?)),
            _ => Err("Expected atom".to_string()),
        }
    }

    fn parse_ident(&mut self) -> Result<Identifier, String> {
        match self.next() {
            Some(Token::Ident(name)) => Identifier::new(&name).map_err(|e| e.to_string()),
            _ => Err("Expected identifier".to_string()),
        }
    }

    fn parse_label(&mut self) -> Result<Identifier, String> {
        self.expect(Token::Colon)?;
        self.parse_ident()
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
                "static" => Token::Static,
                "Nop" => Token::Nop,
                "if" => Token::If,
                "break" => Token::Break,
                "continue" => Token::Continue,
                "loop" => Token::Loop,
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
            '{' => {
                chars.next();
                tokens.push(Token::LBrace);
            }
            '}' => {
                chars.next();
                tokens.push(Token::RBrace);
            }
            ';' => {
                chars.next();
                tokens.push(Token::Semi);
            }
            ':' => {
                chars.next();
                if chars.peek() == Some(&'=') {
                    chars.next();
                    tokens.push(Token::Assign);
                } else {
                    tokens.push(Token::Colon);
                }
            }
            ',' => {
                chars.next();
                tokens.push(Token::Comma);
            }
            '+' => {
                chars.next();
                tokens.push(Token::Plus);
            }
            '-' => {
                chars.next();
                if chars.peek() == Some(&'>') {
                    chars.next();
                    tokens.push(Token::Arrow);
                } else {
                    tokens.push(Token::Minus);
                }
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
            _ => return Err(format!("Unexpected character: {ch}")),
        }
    }
    Ok(tokens)
}

fn atom_to_text(atom: &Atom) -> String {
    match atom {
        Atom::Var(v) => v.as_str().to_string(),
        Atom::Imm(n) => n.to_decimal_string(),
    }
}

fn aexp_to_text(exp: &AExp) -> String {
    match exp {
        AExp::Atom(atom) => atom_to_text(atom),
        AExp::Bin { lhs, op, rhs } => {
            let op = match op {
                ABinOp::Add => "+",
                ABinOp::Sub => "-",
            };
            format!("{} {} {}", atom_to_text(lhs), op, atom_to_text(rhs))
        }
    }
}

fn bexp_to_text(exp: &BExp) -> String {
    let rel = match exp.rel {
        RelOp::Lt => "<",
        RelOp::Eq => "=",
        RelOp::Gt => ">",
    };
    format!("{} {} {}", atom_to_text(&exp.lhs), rel, atom_to_text(&exp.rhs))
}

pub fn stmt_to_text(stmt: &Stmt) -> String {
    match stmt {
        Stmt::Nop => "Nop".to_string(),
        Stmt::Seq(lhs, rhs) => format!("{}; {}", stmt_to_text(lhs), stmt_to_text(rhs)),
        Stmt::Assign { var, expr } => format!("{} := {}", var.as_str(), aexp_to_text(expr)),
        Stmt::If { cond, body } => format!("if {} {{ {} }}", bexp_to_text(cond), stmt_to_text(body)),
        Stmt::Break { label, value } => format!("break :{} {}", label.as_str(), value.as_str()),
        Stmt::Continue { label } => format!("continue :{}", label.as_str()),
        Stmt::Loop { label, body, out } => {
            format!("loop :{} ({}) -> {}", label.as_str(), stmt_to_text(body), out.as_str())
        }
    }
}
