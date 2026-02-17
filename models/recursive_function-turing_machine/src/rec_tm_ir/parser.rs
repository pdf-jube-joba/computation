use std::collections::HashMap;

use turing_machine::machine::Sign;
use utils::TextCodec;
use utils::alphabet::Alphabet;

use super::machine::{Function, Program, Stmt};

const KEYWORDS: [&str; 10] = [
    "alphabet", "fn", "loop", "if", "break", "call", "LT", "RT", "READ", "STOR",
];

#[derive(Debug, Clone)]
enum Token {
    Ident(String),
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
    KEYWORDS.contains(&name)
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
            Some(Token::Symbol(sym)) => {
                Err(format!("Expected identifier for {}, got {}", context, sym))
            }
            None => Err(format!("Expected identifier for {}, got EOF", context)),
        }
    }

    fn expect_keyword(&mut self, keyword: &str) -> Result<(), String> {
        match self.next() {
            Some(Token::Ident(ident)) if ident == keyword => Ok(()),
            Some(Token::Ident(ident)) => Err(format!("Expected '{}', got '{}'", keyword, ident)),
            Some(Token::Symbol(sym)) => Err(format!("Expected '{}', got '{}'", keyword, sym)),
            None => Err(format!("Expected '{}', got EOF", keyword)),
        }
    }

    fn expect_symbol(&mut self, symbol: &str) -> Result<(), String> {
        match self.next() {
            Some(Token::Symbol(sym)) if sym == symbol => Ok(()),
            Some(Token::Symbol(sym)) => Err(format!("Expected '{}', got '{}'", symbol, sym)),
            Some(Token::Ident(ident)) => Err(format!("Expected '{}', got '{}'", symbol, ident)),
            None => Err(format!("Expected '{}', got EOF", symbol)),
        }
    }

    fn parse_program(&mut self) -> Result<Program, String> {
        let alphabet = self.parse_alphabet()?;
        let mut functions = HashMap::new();
        while !self.is_eof() {
            let function = self.parse_function()?;
            if functions.contains_key(&function.name) {
                return Err(format!("Function '{}' is defined twice", function.name));
            }
            functions.insert(function.name.clone(), function);
        }
        Ok(Program { alphabet, functions })
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

    fn parse_function(&mut self) -> Result<Function, String> {
        self.expect_keyword("fn")?;
        let name = self.expect_ident("function name")?;
        self.expect_symbol("(")?;
        let params = self.parse_ident_list("parameter")?;
        self.expect_symbol(")")?;
        self.expect_symbol("{")?;
        let body = self.parse_stmt_list()?;
        self.expect_symbol("}")?;
        Ok(Function { name, params, body })
    }

    fn parse_ident_list(&mut self, context: &str) -> Result<Vec<String>, String> {
        let mut items = Vec::new();
        if matches!(self.peek(), Some(Token::Symbol(sym)) if sym == ")") {
            return Ok(items);
        }
        loop {
            let ident = self.expect_ident(context)?;
            items.push(ident);
            match self.peek() {
                Some(Token::Symbol(sym)) if sym == "," => {
                    self.next();
                    if matches!(self.peek(), Some(Token::Symbol(sym)) if sym == ")") {
                        break;
                    }
                }
                _ => break,
            }
        }
        Ok(items)
    }

    fn parse_stmt_list(&mut self) -> Result<Vec<Stmt>, String> {
        let mut stmts = Vec::new();
        while !matches!(self.peek(), Some(Token::Symbol(sym)) if sym == "}") {
            if self.is_eof() {
                return Err("Unexpected EOF in statement list".to_string());
            }
            stmts.push(self.parse_stmt()?);
        }
        Ok(stmts)
    }

    fn parse_stmt(&mut self) -> Result<Stmt, String> {
        let token = self
            .peek()
            .cloned()
            .ok_or_else(|| "Unexpected EOF".to_string())?;
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
                    let var = self.expect_ident("variable")?;
                    Ok(Stmt::Stor(var))
                }
                "if" => {
                    self.next();
                    let var = self.expect_ident("variable")?;
                    self.expect_symbol("==")?;
                    let value = self.parse_sign()?;
                    self.expect_keyword("break")?;
                    let label = self.expect_ident("label")?;
                    Ok(Stmt::IfBreak { var, value, label })
                }
                "loop" => {
                    self.next();
                    let label = self.expect_ident("label")?;
                    self.expect_symbol(":")?;
                    self.expect_symbol("{")?;
                    let body = self.parse_stmt_list()?;
                    self.expect_symbol("}")?;
                    Ok(Stmt::Loop { label, body })
                }
                "call" => {
                    self.next();
                    let name = self.expect_ident("function name")?;
                    self.expect_symbol("(")?;
                    let args = self.parse_ident_list("argument")?;
                    self.expect_symbol(")")?;
                    Ok(Stmt::Call { name, args })
                }
                _ => {
                    let var = self.expect_ident("variable")?;
                    self.expect_symbol(":=")?;
                    let src = self.expect_ident("variable")?;
                    Ok(Stmt::Assign(var, src))
                }
            },
            Token::Symbol(sym) => Err(format!("Unexpected symbol '{}'", sym)),
        }
    }

    fn parse_sign(&mut self) -> Result<Sign, String> {
        match self.next() {
            Some(Token::Ident(ident)) => <Sign as TextCodec>::parse(&ident),
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
        for (idx, function) in self.functions.values().enumerate() {
            if idx > 0 {
                writeln!(f)?;
            }
            write!(f, "fn {}(", function.name)?;
            for (i, param) in function.params.iter().enumerate() {
                if i > 0 {
                    write!(f, ", ")?;
                }
                write!(f, "{}", param)?;
            }
            writeln!(f, ") {{")?;
            write_stmt_list(f, &function.body, 1)?;
            writeln!(f, "}}")?;
        }
        Ok(())
    }
}

fn write_stmt_list(
    f: &mut impl std::fmt::Write,
    stmts: &[Stmt],
    indent: usize,
) -> std::fmt::Result {
    for stmt in stmts {
        for _ in 0..indent {
            write!(f, "  ")?;
        }
        match stmt {
            Stmt::Lt => writeln!(f, "LT")?,
            Stmt::Rt => writeln!(f, "RT")?,
            Stmt::Read(var) => writeln!(f, "READ {}", var)?,
            Stmt::Stor(var) => writeln!(f, "STOR {}", var)?,
            Stmt::Assign(dst, src) => writeln!(f, "{} := {}", dst, src)?,
            Stmt::IfBreak { var, value, label } => {
                writeln!(f, "if {} == {} break {}", var, value.print(), label)?
            }
            Stmt::Loop { label, body } => {
                writeln!(f, "loop {}: {{", label)?;
                write_stmt_list(f, body, indent + 1)?;
                for _ in 0..indent {
                    write!(f, "  ")?;
                }
                writeln!(f, "}}")?;
            }
            Stmt::Call { name, args } => {
                write!(f, "call {}(", name)?;
                for (i, arg) in args.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", arg)?;
                }
                writeln!(f, ")")?;
            }
        }
    }
    Ok(())
}
