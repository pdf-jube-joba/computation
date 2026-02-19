use turing_machine::machine::Sign;
use utils::TextCodec;
use utils::alphabet::Alphabet;

use super::machine::{Block, Condition, Function, LValue, Program, RValue, Stmt};

const KEYWORDS: [&str; 12] = [
    "alphabet", "fn", "label", "jump", "call", "const", "return", "break", "continue", "if", "LT",
    "RT",
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
            '@' => {
                chars.next();
                tokens.push(Token::Symbol("@".to_string()));
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
        let mut functions: Vec<Function> = Vec::new();
        while !self.is_eof() {
            let function = self.parse_function()?;
            if functions.iter().any(|func| func.name == function.name) {
                return Err(format!("Function '{}' is defined twice", function.name));
            }
            functions.push(function);
        }
        Ok(Program {
            alphabet,
            functions,
        })
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
        if matches!(self.peek(), Some(Token::Symbol(sym)) if sym == "(") {
            self.expect_symbol("(")?;
            self.expect_symbol(")")?;
        }
        self.expect_symbol("{")?;
        let blocks = self.parse_blocks()?;
        self.expect_symbol("}")?;
        Ok(Function { name, blocks })
    }

    fn parse_blocks(&mut self) -> Result<Vec<Block>, String> {
        let mut blocks = Vec::new();
        while !matches!(self.peek(), Some(Token::Symbol(sym)) if sym == "}") {
            if self.is_eof() {
                return Err("Unexpected EOF in block list".to_string());
            }
            blocks.push(self.parse_block()?);
        }
        Ok(blocks)
    }

    fn parse_block(&mut self) -> Result<Block, String> {
        self.expect_keyword("label")?;
        let label = self.expect_ident("label")?;
        self.expect_symbol(":")?;
        self.expect_symbol("{")?;
        let body = self.parse_stmt_list()?;
        self.expect_symbol("}")?;
        Ok(Block { label, body })
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
                "label" => Err("label blocks cannot be nested".to_string()),
                "jump" => {
                    self.next();
                    let label = self.expect_ident("label")?;
                    let cond = self.parse_optional_condition()?;
                    Ok(Stmt::Jump { label, cond })
                }
                "break" => {
                    self.next();
                    let cond = self.parse_optional_condition()?;
                    Ok(Stmt::Break { cond })
                }
                "continue" => {
                    self.next();
                    let cond = self.parse_optional_condition()?;
                    Ok(Stmt::Continue { cond })
                }
                "return" => {
                    self.next();
                    let cond = self.parse_optional_condition()?;
                    Ok(Stmt::Return { cond })
                }
                "call" => {
                    self.next();
                    let name = self.expect_ident("function name")?;
                    if matches!(self.peek(), Some(Token::Symbol(sym)) if sym == "(") {
                        self.expect_symbol("(")?;
                        self.expect_symbol(")")?;
                    }
                    Ok(Stmt::Call { name })
                }
                _ => {
                    let dst = self.parse_lvalue()?;
                    self.expect_symbol(":=")?;
                    let src = self.parse_rvalue()?;
                    Ok(Stmt::Assign { dst, src })
                }
            },
            Token::Symbol(sym) => {
                if sym == "@" {
                    let dst = self.parse_lvalue()?;
                    self.expect_symbol(":=")?;
                    let src = self.parse_rvalue()?;
                    Ok(Stmt::Assign { dst, src })
                } else {
                    Err(format!("Unexpected symbol '{}'", sym))
                }
            }
        }
    }

    fn parse_sign(&mut self) -> Result<Sign, String> {
        match self.next() {
            Some(Token::Ident(ident)) => <Sign as TextCodec>::parse(&ident),
            Some(Token::Symbol(sym)) => Err(format!("Expected sign, got '{}'", sym)),
            None => Err("Expected sign, got EOF".to_string()),
        }
    }

    fn parse_lvalue(&mut self) -> Result<LValue, String> {
        if matches!(self.peek(), Some(Token::Symbol(sym)) if sym == "@") {
            self.expect_symbol("@")?;
            Ok(LValue::Head)
        } else {
            let var = self.expect_ident("variable")?;
            Ok(LValue::Var(var))
        }
    }

    fn parse_rvalue(&mut self) -> Result<RValue, String> {
        if matches!(self.peek(), Some(Token::Symbol(sym)) if sym == "@") {
            self.expect_symbol("@")?;
            return Ok(RValue::Head);
        }
        if matches!(self.peek(), Some(Token::Ident(id)) if id == "const") {
            self.expect_keyword("const")?;
            let value = self.parse_sign()?;
            return Ok(RValue::Const(value));
        }
        let var = self.expect_ident("variable")?;
        Ok(RValue::Var(var))
    }

    fn parse_optional_condition(&mut self) -> Result<Option<Condition>, String> {
        if !matches!(self.peek(), Some(Token::Ident(id)) if id == "if") {
            return Ok(None);
        }
        self.expect_keyword("if")?;
        let left = self.parse_rvalue()?;
        self.expect_symbol("==")?;
        let right = self.parse_rvalue()?;
        Ok(Some(Condition { left, right }))
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
        for (idx, function) in self.functions.iter().enumerate() {
            if idx > 0 {
                writeln!(f)?;
            }
            writeln!(f, "fn {}() {{", function.name)?;
            write_blocks(f, &function.blocks, 1)?;
            writeln!(f, "}}")?;
        }
        Ok(())
    }
}

pub(crate) fn render_text(stmt: &Stmt) -> String {
    let mut out = String::new();
    let _ = write_stmt_line(&mut out, stmt, 0);
    out.trim_end().to_string()
}

fn write_blocks(f: &mut impl std::fmt::Write, blocks: &[Block], indent: usize) -> std::fmt::Result {
    for block in blocks {
        for _ in 0..indent {
            write!(f, "  ")?;
        }
        writeln!(f, "label {}: {{", block.label)?;
        write_stmt_list(f, &block.body, indent + 1)?;
        for _ in 0..indent {
            write!(f, "  ")?;
        }
        writeln!(f, "}}")?;
    }
    Ok(())
}

fn write_stmt_list(
    f: &mut impl std::fmt::Write,
    stmts: &[Stmt],
    indent: usize,
) -> std::fmt::Result {
    for stmt in stmts {
        write_stmt_line(f, stmt, indent)?;
    }
    Ok(())
}

fn write_stmt_line(f: &mut impl std::fmt::Write, stmt: &Stmt, indent: usize) -> std::fmt::Result {
    for _ in 0..indent {
        write!(f, "  ")?;
    }
    match stmt {
        Stmt::Lt => writeln!(f, "LT"),
        Stmt::Rt => writeln!(f, "RT"),
        Stmt::Assign { dst, src } => {
            write!(f, "{} := ", render_lvalue(dst))?;
            writeln!(f, "{}", render_rvalue(src))
        }
        Stmt::Break { cond } => {
            write!(f, "break")?;
            write_condition(f, cond)
        }
        Stmt::Continue { cond } => {
            write!(f, "continue")?;
            write_condition(f, cond)
        }
        Stmt::Jump { label, cond } => {
            write!(f, "jump {}", label)?;
            write_condition(f, cond)
        }
        Stmt::Return { cond } => {
            write!(f, "return")?;
            write_condition(f, cond)
        }
        Stmt::Call { name } => writeln!(f, "call {}", name),
    }
}

fn render_lvalue(value: &LValue) -> String {
    match value {
        LValue::Var(var) => var.clone(),
        LValue::Head => "@".to_string(),
    }
}

fn render_rvalue(value: &RValue) -> String {
    match value {
        RValue::Var(var) => var.clone(),
        RValue::Head => "@".to_string(),
        RValue::Const(sign) => format!("const {}", sign.print()),
    }
}

fn write_condition(
    f: &mut impl std::fmt::Write,
    cond: &Option<Condition>,
) -> std::fmt::Result {
    if let Some(cond) = cond {
        write!(
            f,
            " if {} == {}",
            render_rvalue(&cond.left),
            render_rvalue(&cond.right)
        )?;
    }
    writeln!(f)
}
