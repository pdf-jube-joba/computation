use utils::identifier::Identifier;
use utils::number::Number;
use utils::{TextCodec, Token as LexToken, lex};

use crate::internal_ctrl::{ABinOp, AExp, Atom, BExp, Environment, InternalCtrlCode, RelOp, Stmt};

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
        let mut parser = Parser::new(lex(text).map_err(|e| e.to_string())?);
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

    fn peek(&self) -> Option<&LexToken> {
        self.tokens.get(self.pos)
    }

    fn next(&mut self) -> Option<LexToken> {
        let tok = self.tokens.get(self.pos).cloned();
        if tok.is_some() {
            self.pos += 1;
        }
        tok
    }

    fn error_here(&self, message: impl Into<String>) -> String {
        match self.peek() {
            Some(token) => format!("{} near {:?}", message.into(), token),
            None => format!("{} at end of input", message.into()),
        }
    }

    fn is_eof(&self) -> bool {
        self.pos >= self.tokens.len()
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

    fn expect_ident(&mut self, word: &str) -> Result<(), String> {
        if self.eat_ident(word) {
            Ok(())
        } else {
            Err(self.error_here(format!("expected keyword '{word}'")))
        }
    }

    fn parse_program(&mut self) -> Result<InternalCtrlCode, String> {
        self.expect_ident("static")?;
        let mut statics = Vec::new();
        if !self.peek_symbol(';') {
            statics.push(self.parse_ident()?);
            while self.eat_symbol(',') {
                statics.push(self.parse_ident()?);
            }
        }
        self.expect_symbol(';')?;
        let body = self.parse_stmt()?;
        if !self.is_eof() {
            return Err(self.error_here("Unexpected trailing tokens"));
        }
        Ok(InternalCtrlCode { statics, body })
    }

    fn parse_stmt(&mut self) -> Result<Stmt, String> {
        let mut left = self.parse_stmt_single()?;
        while self.eat_symbol(';') {
            if self.peek().is_none() || self.peek_symbol('}') {
                break;
            }
            let right = self.parse_stmt_single()?;
            left = Stmt::Seq(Box::new(left), Box::new(right));
        }
        Ok(left)
    }

    fn parse_stmt_single(&mut self) -> Result<Stmt, String> {
        if self.eat_ident("Nop") {
            return Ok(Stmt::Nop);
        }
        if self.eat_ident("if") {
            let cond = self.parse_bexp()?;
            let body = self.parse_block()?;
            return Ok(Stmt::If {
                cond,
                body: Box::new(body),
            });
        }
        if self.eat_ident("break") {
            let label = self.parse_label()?;
            let value = self.parse_ident()?;
            return Ok(Stmt::Break { label, value });
        }
        if self.eat_ident("continue") {
            let label = self.parse_label()?;
            return Ok(Stmt::Continue { label });
        }
        if self.eat_ident("loop") {
            let label = self.parse_label()?;
            self.expect_symbol('(')?;
            let body = self.parse_stmt()?;
            self.expect_symbol(')')?;
            self.expect_symbol('-')?;
            self.expect_symbol('>')?;
            let out = self.parse_ident()?;
            return Ok(Stmt::Loop {
                label,
                body: Box::new(body),
                out,
            });
        }
        if self.peek_symbol('{') {
            return self.parse_block();
        }
        if matches!(self.peek(), Some(LexToken::Ident(_))) {
            let var = self.parse_ident()?;
            self.expect_symbol(':')?;
            self.expect_symbol('=')?;
            let expr = self.parse_aexp()?;
            return Ok(Stmt::Assign { var, expr });
        }
        Err(self.error_here("Invalid statement"))
    }

    fn parse_block(&mut self) -> Result<Stmt, String> {
        self.expect_symbol('{')?;
        let stmt = self.parse_stmt()?;
        self.expect_symbol('}')?;
        Ok(stmt)
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
        let rel = if self.eat_symbol('<') {
            RelOp::Lt
        } else if self.eat_symbol('=') {
            RelOp::Eq
        } else if self.eat_symbol('>') {
            RelOp::Gt
        } else {
            return Err(self.error_here("Expected relation operator"));
        };
        let rhs = self.parse_atom()?;
        Ok(BExp { lhs, rel, rhs })
    }

    fn parse_atom(&mut self) -> Result<Atom, String> {
        match self.next() {
            Some(LexToken::Ident(name)) => Ok(Atom::Var(
                Identifier::new(&name).map_err(|e| e.to_string())?,
            )),
            Some(LexToken::Number(num)) => Ok(Atom::Imm(Number::parse(&num)?)),
            _ => Err(self.error_here("Expected atom")),
        }
    }

    fn parse_ident(&mut self) -> Result<Identifier, String> {
        match self.next() {
            Some(LexToken::Ident(name)) => Identifier::new(&name).map_err(|e| e.to_string()),
            _ => Err(self.error_here("Expected identifier")),
        }
    }

    fn parse_label(&mut self) -> Result<Identifier, String> {
        self.expect_symbol(':')?;
        self.parse_ident()
    }
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
    format!(
        "{} {} {}",
        atom_to_text(&exp.lhs),
        rel,
        atom_to_text(&exp.rhs)
    )
}

pub fn stmt_to_text(stmt: &Stmt) -> String {
    match stmt {
        Stmt::Nop => "Nop".to_string(),
        Stmt::Seq(lhs, rhs) => format!("{}; {}", stmt_to_text(lhs), stmt_to_text(rhs)),
        Stmt::Assign { var, expr } => format!("{} := {}", var.as_str(), aexp_to_text(expr)),
        Stmt::If { cond, body } => {
            format!("if {} {{ {} }}", bexp_to_text(cond), stmt_to_text(body))
        }
        Stmt::Break { label, value } => format!("break :{} {}", label.as_str(), value.as_str()),
        Stmt::Continue { label } => format!("continue :{}", label.as_str()),
        Stmt::Loop { label, body, out } => {
            format!(
                "loop :{} ({}) -> {}",
                label.as_str(),
                stmt_to_text(body),
                out.as_str()
            )
        }
    }
}
