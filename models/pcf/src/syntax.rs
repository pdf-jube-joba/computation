use std::collections::HashSet;

use serde::{Deserialize, Serialize};
use utils::{TextCodec, identifier::Identifier};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExprCode(pub Expr);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Expr {
    Nat(usize),
    Bool(bool),
    Unit,
    Print(usize),
    Var(Identifier),
    BinOp(Box<Expr>, BinOp, Box<Expr>),
    UnOp(UnOp, Box<Expr>),
    Fun {
        param: Identifier,
        body: Box<Expr>,
    },
    Rec {
        name: Identifier,
        param: Identifier,
        body: Box<Expr>,
    },
    App(Box<Expr>, Box<Expr>),
    If {
        cond: Box<Expr>,
        then_branch: Box<Expr>,
        else_branch: Box<Expr>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BinOp {
    Add,
    Sub,
    And,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum UnOp {
    Inc,
    Dec,
    Not,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct PrintEffect(pub Option<usize>);

impl TextCodec for PrintEffect {
    fn parse(text: &str) -> Result<Self, String> {
        let trimmed = text.trim();
        if trimmed.is_empty() {
            Ok(Self(None))
        } else {
            let value = trimmed.parse::<usize>().map_err(|e| e.to_string())?;
            Ok(Self(Some(value)))
        }
    }

    fn write_fmt(&self, f: &mut impl std::fmt::Write) -> std::fmt::Result {
        if let Some(value) = self.0 {
            write!(f, "{value}")
        } else {
            Ok(())
        }
    }
}

impl Expr {
    pub fn apply_inputs(self, inputs: &[usize]) -> Self {
        let mut expr = self;
        for input in inputs {
            expr = Expr::App(Box::new(expr), Box::new(Expr::Nat(*input)));
        }
        expr
    }

    pub fn is_value(&self) -> bool {
        matches!(
            self,
            Expr::Nat(_) | Expr::Bool(_) | Expr::Unit | Expr::Fun { .. } | Expr::Rec { .. }
        )
    }

    pub fn expect_nat(&self) -> Result<usize, String> {
        match self {
            Expr::Nat(value) => Ok(*value),
            _ => Err("final value is not a natural number".to_string()),
        }
    }
}

impl TextCodec for ExprCode {
    fn parse(text: &str) -> Result<Self, String> {
        let tokens = lex(text)?;
        let mut parser = Parser::new(tokens);
        let expr = parser.parse_expr()?;
        parser.expect_eof()?;
        Ok(Self(expr))
    }

    fn write_fmt(&self, f: &mut impl std::fmt::Write) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::fmt::Display for Expr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Expr::Nat(value) => write!(f, "{value}"),
            Expr::Bool(true) => write!(f, "#true"),
            Expr::Bool(false) => write!(f, "#false"),
            Expr::Unit => write!(f, "#unit"),
            Expr::Print(value) => write!(f, "print {value}"),
            Expr::Var(name) => write!(f, "{name}"),
            Expr::BinOp(lhs, op, rhs) => write!(f, "({lhs} {op} {rhs})"),
            Expr::UnOp(op, expr) => write!(f, "({op} {expr})"),
            Expr::Fun { param, body } => write!(f, "(fun {param} => {body})"),
            Expr::Rec { name, param, body } => write!(f, "(rec {name} {param} => {body})"),
            Expr::App(fun, arg) => write!(f, "{fun}({arg})"),
            Expr::If {
                cond,
                then_branch,
                else_branch,
            } => write!(f, "(if {cond} then {then_branch} else {else_branch} fi)"),
        }
    }
}

impl std::fmt::Display for BinOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BinOp::Add => write!(f, "+"),
            BinOp::Sub => write!(f, "-"),
            BinOp::And => write!(f, "&&"),
        }
    }
}

impl std::fmt::Display for UnOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UnOp::Inc => write!(f, "inc"),
            UnOp::Dec => write!(f, "dec"),
            UnOp::Not => write!(f, "not"),
        }
    }
}

pub fn substitute(expr: &Expr, target: &Identifier, replacement: &Expr) -> Expr {
    match expr {
        Expr::Nat(_) | Expr::Bool(_) | Expr::Unit | Expr::Print(_) => expr.clone(),
        Expr::Var(name) => {
            if name == target {
                replacement.clone()
            } else {
                expr.clone()
            }
        }
        Expr::BinOp(lhs, op, rhs) => Expr::BinOp(
            Box::new(substitute(lhs, target, replacement)),
            *op,
            Box::new(substitute(rhs, target, replacement)),
        ),
        Expr::UnOp(op, inner) => Expr::UnOp(*op, Box::new(substitute(inner, target, replacement))),
        Expr::Fun { param, body } => {
            if param == target {
                expr.clone()
            } else {
                let mut param = param.clone();
                let mut body = (**body).clone();
                let replacement_fv = free_vars(replacement);
                if replacement_fv.contains(param.as_str()) {
                    let fresh =
                        fresh_identifier(&all_names_expr(expr), &replacement_fv, param.as_str());
                    body = rename_bound_occurrences(&body, &param, &fresh);
                    param = fresh;
                }
                Expr::Fun {
                    param,
                    body: Box::new(substitute(&body, target, replacement)),
                }
            }
        }
        Expr::Rec { name, param, body } => {
            if name == target || param == target {
                expr.clone()
            } else {
                let replacement_fv = free_vars(replacement);
                let mut name = name.clone();
                let mut param = param.clone();
                let mut body = (**body).clone();
                let used_names = all_names_expr(expr);

                if replacement_fv.contains(name.as_str()) {
                    let fresh = fresh_identifier(&used_names, &replacement_fv, name.as_str());
                    body = rename_bound_occurrences(&body, &name, &fresh);
                    name = fresh;
                }

                if replacement_fv.contains(param.as_str()) {
                    let fresh = fresh_identifier(&used_names, &replacement_fv, param.as_str());
                    body = rename_bound_occurrences(&body, &param, &fresh);
                    param = fresh;
                }

                Expr::Rec {
                    name,
                    param,
                    body: Box::new(substitute(&body, target, replacement)),
                }
            }
        }
        Expr::App(fun, arg) => Expr::App(
            Box::new(substitute(fun, target, replacement)),
            Box::new(substitute(arg, target, replacement)),
        ),
        Expr::If {
            cond,
            then_branch,
            else_branch,
        } => Expr::If {
            cond: Box::new(substitute(cond, target, replacement)),
            then_branch: Box::new(substitute(then_branch, target, replacement)),
            else_branch: Box::new(substitute(else_branch, target, replacement)),
        },
    }
}

fn rename_bound_occurrences(expr: &Expr, from: &Identifier, to: &Identifier) -> Expr {
    match expr {
        Expr::Nat(_) | Expr::Bool(_) | Expr::Unit | Expr::Print(_) => expr.clone(),
        Expr::Var(name) => {
            if name == from {
                Expr::Var(to.clone())
            } else {
                expr.clone()
            }
        }
        Expr::BinOp(lhs, op, rhs) => Expr::BinOp(
            Box::new(rename_bound_occurrences(lhs, from, to)),
            *op,
            Box::new(rename_bound_occurrences(rhs, from, to)),
        ),
        Expr::UnOp(op, inner) => {
            Expr::UnOp(*op, Box::new(rename_bound_occurrences(inner, from, to)))
        }
        Expr::Fun { param, body } => {
            if param == from {
                expr.clone()
            } else {
                Expr::Fun {
                    param: param.clone(),
                    body: Box::new(rename_bound_occurrences(body, from, to)),
                }
            }
        }
        Expr::Rec { name, param, body } => {
            if name == from || param == from {
                expr.clone()
            } else {
                Expr::Rec {
                    name: name.clone(),
                    param: param.clone(),
                    body: Box::new(rename_bound_occurrences(body, from, to)),
                }
            }
        }
        Expr::App(fun, arg) => Expr::App(
            Box::new(rename_bound_occurrences(fun, from, to)),
            Box::new(rename_bound_occurrences(arg, from, to)),
        ),
        Expr::If {
            cond,
            then_branch,
            else_branch,
        } => Expr::If {
            cond: Box::new(rename_bound_occurrences(cond, from, to)),
            then_branch: Box::new(rename_bound_occurrences(then_branch, from, to)),
            else_branch: Box::new(rename_bound_occurrences(else_branch, from, to)),
        },
    }
}

fn free_vars(expr: &Expr) -> HashSet<String> {
    match expr {
        Expr::Nat(_) | Expr::Bool(_) | Expr::Unit | Expr::Print(_) => HashSet::new(),
        Expr::Var(name) => HashSet::from([name.as_str().to_string()]),
        Expr::BinOp(lhs, _, rhs) | Expr::App(lhs, rhs) => {
            let mut set = free_vars(lhs);
            set.extend(free_vars(rhs));
            set
        }
        Expr::UnOp(_, expr) => free_vars(expr),
        Expr::Fun { param, body } => {
            let mut set = free_vars(body);
            set.remove(param.as_str());
            set
        }
        Expr::Rec { name, param, body } => {
            let mut set = free_vars(body);
            set.remove(name.as_str());
            set.remove(param.as_str());
            set
        }
        Expr::If {
            cond,
            then_branch,
            else_branch,
        } => {
            let mut set = free_vars(cond);
            set.extend(free_vars(then_branch));
            set.extend(free_vars(else_branch));
            set
        }
    }
}

fn all_names_expr(expr: &Expr) -> HashSet<String> {
    match expr {
        Expr::Nat(_) | Expr::Bool(_) | Expr::Unit | Expr::Print(_) => HashSet::new(),
        Expr::Var(name) => HashSet::from([name.as_str().to_string()]),
        Expr::BinOp(lhs, _, rhs) | Expr::App(lhs, rhs) => {
            let mut set = all_names_expr(lhs);
            set.extend(all_names_expr(rhs));
            set
        }
        Expr::UnOp(_, expr) => all_names_expr(expr),
        Expr::Fun { param, body } => {
            let mut set = all_names_expr(body);
            set.insert(param.as_str().to_string());
            set
        }
        Expr::Rec { name, param, body } => {
            let mut set = all_names_expr(body);
            set.insert(name.as_str().to_string());
            set.insert(param.as_str().to_string());
            set
        }
        Expr::If {
            cond,
            then_branch,
            else_branch,
        } => {
            let mut set = all_names_expr(cond);
            set.extend(all_names_expr(then_branch));
            set.extend(all_names_expr(else_branch));
            set
        }
    }
}

fn fresh_identifier(
    used_in_expr: &HashSet<String>,
    used_in_replacement: &HashSet<String>,
    base: &str,
) -> Identifier {
    let mut index = 0usize;
    loop {
        let candidate = format!("{base}_{index}");
        if !used_in_expr.contains(&candidate) && !used_in_replacement.contains(&candidate) {
            return Identifier::new(candidate).expect("fresh identifier must be valid");
        }
        index += 1;
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Token {
    Number(usize),
    Ident(String),
    True,
    False,
    Unit,
    Print,
    Fun,
    Rec,
    If,
    Then,
    Else,
    Fi,
    Inc,
    Dec,
    Not,
    Plus,
    Minus,
    AndAnd,
    Arrow,
    LParen,
    RParen,
}

fn lex(text: &str) -> Result<Vec<Token>, String> {
    let chars: Vec<char> = text.chars().collect();
    let mut tokens = Vec::new();
    let mut pos = 0usize;

    while pos < chars.len() {
        let ch = chars[pos];
        if ch.is_whitespace() {
            pos += 1;
            continue;
        }

        if ch.is_ascii_digit() {
            let start = pos;
            pos += 1;
            while pos < chars.len() && chars[pos].is_ascii_digit() {
                pos += 1;
            }
            let value = text[start..pos]
                .parse::<usize>()
                .map_err(|e| e.to_string())?;
            tokens.push(Token::Number(value));
            continue;
        }

        if ch == '#' {
            let start = pos;
            pos += 1;
            while pos < chars.len() && (chars[pos].is_ascii_alphanumeric() || chars[pos] == '_') {
                pos += 1;
            }
            match &text[start..pos] {
                "#true" => tokens.push(Token::True),
                "#false" => tokens.push(Token::False),
                "#unit" => tokens.push(Token::Unit),
                other => return Err(format!("unknown token: {other}")),
            }
            continue;
        }

        if ch.is_ascii_alphabetic() || ch == '_' {
            let start = pos;
            pos += 1;
            while pos < chars.len()
                && (chars[pos].is_ascii_alphanumeric() || chars[pos] == '_' || chars[pos] == '-')
            {
                pos += 1;
            }
            let word = &text[start..pos];
            let token = match word {
                "print" => Token::Print,
                "fun" => Token::Fun,
                "rec" => Token::Rec,
                "if" => Token::If,
                "then" => Token::Then,
                "else" => Token::Else,
                "fi" => Token::Fi,
                "inc" => Token::Inc,
                "dec" => Token::Dec,
                "not" => Token::Not,
                _ => Token::Ident(word.to_string()),
            };
            tokens.push(token);
            continue;
        }

        match ch {
            '+' => {
                tokens.push(Token::Plus);
                pos += 1;
            }
            '-' => {
                tokens.push(Token::Minus);
                pos += 1;
            }
            '&' => {
                if chars.get(pos + 1) == Some(&'&') {
                    tokens.push(Token::AndAnd);
                    pos += 2;
                } else {
                    return Err("expected '&&'".to_string());
                }
            }
            '=' => {
                if chars.get(pos + 1) == Some(&'>') {
                    tokens.push(Token::Arrow);
                    pos += 2;
                } else {
                    return Err("expected '=>'".to_string());
                }
            }
            '(' => {
                tokens.push(Token::LParen);
                pos += 1;
            }
            ')' => {
                tokens.push(Token::RParen);
                pos += 1;
            }
            _ => return Err(format!("unexpected character: {ch}")),
        }
    }

    Ok(tokens)
}

struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, pos: 0 }
    }

    fn expect_eof(&self) -> Result<(), String> {
        if self.pos == self.tokens.len() {
            Ok(())
        } else {
            Err("unexpected trailing tokens".to_string())
        }
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

    fn parse_expr(&mut self) -> Result<Expr, String> {
        if matches!(self.peek(), Some(Token::If)) {
            self.parse_if()
        } else {
            self.parse_binop_expr()
        }
    }

    fn parse_if(&mut self) -> Result<Expr, String> {
        self.expect_token(Token::If)?;
        let cond = self.parse_expr()?;
        self.expect_token(Token::Then)?;
        let then_branch = self.parse_expr()?;
        self.expect_token(Token::Else)?;
        let else_branch = self.parse_expr()?;
        self.expect_token(Token::Fi)?;
        Ok(Expr::If {
            cond: Box::new(cond),
            then_branch: Box::new(then_branch),
            else_branch: Box::new(else_branch),
        })
    }

    fn parse_binop_expr(&mut self) -> Result<Expr, String> {
        let mut expr = self.parse_application()?;
        loop {
            let op = match self.peek() {
                Some(Token::Plus) => BinOp::Add,
                Some(Token::Minus) => BinOp::Sub,
                Some(Token::AndAnd) => BinOp::And,
                _ => break,
            };
            self.next();
            let rhs = self.parse_application()?;
            expr = Expr::BinOp(Box::new(expr), op, Box::new(rhs));
        }
        Ok(expr)
    }

    fn parse_application(&mut self) -> Result<Expr, String> {
        let mut expr = self.parse_prefix()?;
        loop {
            if !matches!(self.peek(), Some(Token::LParen)) {
                break;
            }
            self.next();
            let arg = self.parse_expr()?;
            self.expect_token(Token::RParen)?;
            expr = Expr::App(Box::new(expr), Box::new(arg));
        }
        Ok(expr)
    }

    fn parse_prefix(&mut self) -> Result<Expr, String> {
        match self.peek() {
            Some(Token::Inc) => {
                self.next();
                Ok(Expr::UnOp(UnOp::Inc, Box::new(self.parse_prefix()?)))
            }
            Some(Token::Dec) => {
                self.next();
                Ok(Expr::UnOp(UnOp::Dec, Box::new(self.parse_prefix()?)))
            }
            Some(Token::Not) => {
                self.next();
                Ok(Expr::UnOp(UnOp::Not, Box::new(self.parse_prefix()?)))
            }
            _ => self.parse_atom(),
        }
    }

    fn parse_atom(&mut self) -> Result<Expr, String> {
        match self.next() {
            Some(Token::Number(value)) => Ok(Expr::Nat(value)),
            Some(Token::True) => Ok(Expr::Bool(true)),
            Some(Token::False) => Ok(Expr::Bool(false)),
            Some(Token::Unit) => Ok(Expr::Unit),
            Some(Token::Print) => match self.next() {
                Some(Token::Number(value)) => Ok(Expr::Print(value)),
                _ => Err("print expects a natural number literal".to_string()),
            },
            Some(Token::Ident(name)) => {
                let ident = Identifier::new(name).map_err(|e| e.to_string())?;
                Ok(Expr::Var(ident))
            }
            Some(Token::Fun) => {
                let param = self.parse_identifier()?;
                self.expect_token(Token::Arrow)?;
                let body = self.parse_expr()?;
                Ok(Expr::Fun {
                    param,
                    body: Box::new(body),
                })
            }
            Some(Token::Rec) => {
                let name = self.parse_identifier()?;
                let param = self.parse_identifier()?;
                self.expect_token(Token::Arrow)?;
                let body = self.parse_expr()?;
                Ok(Expr::Rec {
                    name,
                    param,
                    body: Box::new(body),
                })
            }
            Some(Token::LParen) => {
                let expr = self.parse_expr()?;
                self.expect_token(Token::RParen)?;
                Ok(expr)
            }
            Some(Token::If) => {
                self.pos -= 1;
                self.parse_if()
            }
            other => Err(format!("unexpected token: {other:?}")),
        }
    }

    fn parse_identifier(&mut self) -> Result<Identifier, String> {
        match self.next() {
            Some(Token::Ident(name)) => Identifier::new(name).map_err(|e| e.to_string()),
            other => Err(format!("identifier expected, found {other:?}")),
        }
    }

    fn expect_token(&mut self, expected: Token) -> Result<(), String> {
        let token = self.next();
        if token == Some(expected.clone()) {
            Ok(())
        } else {
            Err(format!("expected {:?}, found {:?}", expected, token))
        }
    }
}
