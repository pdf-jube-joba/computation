use utils::TextCodec;

use crate::mini_prog_machine::{
    BinOp, Block, FnDecl, MiniProgCode, PlaceExpr, Program, StaticDecl, Stmt, Type, UnOp,
    ValueExpr,
};

impl TextCodec for MiniProgCode {
    fn parse(text: &str) -> Result<Self, String> {
        let tokens = lex(text)?;
        let mut parser = Parser::new(tokens);
        let program = parser.parse_program()?;
        parser.expect_eof()?;
        Ok(Self(program))
    }

    fn write_fmt(&self, f: &mut impl std::fmt::Write) -> std::fmt::Result {
        write!(f, "{:?}", self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Token {
    Ident(String),
    Number(usize),
    Char(char),
    TypeNum,
    TypeU8,
    TypeBool,
    TypeUnit,
    TypePtr,
    TypeFn,
    True,
    False,
    NullPtr,
    NullFn,
    Static,
    Local,
    Fn,
    Assign,
    If,
    Case,
    Of,
    HAlloc,
    HFree,
    Loop,
    Break,
    Continue,
    Call,
    Return,
    Block,
    Load,
    Pair,
    Tag,
    Arrow,
    ColonEq,
    Colon,
    Comma,
    Semi,
    Dot,
    Question,
    LParen,
    RParen,
    LBrace,
    RBrace,
    LBracket,
    RBracket,
    Plus,
    Minus,
    EqEq,
    Lt,
    AndAnd,
    OrOr,
    Bang,
    HashAddr,
    HashLoc,
    Star,
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
        if ch == '\'' {
            if i + 2 >= chars.len() || chars[i + 2] != '\'' {
                return Err("invalid char literal".to_string());
            }
            tokens.push(Token::Char(chars[i + 1]));
            i += 3;
            continue;
        }
        if ch.is_ascii_digit() {
            let start = i;
            i += 1;
            while i < chars.len() && chars[i].is_ascii_digit() {
                i += 1;
            }
            tokens.push(Token::Number(
                text[start..i].parse::<usize>().map_err(|e| e.to_string())?,
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
                "static" => Token::Static,
                "local" => Token::Local,
                "fn" => Token::Fn,
                "assign" => Token::Assign,
                "if" => Token::If,
                "case" => Token::Case,
                "of" => Token::Of,
                "halloc" => Token::HAlloc,
                "hfree" => Token::HFree,
                "loop" => Token::Loop,
                "break" => Token::Break,
                "continue" => Token::Continue,
                "call" => Token::Call,
                "return" => Token::Return,
                "block" => Token::Block,
                "ld" => Token::Load,
                "pair" => Token::Pair,
                "tag" => Token::Tag,
                _ => Token::Ident(word.to_string()),
            });
            continue;
        }
        if ch == '#' {
            let rest = &text[i..];
            let matched = [
                ("#num", Token::TypeNum),
                ("#u8", Token::TypeU8),
                ("#bool", Token::TypeBool),
                ("#unit", Token::TypeUnit),
                ("#ptr", Token::TypePtr),
                ("#fn", Token::TypeFn),
                ("#true", Token::True),
                ("#false", Token::False),
                ("#null-ptr", Token::NullPtr),
                ("#null-fn", Token::NullFn),
                ("#addr", Token::HashAddr),
                ("#loc", Token::HashLoc),
            ]
            .into_iter()
            .find(|(pat, _)| rest.starts_with(pat));
            if let Some((pat, token)) = matched {
                tokens.push(token);
                i += pat.len();
                continue;
            }
            return Err(format!("unknown token starting with '#': {rest}"));
        }
        match ch {
            '(' => tokens.push(Token::LParen),
            ')' => tokens.push(Token::RParen),
            '{' => tokens.push(Token::LBrace),
            '}' => tokens.push(Token::RBrace),
            '[' => tokens.push(Token::LBracket),
            ']' => tokens.push(Token::RBracket),
            ',' => tokens.push(Token::Comma),
            ';' => tokens.push(Token::Semi),
            '.' => tokens.push(Token::Dot),
            '?' => tokens.push(Token::Question),
            '*' => tokens.push(Token::Star),
            '!' => tokens.push(Token::Bang),
            '<' => tokens.push(Token::Lt),
            '+' => tokens.push(Token::Plus),
            '-' => {
                if i + 1 < chars.len() && chars[i + 1] == '>' {
                    tokens.push(Token::Arrow);
                    i += 2;
                    continue;
                }
                tokens.push(Token::Minus);
            }
            ':' => {
                if i + 1 < chars.len() && chars[i + 1] == '=' {
                    tokens.push(Token::ColonEq);
                    i += 2;
                    continue;
                }
                tokens.push(Token::Colon);
            }
            '=' => {
                if i + 1 < chars.len() && chars[i + 1] == '=' {
                    tokens.push(Token::EqEq);
                    i += 2;
                    continue;
                }
                return Err("unexpected '='".to_string());
            }
            '&' if i + 1 < chars.len() && chars[i + 1] == '&' => {
                tokens.push(Token::AndAnd);
                i += 2;
                continue;
            }
            '|' if i + 1 < chars.len() && chars[i + 1] == '|' => {
                tokens.push(Token::OrOr);
                i += 2;
                continue;
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
        let mut statics = Vec::new();
        let mut functions = Vec::new();
        while !self.is_eof() {
            match self.peek() {
                Some(Token::Static) => statics.push(self.parse_static_decl()?),
                Some(Token::Fn) => functions.push(self.parse_fn_decl()?),
                other => return Err(format!("unexpected token at top level: {other:?}")),
            }
        }
        Ok(Program { statics, functions })
    }

    fn parse_static_decl(&mut self) -> Result<StaticDecl, String> {
        self.expect(Token::Static)?;
        let name = self.expect_ident()?;
        self.expect(Token::Colon)?;
        let ty = self.parse_type()?;
        Ok(StaticDecl { name, ty })
    }

    fn parse_fn_decl(&mut self) -> Result<FnDecl, String> {
        self.expect(Token::Fn)?;
        let name = self.expect_ident()?;
        let block = self.parse_block()?;
        Ok(FnDecl { name, block })
    }

    fn parse_block(&mut self) -> Result<Block, String> {
        self.expect(Token::LParen)?;
        let mut bindings = Vec::new();
        if !matches!(self.peek(), Some(Token::RParen)) {
            loop {
                let name = self.expect_ident()?;
                self.expect(Token::Colon)?;
                let ty = self.parse_type()?;
                bindings.push((name, ty));
                if matches!(self.peek(), Some(Token::Comma)) {
                    self.pos += 1;
                } else {
                    break;
                }
            }
        }
        self.expect(Token::RParen)?;
        self.expect(Token::LBrace)?;
        let mut stmts = Vec::new();
        while !matches!(self.peek(), Some(Token::RBrace)) {
            stmts.push(self.parse_stmt()?);
            if matches!(self.peek(), Some(Token::Semi)) {
                self.pos += 1;
            } else if !matches!(self.peek(), Some(Token::RBrace)) {
                return Err("expected ';' or '}' after statement".to_string());
            }
        }
        self.expect(Token::RBrace)?;
        Ok(Block { bindings, stmts })
    }

    fn parse_type(&mut self) -> Result<Type, String> {
        if matches!(self.peek(), Some(Token::LBracket)) {
            self.pos += 1;
            let inner = self.parse_type()?;
            self.expect(Token::Semi)?;
            let len = self.expect_number()?;
            self.expect(Token::RBracket)?;
            return Ok(Type::Array(Box::new(inner), len));
        }

        let first = self.parse_type_atom()?;
        let mut items = vec![first];
        let mut kind = None;
        loop {
            match self.peek() {
                Some(Token::Star) => {
                    self.pos += 1;
                    if matches!(kind, Some("sum")) {
                        return Err("cannot mix '*' and '+' in a type without brackets".to_string());
                    }
                    kind = Some("product");
                    items.push(self.parse_type_atom()?);
                }
                Some(Token::Plus) => {
                    self.pos += 1;
                    if matches!(kind, Some("product")) {
                        return Err("cannot mix '*' and '+' in a type without brackets".to_string());
                    }
                    kind = Some("sum");
                    items.push(self.parse_type_atom()?);
                }
                _ => break,
            }
        }
        Ok(match kind {
            None => items.into_iter().next().unwrap(),
            Some("product") => Type::Product(items),
            Some("sum") => Type::Sum(items),
            Some(_) => unreachable!(),
        })
    }

    fn parse_type_atom(&mut self) -> Result<Type, String> {
        match self.next() {
            Some(Token::TypeNum) => Ok(Type::Num),
            Some(Token::TypeU8) => Ok(Type::U8),
            Some(Token::TypeBool) => Ok(Type::Bool),
            Some(Token::TypeUnit) => Ok(Type::Unit),
            Some(Token::TypePtr) => Ok(Type::Ptr),
            Some(Token::TypeFn) => Ok(Type::Fn),
            other => Err(format!("expected type atom, found {other:?}")),
        }
    }

    fn parse_stmt(&mut self) -> Result<Stmt, String> {
        match self.peek() {
            Some(Token::Assign) => {
                self.pos += 1;
                let place = self.parse_place_expr()?;
                self.expect(Token::ColonEq)?;
                let value = self.parse_value_expr_until(Self::is_stmt_terminator)?;
                Ok(Stmt::Assign { place, value })
            }
            Some(Token::If) => {
                self.pos += 1;
                let cond = self.parse_value_expr_until(Self::is_stmt_start)?;
                let stmt = self.parse_stmt()?;
                Ok(Stmt::If {
                    cond,
                    stmt: Box::new(stmt),
                })
            }
            Some(Token::Case) => {
                self.pos += 1;
                let tag = self.expect_number()?;
                self.expect(Token::Of)?;
                let value = self.parse_value_expr_until(Self::is_stmt_start)?;
                let stmt = self.parse_stmt()?;
                Ok(Stmt::Case {
                    tag,
                    value,
                    stmt: Box::new(stmt),
                })
            }
            Some(Token::HAlloc) => {
                self.pos += 1;
                let ty = self.parse_type()?;
                self.expect(Token::Arrow)?;
                let place = self.parse_place_expr()?;
                Ok(Stmt::HAlloc { ty, place })
            }
            Some(Token::HFree) => {
                self.pos += 1;
                let value = self.parse_value_expr_until(Self::is_stmt_terminator)?;
                Ok(Stmt::HFree { value })
            }
            Some(Token::Loop) => {
                self.pos += 1;
                let label = self.expect_ident()?;
                self.expect(Token::Colon)?;
                let stmt = self.parse_stmt()?;
                Ok(Stmt::Loop {
                    label,
                    stmt: Box::new(stmt),
                })
            }
            Some(Token::Break) => {
                self.pos += 1;
                Ok(Stmt::Break(self.expect_ident()?))
            }
            Some(Token::Continue) => {
                self.pos += 1;
                Ok(Stmt::Continue(self.expect_ident()?))
            }
            Some(Token::Call) => {
                self.pos += 1;
                let callee = self.parse_value_expr_until(|parser| matches!(parser.peek(), Some(Token::LParen)))?;
                self.expect(Token::LParen)?;
                let args = self.parse_value_expr_list(Token::RParen)?;
                self.expect(Token::RParen)?;
                self.expect(Token::Arrow)?;
                let rets = self.parse_place_list()?;
                Ok(Stmt::Call { callee, args, rets })
            }
            Some(Token::Return) => {
                self.pos += 1;
                let values = if Self::is_stmt_terminator(self) {
                    Vec::new()
                } else {
                    self.parse_value_expr_list_until_stmt_end()?
                };
                Ok(Stmt::Return(values))
            }
            Some(Token::Block) => {
                self.pos += 1;
                Ok(Stmt::Block(self.parse_block()?))
            }
            other => Err(format!("unexpected token in statement: {other:?}")),
        }
    }

    fn parse_place_list(&mut self) -> Result<Vec<PlaceExpr>, String> {
        let mut places = Vec::new();
        loop {
            places.push(self.parse_place_expr()?);
            if matches!(self.peek(), Some(Token::Comma)) {
                self.pos += 1;
            } else {
                break;
            }
        }
        Ok(places)
    }

    fn parse_value_expr_list(&mut self, end: Token) -> Result<Vec<ValueExpr>, String> {
        let mut values = Vec::new();
        if matches!(self.peek(), Some(token) if *token == end) {
            return Ok(values);
        }
        loop {
            values.push(self.parse_value_expr_until(|parser| {
                matches!(parser.peek(), Some(Token::Comma))
                    || matches!(parser.peek(), Some(token) if *token == end)
            })?);
            if matches!(self.peek(), Some(Token::Comma)) {
                self.pos += 1;
            } else {
                break;
            }
        }
        Ok(values)
    }

    fn parse_value_expr_list_until_stmt_end(&mut self) -> Result<Vec<ValueExpr>, String> {
        let mut values = Vec::new();
        loop {
            values.push(self.parse_value_expr_until(|parser| {
                matches!(parser.peek(), Some(Token::Comma)) || Self::is_stmt_terminator(parser)
            })?);
            if matches!(self.peek(), Some(Token::Comma)) {
                self.pos += 1;
            } else {
                break;
            }
        }
        Ok(values)
    }

    fn parse_place_expr(&mut self) -> Result<PlaceExpr, String> {
        let mut place = match self.peek() {
            Some(Token::Static) => {
                self.pos += 1;
                PlaceExpr::Static(self.expect_ident()?)
            }
            Some(Token::Local) => {
                self.pos += 1;
                PlaceExpr::Local(self.expect_ident()?)
            }
            Some(Token::LParen) => {
                self.pos += 1;
                let value = self.parse_value_expr_until(|parser| matches!(parser.peek(), Some(Token::RParen)))?;
                self.expect(Token::RParen)?;
                self.expect(Token::HashLoc)?;
                PlaceExpr::Deref(Box::new(value))
            }
            other => return Err(format!("expected place expression, found {other:?}")),
        };

        loop {
            match self.peek() {
                Some(Token::Dot) => {
                    self.pos += 1;
                    let index = self.expect_number()?;
                    place = PlaceExpr::Field(Box::new(place), index);
                }
                Some(Token::Question) => {
                    self.pos += 1;
                    let tag = self.expect_number()?;
                    place = PlaceExpr::Tag(Box::new(place), tag);
                }
                Some(Token::LBracket) => {
                    self.pos += 1;
                    let index = self.parse_value_expr_until(|parser| matches!(parser.peek(), Some(Token::RBracket)))?;
                    self.expect(Token::RBracket)?;
                    place = PlaceExpr::Index(Box::new(place), Box::new(index));
                }
                _ => break,
            }
        }
        Ok(place)
    }

    fn parse_value_expr_until<F>(&mut self, stop: F) -> Result<ValueExpr, String>
    where
        F: Fn(&Parser) -> bool,
    {
        let mut stack = Vec::new();
        while !self.is_eof() && !stop(self) {
            match self.peek().cloned() {
                Some(Token::Number(n)) => {
                    self.pos += 1;
                    stack.push(ValueExpr::Number(n));
                }
                Some(Token::Char(c)) => {
                    self.pos += 1;
                    stack.push(ValueExpr::Char(c));
                }
                Some(Token::True) => {
                    self.pos += 1;
                    stack.push(ValueExpr::Bool(true));
                }
                Some(Token::False) => {
                    self.pos += 1;
                    stack.push(ValueExpr::Bool(false));
                }
                Some(Token::TypeUnit) => {
                    self.pos += 1;
                    stack.push(ValueExpr::Unit);
                }
                Some(Token::NullPtr) => {
                    self.pos += 1;
                    stack.push(ValueExpr::NullPtr);
                }
                Some(Token::NullFn) => {
                    self.pos += 1;
                    stack.push(ValueExpr::NullFn);
                }
                Some(Token::Fn) => {
                    self.pos += 1;
                    stack.push(ValueExpr::Fn(self.expect_ident()?));
                }
                Some(Token::Load) => {
                    self.pos += 1;
                    stack.push(ValueExpr::Load(Box::new(self.parse_place_expr()?)));
                }
                Some(Token::Static) | Some(Token::Local) | Some(Token::LParen) => {
                    let place = self.parse_place_expr()?;
                    self.expect(Token::HashAddr)?;
                    stack.push(ValueExpr::Addr(Box::new(place)));
                }
                Some(Token::Bang) => {
                    self.pos += 1;
                    let expr = stack.pop().ok_or_else(|| "operator '!' needs one operand".to_string())?;
                    stack.push(ValueExpr::UnOp(UnOp::Not, Box::new(expr)));
                }
                Some(Token::Plus) => self.reduce_binop(&mut stack, BinOp::Add)?,
                Some(Token::Minus) => self.reduce_binop(&mut stack, BinOp::Sub)?,
                Some(Token::EqEq) => self.reduce_binop(&mut stack, BinOp::Eq)?,
                Some(Token::Lt) => self.reduce_binop(&mut stack, BinOp::Lt)?,
                Some(Token::AndAnd) => self.reduce_binop(&mut stack, BinOp::And)?,
                Some(Token::OrOr) => self.reduce_binop(&mut stack, BinOp::Or)?,
                Some(Token::Pair) => {
                    self.pos += 1;
                    self.expect(Token::LParen)?;
                    let arity = self.expect_number()?;
                    self.expect(Token::RParen)?;
                    if stack.len() < arity {
                        return Err("pair arity exceeds expression stack".to_string());
                    }
                    let mut items = stack.split_off(stack.len() - arity);
                    stack.push(ValueExpr::Pair(std::mem::take(&mut items)));
                }
                Some(Token::Tag) => {
                    self.pos += 1;
                    self.expect(Token::LParen)?;
                    let tag = self.expect_number()?;
                    self.expect(Token::RParen)?;
                    let value = stack.pop().ok_or_else(|| "tag needs one operand".to_string())?;
                    stack.push(ValueExpr::Tag(tag, Box::new(value)));
                }
                other => return Err(format!("unexpected token in value expression: {other:?}")),
            }
        }

        if stack.len() != 1 {
            return Err("value expression did not reduce to a single value".to_string());
        }
        Ok(stack.pop().unwrap())
    }

    fn reduce_binop(&mut self, stack: &mut Vec<ValueExpr>, op: BinOp) -> Result<(), String> {
        self.pos += 1;
        let rhs = stack.pop().ok_or_else(|| format!("operator '{op:?}' needs two operands"))?;
        let lhs = stack.pop().ok_or_else(|| format!("operator '{op:?}' needs two operands"))?;
        stack.push(ValueExpr::BinOp(Box::new(lhs), op, Box::new(rhs)));
        Ok(())
    }

    fn is_stmt_start(&self) -> bool {
        matches!(
            self.peek(),
            Some(Token::Assign)
                | Some(Token::If)
                | Some(Token::Case)
                | Some(Token::HAlloc)
                | Some(Token::HFree)
                | Some(Token::Loop)
                | Some(Token::Break)
                | Some(Token::Continue)
                | Some(Token::Call)
                | Some(Token::Return)
                | Some(Token::Block)
        )
    }

    fn is_stmt_terminator(&self) -> bool {
        matches!(self.peek(), Some(Token::Semi) | Some(Token::RBrace))
    }

    fn expect_ident(&mut self) -> Result<String, String> {
        match self.next() {
            Some(Token::Ident(name)) => Ok(name),
            other => Err(format!("expected identifier, found {other:?}")),
        }
    }

    fn expect_number(&mut self) -> Result<usize, String> {
        match self.next() {
            Some(Token::Number(n)) => Ok(n),
            other => Err(format!("expected number, found {other:?}")),
        }
    }

    fn expect(&mut self, token: Token) -> Result<(), String> {
        match self.next() {
            Some(next) if next == token => Ok(()),
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

    fn next(&mut self) -> Option<Token> {
        let token = self.tokens.get(self.pos).cloned();
        if token.is_some() {
            self.pos += 1;
        }
        token
    }

    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.pos)
    }

    fn is_eof(&self) -> bool {
        self.pos >= self.tokens.len()
    }
}
