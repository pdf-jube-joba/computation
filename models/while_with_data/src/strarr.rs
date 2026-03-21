use serde::{Deserialize, Serialize};
use utils::identifier::Identifier;
use utils::{Machine, StepResult, TextCodec};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StrArrCode(pub Stmt);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AExp {
    Const(Vec<char>),
    Var(Identifier),
    Concat(Box<AExp>, Box<AExp>),
    Head(Box<AExp>),
    Tail(Box<AExp>),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum BExp {
    IsEmpty(AExp),
    Eq(AExp, AExp),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Stmt {
    Nop,
    Assign { var: Identifier, expr: AExp },
    Seq(Box<Stmt>, Box<Stmt>),
    If { cond: BExp, body: Box<Stmt> },
    While { cond: BExp, body: Box<Stmt> },
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Environment {
    pub vars: Vec<(Identifier, Vec<char>)>,
}

impl Environment {
    pub fn get(&self, var: &Identifier) -> Vec<char> {
        self.vars
            .iter()
            .find_map(|(k, v)| if k == var { Some(v.clone()) } else { None })
            .unwrap_or_default()
    }

    pub fn set(&mut self, var: Identifier, value: Vec<char>) {
        if let Some((_, v)) = self.vars.iter_mut().find(|(k, _)| *k == var) {
            *v = value;
        } else {
            self.vars.push((var, value));
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StrArrMachine {
    pub code: StrArrCode,
    pub stmt: Stmt,
    pub env: Environment,
}

impl StrArrMachine {
    fn eval_aexp(exp: &AExp, env: &Environment) -> Result<Vec<char>, String> {
        match exp {
            AExp::Const(chars) => Ok(chars.clone()),
            AExp::Var(var) => Ok(env.get(var)),
            AExp::Concat(lhs, rhs) => {
                let mut left = Self::eval_aexp(lhs, env)?;
                left.extend(Self::eval_aexp(rhs, env)?);
                Ok(left)
            }
            AExp::Head(exp) => {
                let value = Self::eval_aexp(exp, env)?;
                Ok(value.first().copied().into_iter().collect())
            }
            AExp::Tail(exp) => {
                let value = Self::eval_aexp(exp, env)?;
                Ok(value.into_iter().skip(1).collect())
            }
        }
    }

    fn eval_bexp(exp: &BExp, env: &Environment) -> Result<bool, String> {
        match exp {
            BExp::IsEmpty(exp) => Ok(Self::eval_aexp(exp, env)?.is_empty()),
            BExp::Eq(lhs, rhs) => Ok(Self::eval_aexp(lhs, env)? == Self::eval_aexp(rhs, env)?),
        }
    }

    fn small_step(stmt: Stmt, mut env: Environment) -> Result<(Stmt, Environment), String> {
        match stmt {
            Stmt::Nop => Ok((Stmt::Nop, env)),
            Stmt::Assign { var, expr } => {
                let value = Self::eval_aexp(&expr, &env)?;
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
            Stmt::If { cond, body } => {
                if Self::eval_bexp(&cond, &env)? {
                    Ok((*body, env))
                } else {
                    Ok((Stmt::Nop, env))
                }
            }
            Stmt::While { cond, body } => Ok((
                Stmt::If {
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

impl Machine for StrArrMachine {
    type Code = StrArrCode;
    type AInput = Environment;
    type FOutput = Environment;
    type SnapShot = StrArrMachine;
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
            let value = parse_char_list(right.trim())?;
            env.set(name, value);
        }
        Ok(env)
    }

    fn write_fmt(&self, f: &mut impl std::fmt::Write) -> std::fmt::Result {
        for (var, value) in &self.vars {
            writeln!(f, "{} = {}", var.as_str(), char_list_to_text(value))?;
        }
        Ok(())
    }
}

impl TextCodec for StrArrCode {
    fn parse(text: &str) -> Result<Self, String> {
        let tokens = lex(text)?;
        let mut parser = Parser::new(tokens);
        let stmt = parser.parse_stmt()?;
        parser.expect_eof()?;
        Ok(StrArrCode(stmt))
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
    If,
    Then,
    End,
    While,
    IsEmpty,
    Head,
    Tail,
    LParen,
    RParen,
    LBracket,
    RBracket,
    LBrace,
    RBrace,
    Comma,
    Semi,
    Assign,
    Concat,
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
            Some(Token::If) => {
                self.next();
                let cond = self.parse_bexp()?;
                self.expect(Token::Then)?;
                let body = self.parse_stmt()?;
                self.expect(Token::End)?;
                Ok(Stmt::If {
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
                let expr = self.parse_aexp()?;
                Ok(Stmt::Assign { var, expr })
            }
            _ => Err("Invalid statement".to_string()),
        }
    }

    fn parse_bexp(&mut self) -> Result<BExp, String> {
        if self.eat(&Token::IsEmpty) {
            return Ok(BExp::IsEmpty(self.parse_aexp()?));
        }

        let lhs = self.parse_aexp()?;
        self.expect(Token::EqEq)?;
        let rhs = self.parse_aexp()?;
        Ok(BExp::Eq(lhs, rhs))
    }

    fn parse_aexp(&mut self) -> Result<AExp, String> {
        let mut left = self.parse_aexp_prefix()?;
        while self.eat(&Token::Concat) {
            let right = self.parse_aexp_prefix()?;
            left = AExp::Concat(Box::new(left), Box::new(right));
        }
        Ok(left)
    }

    fn parse_aexp_prefix(&mut self) -> Result<AExp, String> {
        if self.eat(&Token::Head) {
            return Ok(AExp::Head(Box::new(self.parse_aexp_prefix()?)));
        }
        if self.eat(&Token::Tail) {
            return Ok(AExp::Tail(Box::new(self.parse_aexp_prefix()?)));
        }
        self.parse_aexp_atom()
    }

    fn parse_aexp_atom(&mut self) -> Result<AExp, String> {
        match self.next() {
            Some(Token::Ident(name)) => {
                Ok(AExp::Var(Identifier::new(name).map_err(|e| e.to_string())?))
            }
            Some(Token::LParen) => {
                let expr = self.parse_aexp()?;
                self.expect(Token::RParen)?;
                Ok(expr)
            }
            Some(Token::LBracket) => {
                let mut chars = Vec::new();
                if !self.eat(&Token::RBracket) {
                    loop {
                        match self.next() {
                            Some(Token::Char(ch)) => chars.push(ch),
                            _ => return Err("Expected character literal".to_string()),
                        }
                        if self.eat(&Token::RBracket) {
                            break;
                        }
                        self.expect(Token::Comma)?;
                    }
                }
                Ok(AExp::Const(chars))
            }
            _ => Err("Invalid aexp".to_string()),
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
                "if" => Token::If,
                "then" => Token::Then,
                "end" => Token::End,
                "while" => Token::While,
                "is-empty" => Token::IsEmpty,
                "head" => Token::Head,
                "tail" => Token::Tail,
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
            '[' => {
                chars.next();
                tokens.push(Token::LBracket);
            }
            ']' => {
                chars.next();
                tokens.push(Token::RBracket);
            }
            '{' => {
                chars.next();
                tokens.push(Token::LBrace);
            }
            '}' => {
                chars.next();
                tokens.push(Token::RBrace);
            }
            ',' => {
                chars.next();
                tokens.push(Token::Comma);
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
            '+' => {
                chars.next();
                if chars.next() == Some('+') {
                    tokens.push(Token::Concat);
                } else {
                    return Err("Expected second '+'".to_string());
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
            '\'' => {
                tokens.push(Token::Char(parse_char_literal(&mut chars)?));
            }
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

fn char_list_to_text(chars: &[char]) -> String {
    let items: Vec<_> = chars.iter().copied().map(char_to_text).collect();
    format!("[{}]", items.join(", "))
}

fn parse_char_list(text: &str) -> Result<Vec<char>, String> {
    let tokens = lex(text)?;
    let mut parser = Parser::new(tokens);
    let expr = parser.parse_aexp()?;
    parser.expect_eof()?;
    match expr {
        AExp::Const(chars) => Ok(chars),
        _ => Err("Expected constant character list".to_string()),
    }
}

fn aexp_to_text(exp: &AExp) -> String {
    match exp {
        AExp::Const(chars) => char_list_to_text(chars),
        AExp::Var(var) => var.as_str().to_string(),
        AExp::Concat(lhs, rhs) => format!("({} ++ {})", aexp_to_text(lhs), aexp_to_text(rhs)),
        AExp::Head(exp) => format!("head {}", aexp_to_text(exp)),
        AExp::Tail(exp) => format!("tail {}", aexp_to_text(exp)),
    }
}

fn bexp_to_text(exp: &BExp) -> String {
    match exp {
        BExp::IsEmpty(exp) => format!("is-empty {}", aexp_to_text(exp)),
        BExp::Eq(lhs, rhs) => format!("{} == {}", aexp_to_text(lhs), aexp_to_text(rhs)),
    }
}

fn stmt_to_text(stmt: &Stmt) -> String {
    match stmt {
        Stmt::Nop => "Nop".to_string(),
        Stmt::Assign { var, expr } => format!("{} := {}", var.as_str(), aexp_to_text(expr)),
        Stmt::Seq(lhs, rhs) => format!("{} ; {}", stmt_to_text(lhs), stmt_to_text(rhs)),
        Stmt::If { cond, body } => format!("if {} then {} end", bexp_to_text(cond), stmt_to_text(body)),
        Stmt::While { cond, body } => format!("while {} {{ {} }}", bexp_to_text(cond), stmt_to_text(body)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn code_roundtrip() {
        let text = "x := ['a', 'b'] ; while x == ['a', 'b'] { x := tail x }";
        let parsed = StrArrCode::parse(text).unwrap();
        let printed = parsed.print();
        let reparsed = StrArrCode::parse(&printed).unwrap();
        assert_eq!(parsed, reparsed);
    }

    #[test]
    fn machine_runs() {
        let code = StrArrCode::parse("x := ['a', 'b'] ; while x == ['a', 'b'] { x := tail x }")
            .unwrap();
        let mut machine = StrArrMachine::make(code, Environment::default()).unwrap();
        loop {
            match machine.step(()).unwrap() {
                StepResult::Continue { next, .. } => machine = next,
                StepResult::Halt { output } => {
                    assert_eq!(output.get(&Identifier::new("x").unwrap()), vec!['b']);
                    return;
                }
            }
        }
    }
}
