use std::collections::BTreeMap;

use utils::number::Number;
use utils::{TextCodec, identifier::Identifier};

use crate::coroutine::{
    AAtom, ABinOp, AExp, BExp, CoroutineCode, FnDecl, GlobalEnv, Program, RelOp, Stmt,
};

impl TextCodec for GlobalEnv {
    fn parse(text: &str) -> Result<Self, String> {
        let mut vars = BTreeMap::new();
        let mut task_ids = BTreeMap::new();
        for raw in text.lines() {
            let line = raw.trim();
            if line.is_empty() {
                continue;
            }
            let (name, value) = line
                .split_once('=')
                .ok_or_else(|| format!("invalid env line: {line}"))?;
            let name = name.trim();
            let value = value.trim();
            if let Some(name) = name.strip_prefix('$') {
                validate_ident(name)?;
                let task_id = if value.eq_ignore_ascii_case("none") {
                    None
                } else {
                    Some(value.parse::<usize>().map_err(|e| e.to_string())?)
                };
                task_ids.insert(name.to_string(), task_id);
            } else {
                validate_ident(name)?;
                vars.insert(name.to_string(), Number::parse(value)?);
            }
        }
        Ok(Self { vars, task_ids })
    }

    fn write_fmt(&self, f: &mut impl std::fmt::Write) -> std::fmt::Result {
        for (name, value) in &self.vars {
            writeln!(f, "{name} = {}", value.to_decimal_string())?;
        }
        for (name, value) in &self.task_ids {
            match value {
                Some(task_id) => writeln!(f, "${name} = {task_id}")?,
                None => writeln!(f, "${name} = none")?,
            }
        }
        Ok(())
    }
}

impl TextCodec for CoroutineCode {
    fn parse(text: &str) -> Result<Self, String> {
        Ok(Self(parse_program(text)?))
    }

    fn write_fmt(&self, f: &mut impl std::fmt::Write) -> std::fmt::Result {
        for (index, function) in self.0.functions.iter().enumerate() {
            if index > 0 {
                writeln!(f)?;
            }
            writeln!(f, "fn {} {{", function.name)?;
            if !function.body.is_empty() {
                writeln!(
                    f,
                    "  {}",
                    function
                        .body
                        .iter()
                        .map(stmt_to_text)
                        .collect::<Vec<_>>()
                        .join("; ")
                )?;
            }
            writeln!(f, "}}")?;
        }
        Ok(())
    }
}

fn validate_ident(name: &str) -> Result<(), String> {
    Identifier::new(name)
        .map(|_| ())
        .map_err(|e| e.to_string())
}

pub fn stmt_to_text(stmt: &Stmt) -> String {
    match stmt {
        Stmt::Assign { var, expr } => format!("{var} <- {}", aexp_to_text(expr)),
        Stmt::IfGoto { cond, offset } => format!("if {} goto {}", bexp_to_text(cond), fmt_offset(*offset)),
        Stmt::Goto { offset } => format!("goto {}", fmt_offset(*offset)),
        Stmt::Call { name } => format!("call {name}"),
        Stmt::Run { name, id_var } => format!("run {name} -> ${id_var}"),
        Stmt::Yield => "yield".to_string(),
    }
}

fn fmt_offset(offset: isize) -> String {
    if offset >= 0 {
        format!("+{offset}")
    } else {
        offset.to_string()
    }
}

fn aexp_to_text(exp: &AExp) -> String {
    match exp {
        AExp::Atom(AAtom::Var(name)) => name.clone(),
        AExp::Atom(AAtom::Imm(value)) => value.to_decimal_string(),
        AExp::Bin { lhs, op, rhs } => format!(
            "({} {} {})",
            aexp_to_text(lhs),
            match op {
                ABinOp::Add => "+",
                ABinOp::Sub => "-",
                ABinOp::Mul => "*",
            },
            aexp_to_text(rhs)
        ),
    }
}

fn bexp_to_text(exp: &BExp) -> String {
    match exp {
        BExp::Not(inner) => format!("!{}", bexp_to_text(inner)),
        BExp::Rel { lhs, op, rhs } => format!(
            "{} {} {}",
            aexp_to_text(lhs),
            match op {
                RelOp::Eq => "==",
                RelOp::Ne => "!=",
                RelOp::Lt => "<",
            },
            aexp_to_text(rhs)
        ),
        BExp::And(lhs, rhs) => format!("({} && {})", bexp_to_text(lhs), bexp_to_text(rhs)),
        BExp::Or(lhs, rhs) => format!("({} || {})", bexp_to_text(lhs), bexp_to_text(rhs)),
        BExp::Done(id_var) => format!("done ${id_var}"),
    }
}

pub fn parse_program(text: &str) -> Result<Program, String> {
    let mut parser = ProgramParser::new(text);
    parser.parse_program()
}

struct ProgramParser<'a> {
    lines: Vec<&'a str>,
    pos: usize,
}

impl<'a> ProgramParser<'a> {
    fn new(text: &'a str) -> Self {
        Self {
            lines: text.lines().collect(),
            pos: 0,
        }
    }

    fn parse_program(&mut self) -> Result<Program, String> {
        let mut functions = Vec::new();
        while let Some(line) = self.next_nonempty_line() {
            let header = strip_comment(line).trim();
            if header.is_empty() {
                continue;
            }
            functions.push(self.parse_function(header)?);
        }
        if functions.is_empty() {
            return Err("at least one function is required".to_string());
        }
        Ok(Program { functions })
    }

    fn parse_function(&mut self, header: &str) -> Result<FnDecl, String> {
        let header = header
            .strip_prefix("fn ")
            .ok_or_else(|| format!("expected function declaration, got: {header}"))?;
        let (name, rest) = header
            .split_once('{')
            .ok_or_else(|| format!("expected '{{' in function declaration: {header}"))?;
        let name = name.trim();
        validate_ident(name)?;

        let mut body_parts = vec![rest.trim()];
        let mut found_end = header.contains('}');
        if found_end {
            let last = body_parts.pop().unwrap();
            let (before, after) = last
                .split_once('}')
                .ok_or_else(|| "malformed function body".to_string())?;
            if !after.trim().is_empty() {
                return Err(format!("unexpected tokens after '}}': {after}"));
            }
            body_parts.push(before.trim());
        } else {
            while let Some(line) = self.next_nonempty_line() {
                let line = strip_comment(line).trim();
                if let Some((before, after)) = line.split_once('}') {
                    if !after.trim().is_empty() {
                        return Err(format!("unexpected tokens after '}}': {after}"));
                    }
                    body_parts.push(before.trim());
                    found_end = true;
                    break;
                }
                body_parts.push(line);
            }
        }

        if !found_end {
            return Err(format!("function {name} is missing closing '}}'"));
        }

        let body_text = body_parts
            .into_iter()
            .filter(|part| !part.is_empty())
            .collect::<Vec<_>>()
            .join(" ");

        let body = split_statements(&body_text)?
            .into_iter()
            .map(parse_stmt)
            .collect::<Result<Vec<_>, _>>()?;

        Ok(FnDecl {
            name: name.to_string(),
            body,
        })
    }

    fn next_nonempty_line(&mut self) -> Option<&'a str> {
        while self.pos < self.lines.len() {
            let line = self.lines[self.pos];
            self.pos += 1;
            if !strip_comment(line).trim().is_empty() {
                return Some(line);
            }
        }
        None
    }
}

fn strip_comment(line: &str) -> &str {
    line.split_once("//").map(|(head, _)| head).unwrap_or(line)
}

fn split_statements(text: &str) -> Result<Vec<&str>, String> {
    let mut stmts = Vec::new();
    let mut start = 0;
    let mut depth = 0usize;
    for (idx, ch) in text.char_indices() {
        match ch {
            '(' => depth += 1,
            ')' => {
                if depth == 0 {
                    return Err("unmatched ')'".to_string());
                }
                depth -= 1;
            }
            ';' if depth == 0 => {
                let stmt = text[start..idx].trim();
                if !stmt.is_empty() {
                    stmts.push(stmt);
                }
                start = idx + 1;
            }
            _ => {}
        }
    }
    if depth != 0 {
        return Err("unmatched '('".to_string());
    }
    let tail = text[start..].trim();
    if !tail.is_empty() {
        stmts.push(tail);
    }
    Ok(stmts)
}

fn parse_stmt(text: &str) -> Result<Stmt, String> {
    if let Some(rest) = text.strip_prefix("if ") {
        let (cond, offset) = rest
            .rsplit_once(" goto ")
            .ok_or_else(|| format!("invalid if-goto statement: {text}"))?;
        return Ok(Stmt::IfGoto {
            cond: ExprParser::new(cond)?.parse_bexp()?,
            offset: parse_offset(offset)?,
        });
    }
    if let Some(offset) = text.strip_prefix("goto ") {
        return Ok(Stmt::Goto {
            offset: parse_offset(offset)?,
        });
    }
    if let Some(name) = text.strip_prefix("call ") {
        let name = name.trim();
        validate_ident(name)?;
        return Ok(Stmt::Call {
            name: name.to_string(),
        });
    }
    if let Some(rest) = text.strip_prefix("run ") {
        let (name, id_var) = rest
            .split_once("->")
            .ok_or_else(|| format!("invalid run statement: {text}"))?;
        let name = name.trim();
        let id_var = id_var.trim();
        validate_ident(name)?;
        let id_var = id_var
            .strip_prefix('$')
            .ok_or_else(|| format!("expected task id after '->' in: {text}"))?;
        validate_ident(id_var)?;
        return Ok(Stmt::Run {
            name: name.to_string(),
            id_var: id_var.to_string(),
        });
    }
    if text == "yield" {
        return Ok(Stmt::Yield);
    }
    let (var, expr) = text
        .split_once("<-")
        .ok_or_else(|| format!("invalid statement: {text}"))?;
    let var = var.trim();
    validate_ident(var)?;
    Ok(Stmt::Assign {
        var: var.to_string(),
        expr: ExprParser::new(expr)?.parse_aexp()?,
    })
}

fn parse_offset(text: &str) -> Result<isize, String> {
    let text = text.trim();
    if text.is_empty() {
        return Err("empty offset".to_string());
    }
    text.parse::<isize>()
        .map_err(|e| format!("invalid offset '{text}': {e}"))
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Token {
    Ident(String),
    IdVar(String),
    Number(Number),
    Plus,
    Minus,
    Star,
    LParen,
    RParen,
    EqEq,
    Ne,
    Lt,
    Bang,
    AndAnd,
    OrOr,
    Done,
}

struct ExprParser {
    tokens: Vec<Token>,
    pos: usize,
}

impl ExprParser {
    fn new(text: &str) -> Result<Self, String> {
        Ok(Self {
            tokens: lex_expr(text)?,
            pos: 0,
        })
    }

    fn parse_aexp(mut self) -> Result<AExp, String> {
        let expr = self.parse_aexp_bp(0)?;
        self.expect_eof()?;
        Ok(expr)
    }

    fn parse_bexp(mut self) -> Result<BExp, String> {
        let expr = self.parse_bexp_bp(0)?;
        self.expect_eof()?;
        Ok(expr)
    }

    fn parse_aexp_bp(&mut self, min_bp: u8) -> Result<AExp, String> {
        let mut lhs = match self.next() {
            Some(Token::Ident(name)) => AExp::Atom(AAtom::Var(name)),
            Some(Token::Number(value)) => AExp::Atom(AAtom::Imm(value)),
            Some(Token::LParen) => {
                let expr = self.parse_aexp_bp(0)?;
                self.expect(Token::RParen)?;
                expr
            }
            other => return Err(format!("unexpected token in arithmetic expr: {other:?}")),
        };

        loop {
            let (op, lbp, rbp) = match self.peek() {
                Some(Token::Plus) => (ABinOp::Add, 1, 2),
                Some(Token::Minus) => (ABinOp::Sub, 1, 2),
                Some(Token::Star) => (ABinOp::Mul, 3, 4),
                _ => break,
            };
            if lbp < min_bp {
                break;
            }
            self.pos += 1;
            let rhs = self.parse_aexp_bp(rbp)?;
            lhs = AExp::Bin {
                lhs: Box::new(lhs),
                op,
                rhs: Box::new(rhs),
            };
        }
        Ok(lhs)
    }

    fn parse_bexp_bp(&mut self, min_bp: u8) -> Result<BExp, String> {
        let mut lhs = match self.next() {
            Some(Token::Bang) => {
                let rhs = self.parse_bexp_bp(5)?;
                BExp::Not(Box::new(rhs))
            }
            Some(Token::Done) => match self.next() {
                Some(Token::IdVar(name)) => BExp::Done(name),
                other => return Err(format!("expected task id after done, got: {other:?}")),
            },
            Some(Token::LParen) => {
                let expr = self.parse_bexp_bp(0)?;
                self.expect(Token::RParen)?;
                expr
            }
            Some(Token::Ident(name)) => self.parse_rel_tail(AExp::Atom(AAtom::Var(name)))?,
            Some(Token::Number(value)) => self.parse_rel_tail(AExp::Atom(AAtom::Imm(value)))?,
            other => return Err(format!("unexpected token in boolean expr: {other:?}")),
        };

        loop {
            let (lbp, rbp, ctor) = match self.peek() {
                Some(Token::AndAnd) => (2, 3, 0),
                Some(Token::OrOr) => (0, 1, 1),
                _ => break,
            };
            if lbp < min_bp {
                break;
            }
            self.pos += 1;
            let rhs = self.parse_bexp_bp(rbp)?;
            lhs = if ctor == 0 {
                BExp::And(Box::new(lhs), Box::new(rhs))
            } else {
                BExp::Or(Box::new(lhs), Box::new(rhs))
            };
        }
        Ok(lhs)
    }

    fn parse_rel_tail(&mut self, lhs: AExp) -> Result<BExp, String> {
        let op = match self.next() {
            Some(Token::EqEq) => RelOp::Eq,
            Some(Token::Ne) => RelOp::Ne,
            Some(Token::Lt) => RelOp::Lt,
            other => return Err(format!("expected relational operator, got: {other:?}")),
        };
        let rhs = self.parse_aexp_bp(0)?;
        Ok(BExp::Rel { lhs, op, rhs })
    }

    fn expect(&mut self, token: Token) -> Result<(), String> {
        let got = self.next();
        if got == Some(token.clone()) {
            Ok(())
        } else {
            Err(format!("expected {:?}, got {:?}", token, got))
        }
    }

    fn expect_eof(&self) -> Result<(), String> {
        if self.pos == self.tokens.len() {
            Ok(())
        } else {
            Err(format!("unexpected trailing tokens: {:?}", &self.tokens[self.pos..]))
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
}

fn lex_expr(text: &str) -> Result<Vec<Token>, String> {
    let text = text.trim();
    let chars: Vec<char> = text.chars().collect();
    let mut tokens = Vec::new();
    let mut i = 0;
    while i < chars.len() {
        let ch = chars[i];
        if ch.is_whitespace() {
            i += 1;
            continue;
        }
        if ch.is_ascii_digit() {
            let start = i;
            i += 1;
            while i < chars.len() && chars[i].is_ascii_digit() {
                i += 1;
            }
            let number = text[start..i].parse::<usize>().map_err(|e| e.to_string())?;
            tokens.push(Token::Number(Number::from(number)));
            continue;
        }
        if ch.is_ascii_alphabetic() || ch == '_' {
            let start = i;
            i += 1;
            while i < chars.len() && (chars[i].is_ascii_alphanumeric() || chars[i] == '_') {
                i += 1;
            }
            let word = &text[start..i];
            if word == "done" {
                tokens.push(Token::Done);
            } else {
                validate_ident(word)?;
                tokens.push(Token::Ident(word.to_string()));
            }
            continue;
        }
        if ch == '$' {
            let start = i + 1;
            i += 1;
            while i < chars.len() && (chars[i].is_ascii_alphanumeric() || chars[i] == '_') {
                i += 1;
            }
            let name = &text[start..i];
            validate_ident(name)?;
            tokens.push(Token::IdVar(name.to_string()));
            continue;
        }
        match ch {
            '+' => tokens.push(Token::Plus),
            '-' => tokens.push(Token::Minus),
            '*' => tokens.push(Token::Star),
            '(' => tokens.push(Token::LParen),
            ')' => tokens.push(Token::RParen),
            '!' => {
                if i + 1 < chars.len() && chars[i + 1] == '=' {
                    tokens.push(Token::Ne);
                    i += 2;
                    continue;
                }
                tokens.push(Token::Bang);
            }
            '=' => {
                if i + 1 < chars.len() && chars[i + 1] == '=' {
                    tokens.push(Token::EqEq);
                    i += 2;
                    continue;
                }
                return Err("expected '=='".to_string());
            }
            '<' => tokens.push(Token::Lt),
            '&' => {
                if i + 1 < chars.len() && chars[i + 1] == '&' {
                    tokens.push(Token::AndAnd);
                    i += 2;
                    continue;
                }
                return Err("expected '&&'".to_string());
            }
            '|' => {
                if i + 1 < chars.len() && chars[i + 1] == '|' {
                    tokens.push(Token::OrOr);
                    i += 2;
                    continue;
                }
                return Err("expected '||'".to_string());
            }
            _ => return Err(format!("unexpected character: {ch}")),
        }
        i += 1;
    }
    Ok(tokens)
}
