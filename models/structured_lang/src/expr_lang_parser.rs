use utils::identifier::Identifier;
use utils::number::Number;
use utils::{DelimKind, TextCodec, Token as LexToken, Tree, lex_tree};

use crate::expr_lang::*;

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
        parse_env(text)
    }

    fn write_fmt(&self, f: &mut impl std::fmt::Write) -> std::fmt::Result {
        for (var, value) in &self.vars {
            writeln!(f, "{} = {}", var.as_str(), value.to_decimal_string())?;
        }
        Ok(())
    }
}

impl TextCodec for ExprCode {
    fn parse(text: &str) -> Result<Self, String> {
        let trees = lex_tree(text).map_err(|e| e.to_string())?;
        let mut ps = Parser::new(normalize_trees(trees)?);
        let stmt = ps.parse_stmt()?;
        ps.expect_eof()?;
        Ok(ExprCode(stmt))
    }

    fn write_fmt(&self, f: &mut impl std::fmt::Write) -> std::fmt::Result {
        write!(f, "{}", stmt_to_text(&self.0))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Node {
    Token(LexToken),
    Paren(Vec<Node>),
    Bracket(Vec<Node>),
}

fn normalize_trees(trees: Vec<Tree>) -> Result<Vec<Node>, String> {
    let mut out = Vec::new();
    for tree in trees {
        match tree {
            Tree::Token(LexToken::Whitespace(_) | LexToken::Comment(_)) => {}
            Tree::Token(token) => out.push(Node::Token(token)),
            Tree::Delim {
                delim: DelimKind::Paren,
                child,
            } => out.push(Node::Paren(normalize_trees(child)?)),
            Tree::Delim {
                delim: DelimKind::Bracket,
                child,
            } => out.push(Node::Bracket(normalize_trees(child)?)),
            Tree::Delim { delim, .. } => {
                return Err(format!("unexpected delimiter in expr_lang: {:?}", delim));
            }
        }
    }
    Ok(out)
}

struct Parser {
    nodes: Vec<Node>,
    pos: usize,
}

impl Parser {
    fn new(nodes: Vec<Node>) -> Self {
        Self { nodes, pos: 0 }
    }

    fn peek(&self) -> Option<&Node> {
        self.nodes.get(self.pos)
    }

    fn next(&mut self) -> Option<Node> {
        let t = self.nodes.get(self.pos).cloned();
        if t.is_some() {
            self.pos += 1;
        }
        t
    }

    fn error_here(&self, message: impl Into<String>) -> String {
        match self.peek() {
            Some(token) => format!("{} near {:?}", message.into(), token),
            None => format!("{} at end of input", message.into()),
        }
    }

    fn expect_eof(&self) -> Result<(), String> {
        if self.pos == self.nodes.len() {
            Ok(())
        } else {
            Err(self.error_here("Unexpected trailing tokens"))
        }
    }

    fn peek_symbol(&self, ch: char) -> bool {
        matches!(self.peek(), Some(Node::Token(LexToken::Symbol(found))) if *found == ch)
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
        matches!(self.peek(), Some(Node::Token(LexToken::Ident(found))) if found == word)
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

    fn parse_stmt(&mut self) -> Result<Stmt, String> {
        let mut left = self.parse_stmt_single()?;
        while self.eat_symbol(';') {
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
            let body = self.parse_stmt_single()?;
            return Ok(Stmt::If {
                cond,
                body: Box::new(body),
            });
        }
        if self.eat_ident("while") {
            let cond = self.parse_bexp()?;
            let body = self.parse_bracketed_stmt()?;
            return Ok(Stmt::While {
                cond,
                body: Box::new(body),
            });
        }
        if matches!(self.peek(), Some(Node::Paren(_))) {
            let s = self.parse_paren_stmt()?;
            return Ok(s);
        }
        if matches!(self.peek(), Some(Node::Token(LexToken::Ident(_)))) {
            let var = self.parse_ident()?;
            self.expect_symbol(':')?;
            self.expect_symbol('=')?;
            let expr = self.parse_aexp()?;
            return Ok(Stmt::Assign { var, expr });
        }
        Err(self.error_here("Invalid statement"))
    }

    fn parse_aexp(&mut self) -> Result<AExp, String> {
        if self.eat_ident("if") {
            let cond = self.parse_bexp()?;
            self.expect_ident("then")?;
            let then_exp = self.parse_aexp()?;
            self.expect_ident("else")?;
            let else_exp = self.parse_aexp()?;
            return Ok(AExp::IfThenElse {
                cond: Box::new(cond),
                then_exp: Box::new(then_exp),
                else_exp: Box::new(else_exp),
            });
        }

        let mut left = self.parse_aexp_atom()?;
        loop {
            let op = if self.eat_symbol('+') {
                Some(ABinOp::Add)
            } else if self.eat_symbol('-') {
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
            Some(Node::Token(LexToken::Ident(s))) => {
                Ok(AExp::Var(Identifier::new(&s).map_err(|e| e.to_string())?))
            }
            Some(Node::Token(LexToken::Number(n))) => Ok(AExp::Num(Number::parse(&n)?)),
            Some(Node::Paren(inner)) => Self::parse_group_as_aexp(inner),
            _ => Err(self.error_here("Invalid aexp")),
        }
    }

    fn parse_bexp(&mut self) -> Result<BExp, String> {
        let mut left = self.parse_bexp_not()?;
        while self.eat_symbol('|') {
            self.expect_symbol('|')?;
            let rhs = self.parse_bexp_not()?;
            left = BExp::Or(Box::new(left), Box::new(rhs));
        }
        Ok(left)
    }

    fn parse_bexp_not(&mut self) -> Result<BExp, String> {
        if self.eat_symbol('!') {
            return Ok(BExp::Not(Box::new(self.parse_bexp_not()?)));
        }
        self.parse_bexp_atom()
    }

    fn parse_bexp_atom(&mut self) -> Result<BExp, String> {
        if matches!(self.peek(), Some(Node::Paren(_))) {
            let Some(Node::Paren(inner)) = self.next() else {
                unreachable!();
            };
            return Self::parse_group_as_bexp(inner);
        }
        let lhs = self.parse_aexp()?;
        let rel = if self.eat_symbol('<') {
            RelOp::Lt
        } else if self.eat_symbol('=') {
            RelOp::Eq
        } else if self.eat_symbol('>') {
            RelOp::Gt
        } else {
            return Err(self.error_here("Expected relation operator"));
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
            Some(Node::Token(LexToken::Ident(s))) => Identifier::new(&s).map_err(|e| e.to_string()),
            _ => Err(self.error_here("Expected identifier")),
        }
    }

    fn parse_paren_stmt(&mut self) -> Result<Stmt, String> {
        let Some(Node::Paren(inner)) = self.next() else {
            return Err(self.error_here("expected parenthesized statement"));
        };
        let mut parser = Parser::new(inner);
        let stmt = parser.parse_stmt()?;
        parser.expect_eof()?;
        Ok(stmt)
    }

    fn parse_bracketed_stmt(&mut self) -> Result<Stmt, String> {
        let Some(Node::Bracket(inner)) = self.next() else {
            return Err(self.error_here("expected bracketed statement"));
        };
        let mut parser = Parser::new(inner);
        let stmt = parser.parse_stmt()?;
        parser.expect_eof()?;
        Ok(stmt)
    }

    fn parse_group_as_aexp(inner: Vec<Node>) -> Result<AExp, String> {
        let mut parser = Parser::new(inner);
        let exp = parser.parse_aexp()?;
        parser.expect_eof()?;
        Ok(exp)
    }

    fn parse_group_as_bexp(inner: Vec<Node>) -> Result<BExp, String> {
        let mut parser = Parser::new(inner);
        let exp = parser.parse_bexp()?;
        parser.expect_eof()?;
        Ok(exp)
    }
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
    fn expr_code_roundtrip() {
        let text = r#"
while x < 3 [
  if !(x = 1) y := if x < 2 then x + 1 else x - 1 ;
  x := x + 1
]
"#;
        let parsed = ExprCode::parse(text).unwrap();
        let printed = parsed.print();
        let reparsed = ExprCode::parse(&printed).unwrap();
        assert_eq!(parsed, reparsed);
    }

    #[test]
    fn smallstep_runs() {
        let code = ExprCode::parse("x := 0 ; while x < 3 [ x := x + 1 ]").unwrap();
        let env = Environment::default();
        let mut machine = ExprLangMachine::make(code, env).unwrap();
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
