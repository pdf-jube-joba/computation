use turing_machine::machine::Sign;
use utils::alphabet::Alphabet;
use utils::TextCodec;

use super::machine::{Program, Stmt};

const KEYWORDS: [&str; 8] = ["alphabet", "jump", "if", "const", "LT", "RT", "READ", "STOR"];

#[derive(Debug, Clone)]
enum Token {
    Ident(String),
    Number(String),
    Symbol(String),
}

pub(crate) fn parse_identifier(name: &str) -> Result<String, String> {
    parse_identifier_with_context(name, "identifier")
}

fn parse_identifier_with_context(name: &str, context: &str) -> Result<String, String> {
    if is_keyword(name) {
        return Err(format!("{} cannot be a keyword: '{}'", context, name));
    }
    Alphabet::new(name)
        .map(|al| al.as_str().to_string())
        .map_err(|e| e.to_string())
}

fn is_keyword(name: &str) -> bool {
    KEYWORDS.iter().any(|kw| *kw == name)
}

fn tokenize(text: &str) -> Result<Vec<Token>, String> {
    let mut tokens = Vec::new();
    let mut chars = text.chars().peekable();
    while let Some(&ch) = chars.peek() {
        if ch.is_whitespace() {
            chars.next();
            continue;
        }
        if ch == '-' {
            chars.next();
            tokens.push(Token::Ident("-".to_string()));
            continue;
        }
        if ch.is_ascii_digit() {
            let mut digits = String::new();
            while let Some(&next) = chars.peek() {
                if next.is_ascii_digit() {
                    digits.push(next);
                    chars.next();
                } else {
                    break;
                }
            }
            tokens.push(Token::Number(digits));
            continue;
        }
        if ch.is_ascii_alphabetic() || ch == '_' {
            let mut ident = String::new();
            ident.push(ch);
            chars.next();
            while let Some(&next) = chars.peek() {
                if next.is_ascii_alphanumeric() || next == '_' || next == '-' {
                    ident.push(next);
                    chars.next();
                } else {
                    break;
                }
            }
            tokens.push(Token::Ident(ident));
            continue;
        }
        match ch {
            '(' | ')' | '{' | '}' | ',' => {
                chars.next();
                tokens.push(Token::Symbol(ch.to_string()));
            }
            ':' => {
                chars.next();
                if let Some('=') = chars.peek() {
                    chars.next();
                    tokens.push(Token::Symbol(":=".to_string()));
                } else {
                    tokens.push(Token::Symbol(":".to_string()));
                }
            }
            '=' => {
                chars.next();
                if let Some('=') = chars.peek() {
                    chars.next();
                    tokens.push(Token::Symbol("==".to_string()));
                } else {
                    return Err("Unexpected '='".to_string());
                }
            }
            _ => {
                return Err(format!("Unexpected character '{}'", ch));
            }
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
        Parser { tokens, pos: 0 }
    }

    fn is_eof(&self) -> bool {
        self.pos >= self.tokens.len()
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

    fn expect_ident(&mut self, context: &str) -> Result<String, String> {
        match self.next() {
            Some(Token::Ident(ident)) => parse_identifier_with_context(&ident, context),
            Some(Token::Number(num)) => Err(format!("Expected identifier for {}, got {}", context, num)),
            Some(Token::Symbol(sym)) => Err(format!("Expected identifier for {}, got {}", context, sym)),
            None => Err(format!("Expected identifier for {}, got EOF", context)),
        }
    }

    fn expect_number(&mut self, context: &str) -> Result<usize, String> {
        match self.next() {
            Some(Token::Number(num)) => num
                .parse::<usize>()
                .map_err(|e| format!("Invalid number for {}: {}", context, e)),
            Some(Token::Ident(ident)) => Err(format!("Expected number for {}, got {}", context, ident)),
            Some(Token::Symbol(sym)) => Err(format!("Expected number for {}, got {}", context, sym)),
            None => Err(format!("Expected number for {}, got EOF", context)),
        }
    }

    fn expect_keyword(&mut self, keyword: &str) -> Result<(), String> {
        match self.next() {
            Some(Token::Ident(ident)) if ident == keyword => Ok(()),
            Some(Token::Ident(ident)) => Err(format!("Expected '{}', got '{}'", keyword, ident)),
            Some(Token::Number(num)) => Err(format!("Expected '{}', got '{}'", keyword, num)),
            Some(Token::Symbol(sym)) => Err(format!("Expected '{}', got '{}'", keyword, sym)),
            None => Err(format!("Expected '{}', got EOF", keyword)),
        }
    }

    fn expect_symbol(&mut self, symbol: &str) -> Result<(), String> {
        match self.next() {
            Some(Token::Symbol(sym)) if sym == symbol => Ok(()),
            Some(Token::Symbol(sym)) => Err(format!("Expected '{}', got '{}'", symbol, sym)),
            Some(Token::Ident(ident)) => Err(format!("Expected '{}', got '{}'", symbol, ident)),
            Some(Token::Number(num)) => Err(format!("Expected '{}', got '{}'", symbol, num)),
            None => Err(format!("Expected '{}', got EOF", symbol)),
        }
    }

    fn parse_program(&mut self) -> Result<Program, String> {
        let alphabet = self.parse_alphabet()?;
        let mut body = Vec::new();
        while !self.is_eof() {
            body.push(self.parse_stmt()?);
        }
        Ok(Program { alphabet, body })
    }

    fn parse_alphabet(&mut self) -> Result<Vec<Sign>, String> {
        self.expect_keyword("alphabet")?;
        self.expect_symbol(":")?;
        self.expect_symbol("(")?;
        let mut signs = Vec::new();
        if matches!(self.peek(), Some(Token::Symbol(sym)) if sym == ")") {
            self.next();
            return Ok(signs);
        }
        loop {
            let sign = self.parse_sign()?;
            signs.push(sign);
            match self.peek() {
                Some(Token::Symbol(sym)) if sym == "," => {
                    self.next();
                    if matches!(self.peek(), Some(Token::Symbol(sym)) if sym == ")") {
                        self.next();
                        break;
                    }
                }
                Some(Token::Symbol(sym)) if sym == ")" => {
                    self.next();
                    break;
                }
                _ => return Err("Expected ',' or ')' in alphabet".to_string()),
            }
        }
        Ok(signs)
    }

    fn parse_stmt(&mut self) -> Result<Stmt, String> {
        let token = self.peek().cloned().ok_or_else(|| "Unexpected EOF".to_string())?;
        match token {
            Token::Ident(ident) => match ident.as_str() {
                "LT" => {
                    self.next();
                    Ok(Stmt::Lt)
                }
                "RT" => {
                    self.next();
                    Ok(Stmt::Rt)
                }
                "READ" => {
                    self.next();
                    let var = self.expect_ident("variable")?;
                    Ok(Stmt::Read(var))
                }
                "STOR" => {
                    self.next();
                    if matches!(self.peek(), Some(Token::Ident(id)) if id == "const") {
                        self.expect_keyword("const")?;
                        let value = self.parse_sign()?;
                        Ok(Stmt::StorConst(value))
                    } else {
                        let var = self.expect_ident("variable")?;
                        Ok(Stmt::Stor(var))
                    }
                }
                "jump" => {
                    self.next();
                    if matches!(self.peek(), Some(Token::Ident(id)) if id == "if") {
                        self.expect_keyword("if")?;
                        let var = self.expect_ident("variable")?;
                        self.expect_symbol("==")?;
                        let value = self.parse_sign()?;
                        let target = self.expect_number("jump target")?;
                        Ok(Stmt::JumpIf { var, value, target })
                    } else {
                        let target = self.expect_number("jump target")?;
                        Ok(Stmt::Jump(target))
                    }
                }
                _ => {
                    let var = self.expect_ident("variable")?;
                    self.expect_symbol(":=")?;
                    if matches!(self.peek(), Some(Token::Ident(id)) if id == "const") {
                        self.expect_keyword("const")?;
                        let value = self.parse_sign()?;
                        Ok(Stmt::ConstAssign(var, value))
                    } else {
                        let src = self.expect_ident("variable")?;
                        Ok(Stmt::Assign(var, src))
                    }
                }
            },
            Token::Number(num) => Err(format!("Unexpected number '{}'", num)),
            Token::Symbol(sym) => Err(format!("Unexpected symbol '{}'", sym)),
        }
    }

    fn parse_sign(&mut self) -> Result<Sign, String> {
        match self.next() {
            Some(Token::Ident(ident)) => <Sign as TextCodec>::parse(&ident),
            Some(Token::Number(num)) => Err(format!("Expected sign, got number {}", num)),
            Some(Token::Symbol(sym)) => Err(format!("Expected sign, got '{}'", sym)),
            None => Err("Expected sign, got EOF".to_string()),
        }
    }
}

fn parse_program(text: &str) -> Result<Program, String> {
    let tokens = tokenize(text)?;
    let mut parser = Parser::new(tokens);
    let program = parser.parse_program()?;
    if !parser.is_eof() {
        return Err("Unexpected trailing tokens".to_string());
    }
    Ok(program)
}

impl TextCodec for Program {
    fn parse(text: &str) -> Result<Self, String> {
        parse_program(text)
    }

    fn write_fmt(&self, f: &mut impl std::fmt::Write) -> std::fmt::Result {
        write!(f, "alphabet: ")?;
        self.alphabet.write_fmt(f)?;
        writeln!(f)?;
        for stmt in &self.body {
            match stmt {
                Stmt::Lt => writeln!(f, "LT")?,
                Stmt::Rt => writeln!(f, "RT")?,
            Stmt::Read(var) => writeln!(f, "READ {}", var)?,
            Stmt::Stor(var) => writeln!(f, "STOR {}", var)?,
            Stmt::StorConst(value) => writeln!(f, "STOR const {}", value.print())?,
            Stmt::Assign(dst, src) => writeln!(f, "{} := {}", dst, src)?,
            Stmt::ConstAssign(dst, value) => {
                writeln!(f, "{} := const {}", dst, value.print())?;
            }
            Stmt::Jump(target) => writeln!(f, "jump {}", target)?,
            Stmt::JumpIf { var, value, target } => {
                writeln!(f, "jump if {} == {} {}", var, value.print(), target)?
            }
        }
        }
        Ok(())
    }
}
