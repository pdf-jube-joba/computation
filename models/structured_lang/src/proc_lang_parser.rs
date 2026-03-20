use std::collections::BTreeMap;

use utils::TextCodec;
use utils::number::Number;

use super::{ABinOp, AExp, Atom, BExp, GlobalEnv, ProcCode, ProcDef, Program, RelOp, Stmt};

fn ensure_valid_ident(name: &str) -> Result<(), String> {
    utils::identifier::Identifier::new(name)
        .map(|_| ())
        .map_err(|e| e.to_string())
}

impl TextCodec for GlobalEnv {
    fn parse(text: &str) -> Result<Self, String> {
        let mut vars = BTreeMap::new();
        for raw in text.lines() {
            let line = raw.trim();
            if line.is_empty() {
                continue;
            }
            let (name, value) = line
                .split_once('=')
                .ok_or_else(|| format!("Invalid env line: {line}"))?;
            let name = name.trim();
            let value = value.trim();
            ensure_valid_ident(name)?;
            vars.insert(name.to_string(), Number::parse(value)?);
        }
        Ok(Self { vars })
    }

    fn write_fmt(&self, f: &mut impl std::fmt::Write) -> std::fmt::Result {
        for (name, value) in &self.vars {
            writeln!(f, "{} = {}", name, value.to_decimal_string())?;
        }
        Ok(())
    }
}

impl TextCodec for ProcCode {
    fn parse(text: &str) -> Result<Self, String> {
        Ok(Self(parse_program(text)?))
    }

    fn write_fmt(&self, f: &mut impl std::fmt::Write) -> std::fmt::Result {
        let program = &self.0;
        write!(f, "static")?;
        if !program.statics.is_empty() {
            write!(f, " {}", program.statics.join(", "))?;
        }
        writeln!(f, ";")?;

        for proc in &program.procs {
            writeln!(f, "{}({})[", proc.name, proc.params.join(", "))?;
            if proc.locals.is_empty() {
                writeln!(f, "  local")?;
            } else {
                writeln!(f, "  local {}", proc.locals.join(", "))?;
            }
            writeln!(f, "  {}", stmt_to_text(&proc.body))?;
            writeln!(f, "]")?;
        }
        Ok(())
    }
}

pub fn parse_program(text: &str) -> Result<Program, String> {
    let mut parser = Parser::new(text)?;
    let program = parser.parse_program()?;
    parser.expect(Token::Eof)?;
    Ok(program)
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Token {
    Ident(String),
    Number(String),
    Static,
    Local,
    Nop,
    If,
    While,
    Call,
    Return,
    LParen,
    RParen,
    LBrack,
    RBrack,
    Comma,
    Semi,
    Assign,
    Arrow,
    Plus,
    Minus,
    Lt,
    Eq,
    Gt,
    Eof,
}

struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    fn new(text: &str) -> Result<Self, String> {
        Ok(Self {
            tokens: lex(text)?,
            pos: 0,
        })
    }

    fn peek(&self) -> &Token {
        self.tokens.get(self.pos).unwrap_or(&Token::Eof)
    }

    fn next(&mut self) -> Token {
        let t = self.peek().clone();
        if !matches!(t, Token::Eof) {
            self.pos += 1;
        }
        t
    }

    fn expect(&mut self, expect: Token) -> Result<(), String> {
        let t = self.next();
        if t == expect {
            Ok(())
        } else {
            Err(format!("Unexpected token: {:?}, expected {:?}", t, expect))
        }
    }

    fn parse_program(&mut self) -> Result<Program, String> {
        self.expect(Token::Static)?;
        let statics = self.parse_var_list_until(&Token::Semi)?;
        self.expect(Token::Semi)?;

        let mut procs = Vec::new();
        while !matches!(self.peek(), Token::Eof) {
            procs.push(self.parse_proc()?);
        }

        if procs.is_empty() {
            return Err("At least one procedure is required".to_string());
        }

        Ok(Program { statics, procs })
    }

    fn parse_proc(&mut self) -> Result<ProcDef, String> {
        let name = self.parse_ident()?;
        self.expect(Token::LParen)?;
        let params = self.parse_var_list_until(&Token::RParen)?;
        self.expect(Token::RParen)?;
        self.expect(Token::LBrack)?;
        self.expect(Token::Local)?;
        let locals = self.parse_var_list_stmt_boundary()?;
        let body = self.parse_stmt()?;
        self.expect(Token::RBrack)?;
        Ok(ProcDef {
            name,
            params,
            locals,
            body,
        })
    }

    fn parse_var_list_stmt_boundary(&mut self) -> Result<Vec<String>, String> {
        let mut vars = Vec::new();
        if matches!(self.peek(), Token::Ident(_)) {
            vars.push(self.parse_ident()?);
            while matches!(self.peek(), Token::Comma) {
                self.next();
                vars.push(self.parse_ident()?);
            }
        }
        Ok(vars)
    }

    fn parse_var_list_until(&mut self, end: &Token) -> Result<Vec<String>, String> {
        let mut vars = Vec::new();
        if self.peek() == end {
            return Ok(vars);
        }
        vars.push(self.parse_ident()?);
        while matches!(self.peek(), Token::Comma) {
            self.next();
            vars.push(self.parse_ident()?);
        }
        Ok(vars)
    }

    fn parse_ident(&mut self) -> Result<String, String> {
        match self.next() {
            Token::Ident(s) => Ok(s),
            t => Err(format!("Expected identifier, got {t:?}")),
        }
    }

    fn parse_stmt(&mut self) -> Result<Stmt, String> {
        let left = self.parse_stmt_single()?;
        if matches!(self.peek(), Token::Semi) {
            self.next();
            if matches!(self.peek(), Token::RBrack) {
                return Ok(left);
            }
            let right = self.parse_stmt()?;
            Ok(Stmt::Seq(Box::new(left), Box::new(right)))
        } else {
            Ok(left)
        }
    }

    fn parse_stmt_single(&mut self) -> Result<Stmt, String> {
        match self.peek() {
            Token::Nop => {
                self.next();
                Ok(Stmt::Nop)
            }
            Token::If => {
                self.next();
                let cond = self.parse_bexp()?;
                self.expect(Token::LBrack)?;
                let body = self.parse_stmt()?;
                self.expect(Token::RBrack)?;
                Ok(Stmt::If {
                    cond,
                    body: Box::new(body),
                })
            }
            Token::While => {
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
            Token::Call => {
                self.next();
                let name = self.parse_ident()?;
                self.expect(Token::LParen)?;
                let args = self.parse_var_list_until(&Token::RParen)?;
                self.expect(Token::RParen)?;
                self.expect(Token::Arrow)?;
                let rets = self.parse_var_list_stmt_boundary()?;
                Ok(Stmt::Call { name, args, rets })
            }
            Token::Return => {
                self.next();
                let vars = self.parse_var_list_stmt_boundary()?;
                Ok(Stmt::Return { vars })
            }
            Token::LBrack => {
                self.next();
                let inner = self.parse_stmt()?;
                self.expect(Token::RBrack)?;
                Ok(inner)
            }
            Token::Ident(_) => {
                let var = self.parse_ident()?;
                self.expect(Token::Assign)?;
                let expr = self.parse_aexp()?;
                Ok(Stmt::Assign { var, expr })
            }
            t => Err(format!("Unexpected token in statement: {t:?}")),
        }
    }

    fn parse_aexp(&mut self) -> Result<AExp, String> {
        let lhs = self.parse_atom()?;
        if matches!(self.peek(), Token::Plus | Token::Minus) {
            let op = match self.next() {
                Token::Plus => ABinOp::Add,
                Token::Minus => ABinOp::Sub,
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
        let op = match self.next() {
            Token::Lt => RelOp::Lt,
            Token::Eq => RelOp::Eq,
            Token::Gt => RelOp::Gt,
            t => return Err(format!("Expected relation operator, got {t:?}")),
        };
        let rhs = self.parse_atom()?;
        Ok(BExp { lhs, op, rhs })
    }

    fn parse_atom(&mut self) -> Result<Atom, String> {
        match self.next() {
            Token::Ident(s) => Ok(Atom::Var(s)),
            Token::Number(s) => Ok(Atom::Imm(Number::parse(&s)?)),
            t => Err(format!("Expected atom, got {t:?}")),
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
                "static" => Token::Static,
                "local" => Token::Local,
                "Nop" => Token::Nop,
                "if" => Token::If,
                "while" => Token::While,
                "call" => Token::Call,
                "return" => Token::Return,
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
            ',' => {
                chars.next();
                tokens.push(Token::Comma);
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
            ':' => {
                chars.next();
                if chars.next() == Some('=') {
                    tokens.push(Token::Assign);
                } else {
                    return Err("Expected '=' after ':'".to_string());
                }
            }
            _ => return Err(format!("Unexpected character: {ch}")),
        }
    }
    tokens.push(Token::Eof);
    Ok(tokens)
}

fn atom_to_text(atom: &Atom) -> String {
    match atom {
        Atom::Var(v) => v.clone(),
        Atom::Imm(n) => n.to_decimal_string(),
    }
}

fn aexp_to_text(exp: &AExp) -> String {
    match exp {
        AExp::Atom(a) => atom_to_text(a),
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
    let op = match exp.op {
        RelOp::Lt => "<",
        RelOp::Eq => "=",
        RelOp::Gt => ">",
    };
    format!(
        "{} {} {}",
        atom_to_text(&exp.lhs),
        op,
        atom_to_text(&exp.rhs)
    )
}

pub fn stmt_to_text(stmt: &Stmt) -> String {
    match stmt {
        Stmt::Nop => "Nop".to_string(),
        Stmt::Assign { var, expr } => format!("{var} := {}", aexp_to_text(expr)),
        Stmt::Seq(a, b) => format!("{}; {}", stmt_to_text(a), stmt_to_text(b)),
        Stmt::If { cond, body } => format!("if {} [{}]", bexp_to_text(cond), stmt_to_text(body)),
        Stmt::While { cond, body } => {
            format!("while {} [{}]", bexp_to_text(cond), stmt_to_text(body))
        }
        Stmt::Call { name, args, rets } => {
            format!("call {}({}) -> {}", name, args.join(", "), rets.join(", "))
        }
        Stmt::Return { vars } => format!("return {}", vars.join(", ")),
    }
}
