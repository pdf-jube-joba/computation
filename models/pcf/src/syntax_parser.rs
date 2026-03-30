use utils::{
    TextCodec,
    identifier::Identifier,
    lexer::{DelimKind, Token as LexToken, Tree, lex_tree},
};

use crate::syntax::{BinOp, Expr, ExprCode, UnOp};

impl TextCodec for ExprCode {
    fn parse(text: &str) -> Result<Self, String> {
        let trees = lex_tree(text).map_err(|e| e.to_string())?;
        let mut parser = Parser::new(&trees);
        let expr = parser.parse_expr()?;
        parser.expect_eof()?;
        Ok(Self(expr))
    }

    fn write_fmt(&self, f: &mut impl std::fmt::Write) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Clone, Copy)]
struct Parser<'a> {
    nodes: &'a [Tree],
    pos: usize,
}

impl<'a> Parser<'a> {
    fn new(nodes: &'a [Tree]) -> Self {
        Self { nodes, pos: 0 }
    }

    fn expect_eof(&mut self) -> Result<(), String> {
        self.skip_trivia();
        if self.pos == self.nodes.len() {
            Ok(())
        } else {
            Err("unexpected trailing tokens".to_string())
        }
    }

    fn parse_expr(&mut self) -> Result<Expr, String> {
        if self.peek_keyword("if") {
            self.parse_if()
        } else {
            self.parse_binop_expr()
        }
    }

    fn parse_if(&mut self) -> Result<Expr, String> {
        self.expect_keyword("if")?;
        let cond = self.parse_expr()?;
        self.expect_keyword("then")?;
        let then_branch = self.parse_expr()?;
        self.expect_keyword("else")?;
        let else_branch = self.parse_expr()?;
        self.expect_keyword("fi")?;
        Ok(Expr::If {
            cond: Box::new(cond),
            then_branch: Box::new(then_branch),
            else_branch: Box::new(else_branch),
        })
    }

    fn parse_binop_expr(&mut self) -> Result<Expr, String> {
        let mut expr = self.parse_application()?;
        loop {
            let op = if self.eat_symbol('+') {
                Some(BinOp::Add)
            } else if self.eat_symbol('-') {
                Some(BinOp::Sub)
            } else if self.eat_symbol2('&', '&') {
                Some(BinOp::And)
            } else {
                None
            };
            let Some(op) = op else { break };
            let rhs = self.parse_application()?;
            expr = Expr::BinOp(Box::new(expr), op, Box::new(rhs));
        }
        Ok(expr)
    }

    fn parse_application(&mut self) -> Result<Expr, String> {
        let mut expr = self.parse_prefix()?;
        loop {
            let Some(child) = self.eat_delim(DelimKind::Paren) else {
                break;
            };
            let arg = Self::parse_group_expr(child)?;
            expr = Expr::App(Box::new(expr), Box::new(arg));
        }
        Ok(expr)
    }

    fn parse_prefix(&mut self) -> Result<Expr, String> {
        if self.eat_keyword("inc") {
            return Ok(Expr::UnOp(UnOp::Inc, Box::new(self.parse_prefix()?)));
        }
        if self.eat_keyword("dec") {
            return Ok(Expr::UnOp(UnOp::Dec, Box::new(self.parse_prefix()?)));
        }
        if self.eat_keyword("not") {
            return Ok(Expr::UnOp(UnOp::Not, Box::new(self.parse_prefix()?)));
        }
        self.parse_atom()
    }

    fn parse_atom(&mut self) -> Result<Expr, String> {
        if let Some(value) = self.eat_number() {
            return Ok(Expr::Nat(value));
        }
        if self.eat_symbol('#') {
            let keyword = self.expect_ident_text()?;
            return match keyword.as_str() {
                "true" => Ok(Expr::Bool(true)),
                "false" => Ok(Expr::Bool(false)),
                "unit" => Ok(Expr::Unit),
                _ => Err(format!("unknown token: #{keyword}")),
            };
        }
        if self.eat_keyword("print") {
            let value = self
                .eat_number()
                .ok_or_else(|| "print expects a natural number literal".to_string())?;
            return Ok(Expr::Print(value));
        }
        if self.eat_keyword("fun") {
            let param = self.parse_identifier()?;
            self.expect_symbol2('=', '>')?;
            let body = self.parse_expr()?;
            return Ok(Expr::Fun {
                param,
                body: Box::new(body),
            });
        }
        if self.eat_keyword("rec") {
            let name = self.parse_identifier()?;
            let param = self.parse_identifier()?;
            self.expect_symbol2('=', '>')?;
            let body = self.parse_expr()?;
            return Ok(Expr::Rec {
                name,
                param,
                body: Box::new(body),
            });
        }
        if self.peek_keyword("if") {
            return self.parse_if();
        }
        if let Some(child) = self.eat_delim(DelimKind::Paren) {
            return Self::parse_group_expr(child);
        }
        let ident = self.parse_identifier()?;
        Ok(Expr::Var(ident))
    }

    fn parse_identifier(&mut self) -> Result<Identifier, String> {
        let name = self.expect_ident_text()?;
        if is_reserved(&name) {
            return Err(format!("identifier expected, found keyword '{name}'"));
        }
        Identifier::new(name).map_err(|e| e.to_string())
    }

    fn parse_group_expr(child: &'a [Tree]) -> Result<Expr, String> {
        let mut parser = Self::new(child);
        let expr = parser.parse_expr()?;
        parser.expect_eof()?;
        Ok(expr)
    }

    fn skip_trivia(&mut self) {
        while matches!(
            self.nodes.get(self.pos),
            Some(Tree::Token(LexToken::Whitespace(_) | LexToken::Comment(_)))
        ) {
            self.pos += 1;
        }
    }

    fn peek(&mut self) -> Option<&'a Tree> {
        self.skip_trivia();
        self.nodes.get(self.pos)
    }

    fn eat_number(&mut self) -> Option<usize> {
        match self.peek()? {
            Tree::Token(LexToken::Number(text)) => {
                self.pos += 1;
                Some(text.parse::<usize>().ok()?)
            }
            _ => None,
        }
    }

    fn expect_ident_text(&mut self) -> Result<String, String> {
        match self.peek() {
            Some(Tree::Token(LexToken::Ident(text))) => {
                self.pos += 1;
                Ok(text.clone())
            }
            other => Err(format!("identifier expected, found {other:?}")),
        }
    }

    fn peek_keyword(&mut self, keyword: &str) -> bool {
        matches!(
            self.peek(),
            Some(Tree::Token(LexToken::Ident(text))) if text == keyword
        )
    }

    fn eat_keyword(&mut self, keyword: &str) -> bool {
        if self.peek_keyword(keyword) {
            self.pos += 1;
            true
        } else {
            false
        }
    }

    fn expect_keyword(&mut self, keyword: &str) -> Result<(), String> {
        if self.eat_keyword(keyword) {
            Ok(())
        } else {
            Err(format!("expected keyword '{keyword}'"))
        }
    }

    fn eat_symbol(&mut self, ch: char) -> bool {
        matches!(self.peek(), Some(Tree::Token(LexToken::Symbol(found))) if *found == ch)
            .then(|| self.pos += 1)
            .is_some()
    }

    fn eat_symbol2(&mut self, first: char, second: char) -> bool {
        let checkpoint = self.pos;
        if self.eat_symbol(first) && self.eat_symbol(second) {
            true
        } else {
            self.pos = checkpoint;
            false
        }
    }

    fn expect_symbol2(&mut self, first: char, second: char) -> Result<(), String> {
        if self.eat_symbol2(first, second) {
            Ok(())
        } else {
            Err(format!("expected '{}{}'", first, second))
        }
    }

    fn eat_delim(&mut self, delim: DelimKind) -> Option<&'a [Tree]> {
        match self.peek()? {
            Tree::Delim { delim: found, child } if *found == delim => {
                self.pos += 1;
                Some(child.as_slice())
            }
            _ => None,
        }
    }
}

fn is_reserved(word: &str) -> bool {
    matches!(
        word,
        "print" | "fun" | "rec" | "if" | "then" | "else" | "fi" | "inc" | "dec" | "not"
    )
}
