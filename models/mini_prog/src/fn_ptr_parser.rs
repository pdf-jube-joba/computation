use utils::TextCodec;

use crate::fn_ptr_machine::{BinOp, FnDecl, FnPtrCode, PlaceExpr, Program, Stmt, ValueExpr};

impl TextCodec for FnPtrCode {
    fn parse(text: &str) -> Result<Self, String> {
        let tokens = lex(text)?;
        let mut parser = Parser::new(tokens);
        let program = parser.parse_program()?;
        parser.expect_eof()?;
        Ok(Self(program))
    }

    fn write_fmt(&self, f: &mut impl std::fmt::Write) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Token {
    Ident(String),
    Number(i64),
    Fn,
    AssignKw,
    Ifz,
    Then,
    Else,
    End,
    Call,
    Return,
    LParen,
    RParen,
    LBrace,
    RBrace,
    Comma,
    Semi,
    Assign,
    Plus,
    Minus,
    Load,
    NullPtr,
    Addr,
    Loc,
}

fn lex(text: &str) -> Result<Vec<Token>, String> {
    let mut tokens = Vec::new();
    let chars: Vec<char> = text.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        let ch = chars[i];
        if ch.is_whitespace() {
            i += 1;
            continue;
        }

        if ch == '/' && i + 1 < chars.len() && chars[i + 1] == '/' {
            i += 2;
            while i < chars.len() && chars[i] != '\n' {
                i += 1;
            }
            continue;
        }

        if ch.is_ascii_digit() {
            let start = i;
            i += 1;
            while i < chars.len() && chars[i].is_ascii_digit() {
                i += 1;
            }
            tokens.push(Token::Number(
                text[start..i].parse::<i64>().map_err(|e| e.to_string())?,
            ));
            continue;
        }

        if ch.is_ascii_alphabetic() || ch == '_' {
            let start = i;
            i += 1;
            while i < chars.len() && (chars[i].is_ascii_alphanumeric() || chars[i] == '_') {
                i += 1;
            }
            let word = &text[start..i];
            tokens.push(match word {
                "fn" => Token::Fn,
                "assign" => Token::AssignKw,
                "ifz" => Token::Ifz,
                "then" => Token::Then,
                "else" => Token::Else,
                "end" => Token::End,
                "call" => Token::Call,
                "return" => Token::Return,
                "ld" => Token::Load,
                _ => Token::Ident(word.to_string()),
            });
            continue;
        }

        if ch == '#' {
            if text[i..].starts_with("#null-ptr") {
                i += "#null-ptr".len();
                tokens.push(Token::NullPtr);
                continue;
            }
            if text[i..].starts_with("#addr") {
                i += "#addr".len();
                tokens.push(Token::Addr);
                continue;
            }
            if text[i..].starts_with("#loc") {
                i += "#loc".len();
                tokens.push(Token::Loc);
                continue;
            }
            return Err(format!("unknown token starting with '#': {}", &text[i..]));
        }

        match ch {
            '(' => tokens.push(Token::LParen),
            ')' => tokens.push(Token::RParen),
            '{' => tokens.push(Token::LBrace),
            '}' => tokens.push(Token::RBrace),
            ',' => tokens.push(Token::Comma),
            ';' => tokens.push(Token::Semi),
            '+' => tokens.push(Token::Plus),
            '-' => tokens.push(Token::Minus),
            ':' => {
                if i + 1 < chars.len() && chars[i + 1] == '=' {
                    tokens.push(Token::Assign);
                    i += 2;
                    continue;
                }
                return Err("expected ':='".to_string());
            }
            _ => return Err(format!("unexpected character: {ch}")),
        }
        i += 1;
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

    fn parse_program(&mut self) -> Result<Program, String> {
        let mut functions = Vec::new();
        while !self.is_eof() {
            functions.push(self.parse_fn_decl()?);
        }
        Ok(Program { functions })
    }

    fn parse_fn_decl(&mut self) -> Result<FnDecl, String> {
        self.expect_token(&Token::Fn)?;
        let name = self.expect_ident()?;
        self.expect_token(&Token::LParen)?;
        let params = self.parse_ident_list(Token::RParen)?;
        self.expect_token(&Token::LBrace)?;
        let body = self.parse_stmt_list()?;
        self.expect_token(&Token::RBrace)?;
        Ok(FnDecl { name, params, body })
    }

    fn parse_stmt_list(&mut self) -> Result<Vec<Stmt>, String> {
        let mut stmts = Vec::new();
        if self.peek() == Some(&Token::RBrace) {
            return Ok(stmts);
        }
        loop {
            stmts.push(self.parse_stmt()?);
            if self.peek() == Some(&Token::Semi) {
                self.pos += 1;
                if self.peek() == Some(&Token::RBrace) {
                    break;
                }
            } else {
                break;
            }
        }
        Ok(stmts)
    }

    fn parse_stmt(&mut self) -> Result<Stmt, String> {
        match self.peek() {
            Some(Token::AssignKw) => {
                self.pos += 1;
                let place = self.parse_place_expr()?;
                self.expect_token(&Token::Assign)?;
                let value = self.parse_value_expr()?;
                Ok(Stmt::Assign { place, value })
            }
            Some(Token::Ifz) => {
                self.pos += 1;
                let cond = self.parse_value_expr()?;
                self.expect_token(&Token::Then)?;
                let then_branch = self.parse_stmt()?;
                self.expect_token(&Token::Else)?;
                let else_branch = self.parse_stmt()?;
                self.expect_token(&Token::End)?;
                Ok(Stmt::Ifz {
                    cond,
                    then_branch: Box::new(then_branch),
                    else_branch: Box::new(else_branch),
                })
            }
            Some(Token::Call) => {
                self.pos += 1;
                let name = self.expect_ident()?;
                self.expect_token(&Token::LParen)?;
                let args = self.parse_value_list()?;
                self.expect_token(&Token::RParen)?;
                Ok(Stmt::Call { name, args })
            }
            Some(Token::Return) => {
                self.pos += 1;
                Ok(Stmt::Return)
            }
            other => Err(format!("unexpected token in statement: {other:?}")),
        }
    }

    fn parse_ident_list(&mut self, end: Token) -> Result<Vec<String>, String> {
        let mut items = Vec::new();
        if self.peek() == Some(&end) {
            self.pos += 1;
            return Ok(items);
        }
        loop {
            items.push(self.expect_ident()?);
            match self.peek() {
                Some(Token::Comma) => self.pos += 1,
                Some(token) if *token == end => {
                    self.pos += 1;
                    return Ok(items);
                }
                other => return Err(format!("unexpected token in list: {other:?}")),
            }
        }
    }

    fn parse_value_list(&mut self) -> Result<Vec<ValueExpr>, String> {
        let mut items = Vec::new();
        if self.peek() == Some(&Token::RParen) {
            return Ok(items);
        }
        loop {
            items.push(self.parse_value_expr()?);
            match self.peek() {
                Some(Token::Comma) => self.pos += 1,
                Some(Token::RParen) => return Ok(items),
                other => return Err(format!("unexpected token in argument list: {other:?}")),
            }
        }
    }

    fn parse_place_expr(&mut self) -> Result<PlaceExpr, String> {
        if let Some(Token::Ident(name)) = self.peek().cloned() {
            self.pos += 1;
            return Ok(PlaceExpr::Var(name));
        }

        self.expect_token(&Token::LParen)?;
        let value = self.parse_value_expr()?;
        self.expect_token(&Token::RParen)?;
        self.expect_token(&Token::Loc)?;
        Ok(PlaceExpr::Deref(Box::new(value)))
    }

    fn parse_value_expr(&mut self) -> Result<ValueExpr, String> {
        self.parse_add_sub()
    }

    fn parse_add_sub(&mut self) -> Result<ValueExpr, String> {
        let mut expr = self.parse_value_atom()?;
        loop {
            let op = match self.peek() {
                Some(Token::Plus) => BinOp::Add,
                Some(Token::Minus) => BinOp::Sub,
                _ => break,
            };
            self.pos += 1;
            let rhs = self.parse_value_atom()?;
            expr = ValueExpr::BinOp {
                lhs: Box::new(expr),
                op,
                rhs: Box::new(rhs),
            };
        }
        Ok(expr)
    }

    fn parse_value_atom(&mut self) -> Result<ValueExpr, String> {
        match self.peek().cloned() {
            Some(Token::Number(n)) => {
                self.pos += 1;
                Ok(ValueExpr::Number(n))
            }
            Some(Token::NullPtr) => {
                self.pos += 1;
                Ok(ValueExpr::NullPtr)
            }
            Some(Token::Load) => {
                self.pos += 1;
                Ok(ValueExpr::Load(Box::new(self.parse_place_expr()?)))
            }
            Some(Token::LParen) => {
                self.pos += 1;
                let expr = self.parse_value_expr()?;
                self.expect_token(&Token::RParen)?;
                Ok(expr)
            }
            Some(Token::Ident(_)) => {
                let place = self.parse_place_expr()?;
                self.expect_token(&Token::Addr)?;
                Ok(ValueExpr::Addr(Box::new(place)))
            }
            other => Err(format!("unexpected token in value expression: {other:?}")),
        }
    }

    fn expect_ident(&mut self) -> Result<String, String> {
        match self.peek().cloned() {
            Some(Token::Ident(name)) => {
                self.pos += 1;
                Ok(name)
            }
            other => Err(format!("expected identifier, found {other:?}")),
        }
    }

    fn expect_token(&mut self, token: &Token) -> Result<(), String> {
        match self.peek() {
            Some(next) if next == token => {
                self.pos += 1;
                Ok(())
            }
            other => Err(format!("expected {token:?}, found {other:?}")),
        }
    }

    fn expect_eof(&self) -> Result<(), String> {
        if self.is_eof() {
            Ok(())
        } else {
            Err(format!("unexpected trailing tokens: {:?}", &self.tokens[self.pos..]))
        }
    }

    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.pos)
    }

    fn is_eof(&self) -> bool {
        self.pos >= self.tokens.len()
    }
}
