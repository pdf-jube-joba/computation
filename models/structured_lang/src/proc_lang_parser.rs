use std::collections::BTreeMap;

use utils::number::Number;
use utils::{TextCodec, Token as LexToken, lex};

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
    let mut parser = Parser::new(lex(text).map_err(|e| e.to_string())?);
    let program = parser.parse_program()?;
    parser.expect_eof()?;
    Ok(program)
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

    fn expect_eof(&self) -> Result<(), String> {
        if self.is_eof() {
            Ok(())
        } else {
            Err(self.error_here("Unexpected trailing tokens"))
        }
    }

    fn error_here(&self, message: impl Into<String>) -> String {
        match self.peek() {
            Some(token) => format!("{} near {:?}", message.into(), token),
            None => format!("{} at end of input", message.into()),
        }
    }

    fn peek_ident(&self, ident: &str) -> bool {
        matches!(self.peek(), Some(LexToken::Ident(found)) if found == ident)
    }

    fn eat_ident(&mut self, ident: &str) -> bool {
        if self.peek_ident(ident) {
            self.pos += 1;
            true
        } else {
            false
        }
    }

    fn expect_ident(&mut self, ident: &str) -> Result<(), String> {
        if self.eat_ident(ident) {
            Ok(())
        } else {
            Err(self.error_here(format!("expected keyword '{ident}'")))
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

    fn parse_program(&mut self) -> Result<Program, String> {
        self.expect_ident("static")?;
        let statics = self.parse_var_list_until_symbol(';')?;
        self.expect_symbol(';')?;

        let mut procs = Vec::new();
        while !self.is_eof() {
            procs.push(self.parse_proc()?);
        }

        if procs.is_empty() {
            return Err("At least one procedure is required".to_string());
        }

        Ok(Program { statics, procs })
    }

    fn parse_proc(&mut self) -> Result<ProcDef, String> {
        let name = self.parse_ident()?;
        self.expect_symbol('(')?;
        let params = self.parse_var_list_until_symbol(')')?;
        self.expect_symbol(')')?;
        self.expect_symbol('[')?;
        self.expect_ident("local")?;
        let locals = self.parse_var_list_stmt_boundary()?;
        let body = self.parse_stmt()?;
        self.expect_symbol(']')?;
        Ok(ProcDef {
            name,
            params,
            locals,
            body,
        })
    }

    fn parse_var_list_stmt_boundary(&mut self) -> Result<Vec<String>, String> {
        let mut vars = Vec::new();
        if matches!(self.peek(), Some(LexToken::Ident(_))) {
            vars.push(self.parse_ident()?);
            while self.eat_symbol(',') {
                vars.push(self.parse_ident()?);
            }
        }
        Ok(vars)
    }

    fn parse_var_list_until_symbol(&mut self, end: char) -> Result<Vec<String>, String> {
        let mut vars = Vec::new();
        if self.peek_symbol(end) {
            return Ok(vars);
        }
        vars.push(self.parse_ident()?);
        while self.eat_symbol(',') {
            vars.push(self.parse_ident()?);
        }
        Ok(vars)
    }

    fn parse_ident(&mut self) -> Result<String, String> {
        match self.next() {
            Some(LexToken::Ident(s)) => Ok(s),
            _ => Err(self.error_here("expected identifier")),
        }
    }

    fn parse_stmt(&mut self) -> Result<Stmt, String> {
        let left = self.parse_stmt_single()?;
        if self.eat_symbol(';') {
            if self.peek_symbol(']') {
                return Ok(left);
            }
            let right = self.parse_stmt()?;
            Ok(Stmt::Seq(Box::new(left), Box::new(right)))
        } else {
            Ok(left)
        }
    }

    fn parse_stmt_single(&mut self) -> Result<Stmt, String> {
        if self.eat_ident("Nop") {
            return Ok(Stmt::Nop);
        }
        if self.eat_ident("if") {
            let cond = self.parse_bexp()?;
            self.expect_symbol('[')?;
            let body = self.parse_stmt()?;
            self.expect_symbol(']')?;
            return Ok(Stmt::If {
                cond,
                body: Box::new(body),
            });
        }
        if self.eat_ident("while") {
            let cond = self.parse_bexp()?;
            self.expect_symbol('[')?;
            let body = self.parse_stmt()?;
            self.expect_symbol(']')?;
            return Ok(Stmt::While {
                cond,
                body: Box::new(body),
            });
        }
        if self.eat_ident("call") {
            let name = self.parse_ident()?;
            self.expect_symbol('(')?;
            let args = self.parse_var_list_until_symbol(')')?;
            self.expect_symbol(')')?;
            self.expect_symbol('-')?;
            self.expect_symbol('>')?;
            let rets = self.parse_var_list_stmt_boundary()?;
            return Ok(Stmt::Call { name, args, rets });
        }
        if self.eat_ident("return") {
            let vars = self.parse_var_list_stmt_boundary()?;
            return Ok(Stmt::Return { vars });
        }
        if self.eat_symbol('[') {
            let inner = self.parse_stmt()?;
            self.expect_symbol(']')?;
            return Ok(inner);
        }
        if matches!(self.peek(), Some(LexToken::Ident(_))) {
            let var = self.parse_ident()?;
            self.expect_symbol(':')?;
            self.expect_symbol('=')?;
            let expr = self.parse_aexp()?;
            return Ok(Stmt::Assign { var, expr });
        }
        Err(self.error_here("unexpected token in statement"))
    }

    fn parse_aexp(&mut self) -> Result<AExp, String> {
        let lhs = self.parse_atom()?;
        if self.eat_symbol('+') {
            let rhs = self.parse_atom()?;
            Ok(AExp::Bin {
                lhs,
                op: ABinOp::Add,
                rhs,
            })
        } else if self.eat_symbol('-') {
            let rhs = self.parse_atom()?;
            Ok(AExp::Bin {
                lhs,
                op: ABinOp::Sub,
                rhs,
            })
        } else {
            Ok(AExp::Atom(lhs))
        }
    }

    fn parse_bexp(&mut self) -> Result<BExp, String> {
        let lhs = self.parse_atom()?;
        let op = if self.eat_symbol('<') {
            RelOp::Lt
        } else if self.eat_symbol('=') {
            RelOp::Eq
        } else if self.eat_symbol('>') {
            RelOp::Gt
        } else {
            return Err(self.error_here("expected relation operator"));
        };
        let rhs = self.parse_atom()?;
        Ok(BExp { lhs, op, rhs })
    }

    fn parse_atom(&mut self) -> Result<Atom, String> {
        match self.next() {
            Some(LexToken::Ident(s)) => Ok(Atom::Var(s)),
            Some(LexToken::Number(s)) => Ok(Atom::Imm(Number::parse(&s)?)),
            _ => Err(self.error_here("expected atom")),
        }
    }
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
