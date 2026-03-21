use serde::{Deserialize, Serialize};
use utils::identifier::Identifier;
use utils::{Machine, StepResult, TextCodec};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StrTreeCode(pub Stmt);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AExp {
    Mtoa(Box<TExp>),
    AtomChar(char),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TExp {
    Var(Identifier),
    Atom(Box<AExp>),
    Cons(Box<TExp>, Box<TExp>),
    Left(Box<TExp>),
    Right(Box<TExp>),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum BExp {
    IsAtom(TExp),
    Eq(TExp, TExp),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Stmt {
    Nop,
    Assign { var: Identifier, expr: TExp },
    Seq(Box<Stmt>, Box<Stmt>),
    IfEq { cond: BExp, body: Box<Stmt> },
    While { cond: BExp, body: Box<Stmt> },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Value {
    Atom(char),
    Cons(Box<Value>, Box<Value>),
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Environment {
    pub vars: Vec<(Identifier, Value)>,
}

impl Environment {
    pub fn get(&self, var: &Identifier) -> Option<Value> {
        self.vars
            .iter()
            .find_map(|(k, v)| if k == var { Some(v.clone()) } else { None })
    }

    pub fn set(&mut self, var: Identifier, value: Value) {
        if let Some((_, v)) = self.vars.iter_mut().find(|(k, _)| *k == var) {
            *v = value;
        } else {
            self.vars.push((var, value));
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StrTreeMachine {
    pub code: StrTreeCode,
    pub stmt: Stmt,
    pub env: Environment,
}

impl StrTreeMachine {
    fn eval_aexp(exp: &AExp, env: &Environment) -> Result<char, String> {
        match exp {
            AExp::Mtoa(exp) => match Self::eval_texp(exp, env)? {
                Value::Atom(ch) => Ok(ch),
                Value::Cons(_, _) => Err("mtoa expects an atom".to_string()),
            },
            AExp::AtomChar(ch) => Ok(*ch),
        }
    }

    fn eval_texp(exp: &TExp, env: &Environment) -> Result<Value, String> {
        match exp {
            TExp::Var(var) => env
                .get(var)
                .ok_or_else(|| format!("uninitialized variable: {}", var.as_str())),
            TExp::Atom(exp) => Ok(Value::Atom(Self::eval_aexp(exp, env)?)),
            TExp::Cons(lhs, rhs) => Ok(Value::Cons(
                Box::new(Self::eval_texp(lhs, env)?),
                Box::new(Self::eval_texp(rhs, env)?),
            )),
            TExp::Left(exp) => match Self::eval_texp(exp, env)? {
                Value::Atom(_) => Err("left expects a cons cell".to_string()),
                Value::Cons(lhs, _) => Ok(*lhs),
            },
            TExp::Right(exp) => match Self::eval_texp(exp, env)? {
                Value::Atom(_) => Err("right expects a cons cell".to_string()),
                Value::Cons(_, rhs) => Ok(*rhs),
            },
        }
    }

    fn eval_bexp(exp: &BExp, env: &Environment) -> Result<bool, String> {
        match exp {
            BExp::IsAtom(exp) => Ok(matches!(Self::eval_texp(exp, env)?, Value::Atom(_))),
            BExp::Eq(lhs, rhs) => Ok(Self::eval_texp(lhs, env)? == Self::eval_texp(rhs, env)?),
        }
    }

    fn small_step(stmt: Stmt, mut env: Environment) -> Result<(Stmt, Environment), String> {
        match stmt {
            Stmt::Nop => Ok((Stmt::Nop, env)),
            Stmt::Assign { var, expr } => {
                let value = Self::eval_texp(&expr, &env)?;
                env.set(var, value);
                Ok((Stmt::Nop, env))
            }
            Stmt::Seq(a, b) => {
                if matches!(&*a, Stmt::Nop) {
                    Ok((*b, env))
                } else {
                    let (next_a, next_env) = Self::small_step(*a, env)?;
                    Ok((Stmt::Seq(Box::new(next_a), b), next_env))
                }
            }
            Stmt::IfEq { cond, body } => {
                if Self::eval_bexp(&cond, &env)? {
                    Ok((*body, env))
                } else {
                    Ok((Stmt::Nop, env))
                }
            }
            Stmt::While { cond, body } => Ok((
                Stmt::IfEq {
                    cond: cond.clone(),
                    body: Box::new(Stmt::Seq(
                        body.clone(),
                        Box::new(Stmt::While { cond, body }),
                    )),
                },
                env,
            )),
        }
    }
}

impl Machine for StrTreeMachine {
    type Code = StrTreeCode;
    type AInput = Environment;
    type FOutput = Environment;
    type SnapShot = StrTreeMachine;
    type RInput = ();
    type ROutput = ();

    fn make(code: Self::Code, ainput: Self::AInput) -> Result<Self, String> {
        Ok(Self {
            stmt: code.0.clone(),
            code,
            env: ainput,
        })
    }

    fn step(self, _rinput: Self::RInput) -> Result<StepResult<Self>, String> {
        if matches!(self.stmt, Stmt::Nop) {
            return Ok(StepResult::Halt { output: self.env });
        }

        let (stmt, env) = Self::small_step(self.stmt, self.env)?;
        Ok(StepResult::Continue {
            next: Self {
                code: self.code,
                stmt,
                env,
            },
            output: (),
        })
    }

    fn snapshot(&self) -> Self::SnapShot {
        self.clone()
    }

    fn restore(snapshot: Self::SnapShot) -> Self {
        snapshot
    }

    fn render(snapshot: Self::SnapShot) -> utils::RenderState {
        let text = serde_json::to_string_pretty(&snapshot)
            .unwrap_or_else(|_| "failed to serialize snapshot".to_string());
        utils::render_state![utils::render_text!(text, title: "snapshot")]
    }
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
            let value = parse_value(right.trim())?;
            env.set(name, value);
        }
        Ok(env)
    }

    fn write_fmt(&self, f: &mut impl std::fmt::Write) -> std::fmt::Result {
        for (var, value) in &self.vars {
            writeln!(f, "{} = {}", var.as_str(), value_to_text(value))?;
        }
        Ok(())
    }
}

impl TextCodec for StrTreeCode {
    fn parse(text: &str) -> Result<Self, String> {
        let tokens = lex(text)?;
        let mut parser = Parser::new(tokens);
        let stmt = parser.parse_stmt()?;
        parser.expect_eof()?;
        Ok(StrTreeCode(stmt))
    }

    fn write_fmt(&self, f: &mut impl std::fmt::Write) -> std::fmt::Result {
        write!(f, "{}", stmt_to_text(&self.0))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Token {
    Ident(String),
    Char(char),
    Nop,
    IfEq,
    Then,
    End,
    While,
    IsAtom,
    Mtoa,
    Atom,
    Cons,
    Left,
    Right,
    LParen,
    RParen,
    LBrace,
    RBrace,
    Semi,
    Assign,
    EqEq,
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
        let token = self.tokens.get(self.pos).cloned();
        if token.is_some() {
            self.pos += 1;
        }
        token
    }

    fn eat(&mut self, tok: &Token) -> bool {
        if self.peek() == Some(tok) {
            self.pos += 1;
            true
        } else {
            false
        }
    }

    fn expect(&mut self, tok: Token) -> Result<(), String> {
        if self.next() == Some(tok) {
            Ok(())
        } else {
            Err("Unexpected token".to_string())
        }
    }

    fn expect_eof(&self) -> Result<(), String> {
        if self.pos == self.tokens.len() {
            Ok(())
        } else {
            Err("Unexpected trailing tokens".to_string())
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
            Some(Token::IfEq) => {
                self.next();
                let cond = self.parse_bexp()?;
                self.expect(Token::Then)?;
                let body = self.parse_stmt()?;
                self.expect(Token::End)?;
                Ok(Stmt::IfEq {
                    cond,
                    body: Box::new(body),
                })
            }
            Some(Token::While) => {
                self.next();
                let cond = self.parse_bexp()?;
                self.expect(Token::LBrace)?;
                let body = self.parse_stmt()?;
                self.expect(Token::RBrace)?;
                Ok(Stmt::While {
                    cond,
                    body: Box::new(body),
                })
            }
            Some(Token::LParen) => {
                self.next();
                let stmt = self.parse_stmt()?;
                self.expect(Token::RParen)?;
                Ok(stmt)
            }
            Some(Token::Ident(_)) => {
                let var = self.parse_ident()?;
                self.expect(Token::Assign)?;
                let expr = self.parse_texp()?;
                Ok(Stmt::Assign { var, expr })
            }
            _ => Err("Invalid statement".to_string()),
        }
    }

    fn parse_bexp(&mut self) -> Result<BExp, String> {
        if self.eat(&Token::IsAtom) {
            return Ok(BExp::IsAtom(self.parse_texp()?));
        }
        let lhs = self.parse_texp()?;
        self.expect(Token::EqEq)?;
        let rhs = self.parse_texp()?;
        Ok(BExp::Eq(lhs, rhs))
    }

    fn parse_aexp(&mut self) -> Result<AExp, String> {
        match self.peek() {
            Some(Token::Mtoa) => {
                self.next();
                Ok(AExp::Mtoa(Box::new(self.parse_texp()?)))
            }
            Some(Token::Char(_)) => match self.next() {
                Some(Token::Char(ch)) => Ok(AExp::AtomChar(ch)),
                _ => unreachable!(),
            },
            Some(Token::LParen) => {
                self.next();
                let exp = self.parse_aexp()?;
                self.expect(Token::RParen)?;
                Ok(exp)
            }
            _ => Err("Invalid aexp".to_string()),
        }
    }

    fn parse_texp(&mut self) -> Result<TExp, String> {
        match self.peek() {
            Some(Token::Ident(_)) => {
                let var = self.parse_ident()?;
                Ok(TExp::Var(var))
            }
            Some(Token::Atom) => {
                self.next();
                Ok(TExp::Atom(Box::new(self.parse_aexp()?)))
            }
            Some(Token::Cons) => {
                self.next();
                let lhs = self.parse_texp()?;
                let rhs = self.parse_texp()?;
                Ok(TExp::Cons(Box::new(lhs), Box::new(rhs)))
            }
            Some(Token::Left) => {
                self.next();
                Ok(TExp::Left(Box::new(self.parse_texp()?)))
            }
            Some(Token::Right) => {
                self.next();
                Ok(TExp::Right(Box::new(self.parse_texp()?)))
            }
            Some(Token::LParen) => {
                self.next();
                let exp = self.parse_texp()?;
                self.expect(Token::RParen)?;
                Ok(exp)
            }
            _ => Err("Invalid texp".to_string()),
        }
    }

    fn parse_ident(&mut self) -> Result<Identifier, String> {
        match self.next() {
            Some(Token::Ident(name)) => Identifier::new(name).map_err(|e| e.to_string()),
            _ => Err("Expected identifier".to_string()),
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
        if ch.is_ascii_alphabetic() || ch == '_' {
            let mut word = String::new();
            while let Some(&c) = chars.peek() {
                if c.is_ascii_alphanumeric() || c == '_' || c == '-' {
                    word.push(c);
                    chars.next();
                } else {
                    break;
                }
            }
            tokens.push(match word.as_str() {
                "Nop" => Token::Nop,
                "ifeq" => Token::IfEq,
                "then" => Token::Then,
                "end" => Token::End,
                "while" => Token::While,
                "is-atom" => Token::IsAtom,
                "mtoa" => Token::Mtoa,
                "atom" => Token::Atom,
                "cons" => Token::Cons,
                "left" => Token::Left,
                "right" => Token::Right,
                _ => Token::Ident(word),
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
                if chars.next() == Some('=') {
                    tokens.push(Token::Assign);
                } else {
                    return Err("Expected '=' after ':'".to_string());
                }
            }
            '=' => {
                chars.next();
                if chars.next() == Some('=') {
                    tokens.push(Token::EqEq);
                } else {
                    return Err("Expected second '='".to_string());
                }
            }
            '\'' => tokens.push(Token::Char(parse_char_literal(&mut chars)?)),
            _ => return Err(format!("Unexpected character: {ch}")),
        }
    }
    Ok(tokens)
}

fn parse_char_literal<I>(chars: &mut std::iter::Peekable<I>) -> Result<char, String>
where
    I: Iterator<Item = char>,
{
    if chars.next() != Some('\'') {
        return Err("Expected opening quote".to_string());
    }
    let ch = match chars.next() {
        Some('\\') => match chars.next() {
            Some('n') => '\n',
            Some('r') => '\r',
            Some('t') => '\t',
            Some('\\') => '\\',
            Some('\'') => '\'',
            Some(other) => other,
            None => return Err("Unterminated escape".to_string()),
        },
        Some(c) => c,
        None => return Err("Unterminated character literal".to_string()),
    };
    if chars.next() != Some('\'') {
        return Err("Expected closing quote".to_string());
    }
    Ok(ch)
}

fn char_to_text(ch: char) -> String {
    match ch {
        '\n' => "'\\n'".to_string(),
        '\r' => "'\\r'".to_string(),
        '\t' => "'\\t'".to_string(),
        '\\' => "'\\\\'".to_string(),
        '\'' => "'\\''".to_string(),
        other => format!("'{other}'"),
    }
}

fn aexp_to_text(exp: &AExp) -> String {
    match exp {
        AExp::Mtoa(exp) => format!("mtoa {}", texp_to_text(exp)),
        AExp::AtomChar(ch) => char_to_text(*ch),
    }
}

fn texp_to_text(exp: &TExp) -> String {
    match exp {
        TExp::Var(var) => var.as_str().to_string(),
        TExp::Atom(exp) => format!("atom {}", aexp_to_text(exp)),
        TExp::Cons(lhs, rhs) => format!("cons {} {}", texp_to_text(lhs), texp_to_text(rhs)),
        TExp::Left(exp) => format!("left {}", texp_to_text(exp)),
        TExp::Right(exp) => format!("right {}", texp_to_text(exp)),
    }
}

fn bexp_to_text(exp: &BExp) -> String {
    match exp {
        BExp::IsAtom(exp) => format!("is-atom {}", texp_to_text(exp)),
        BExp::Eq(lhs, rhs) => format!("({} == {})", texp_to_text(lhs), texp_to_text(rhs)),
    }
}

fn stmt_to_text(stmt: &Stmt) -> String {
    match stmt {
        Stmt::Nop => "Nop".to_string(),
        Stmt::Assign { var, expr } => format!("{} := {}", var.as_str(), texp_to_text(expr)),
        Stmt::Seq(lhs, rhs) => format!("{} ; {}", stmt_to_text(lhs), stmt_to_text(rhs)),
        Stmt::IfEq { cond, body } => format!("ifeq {} then {} end", bexp_to_text(cond), stmt_to_text(body)),
        Stmt::While { cond, body } => format!("while {} {{ {} }}", bexp_to_text(cond), stmt_to_text(body)),
    }
}

fn value_to_text(value: &Value) -> String {
    match value {
        Value::Atom(ch) => format!("atom {}", char_to_text(*ch)),
        Value::Cons(lhs, rhs) => format!("cons {} {}", value_to_text(lhs), value_to_text(rhs)),
    }
}

fn parse_value(text: &str) -> Result<Value, String> {
    let tokens = lex(text)?;
    let mut parser = Parser::new(tokens);
    let exp = parser.parse_texp()?;
    parser.expect_eof()?;
    fn lower(exp: TExp) -> Result<Value, String> {
        match exp {
            TExp::Var(var) => Err(format!("unexpected variable in value: {}", var.as_str())),
            TExp::Atom(exp) => match *exp {
                AExp::AtomChar(ch) => Ok(Value::Atom(ch)),
                AExp::Mtoa(_) => Err("unexpected mtoa in value".to_string()),
            },
            TExp::Cons(lhs, rhs) => Ok(Value::Cons(Box::new(lower(*lhs)?), Box::new(lower(*rhs)?))),
            TExp::Left(_) | TExp::Right(_) => Err("unexpected projection in value".to_string()),
        }
    }
    lower(exp)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn code_roundtrip() {
        let text = "x := atom 'a' ; ifeq is-atom x then y := cons x x end";
        let parsed = StrTreeCode::parse(text).unwrap();
        let printed = parsed.print();
        let reparsed = StrTreeCode::parse(&printed).unwrap();
        assert_eq!(parsed, reparsed);
    }

    #[test]
    fn machine_runs() {
        let code = StrTreeCode::parse("x := cons atom 'a' atom 'b' ; y := left x").unwrap();
        let mut machine = StrTreeMachine::make(code, Environment::default()).unwrap();
        loop {
            match machine.step(()).unwrap() {
                StepResult::Continue { next, .. } => machine = next,
                StepResult::Halt { output } => {
                    let y = output.get(&Identifier::new("y").unwrap()).unwrap();
                    assert_eq!(y, Value::Atom('a'));
                    return;
                }
            }
        }
    }
}
