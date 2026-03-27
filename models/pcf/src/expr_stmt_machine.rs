use serde::{Deserialize, Serialize};
use utils::{Machine, StepResult, TextCodec, identifier::Identifier};

use crate::syntax::{BinOp, PrintEffect, UnOp};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExprStmtMachine {
    pub code: ExprStmtCode,
    pub state: State,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExprStmtCode(pub Expr);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Expr {
    Nat(usize),
    Bool(bool),
    Unit,
    Var(Identifier),
    BinOp(Box<Expr>, BinOp, Box<Expr>),
    UnOp(UnOp, Box<Expr>),
    Fun {
        param: Identifier,
        body: Box<Expr>,
    },
    Rec {
        name: Identifier,
        param: Identifier,
        body: Box<Expr>,
    },
    App(Box<Expr>, Box<Expr>),
    If {
        cond: Box<Expr>,
        then_branch: Box<Expr>,
        else_branch: Box<Expr>,
    },
    Block {
        stmt: Box<Stmt>,
        expr: Box<Expr>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Stmt {
    Skip,
    Print(usize),
    Seq(Box<Stmt>, Box<Stmt>),
    Let {
        name: Identifier,
        expr: Box<Expr>,
    },
    If {
        cond: Box<Expr>,
        then_branch: Box<Stmt>,
    },
    While {
        cond: Box<Expr>,
        body: Box<Stmt>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum State {
    EvalExpr {
        control: Expr,
        env: Env,
        kont: Vec<Frame>,
    },
    Return {
        value: Value,
        kont: Vec<Frame>,
    },
    EvalStmt {
        control: Stmt,
        env: Env,
        kont: Vec<Frame>,
    },
    Done {
        env: Env,
        kont: Vec<Frame>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Frame {
    BinOpLeft {
        op: BinOp,
        rhs: Expr,
        env: Env,
    },
    BinOpRight {
        lhs: Value,
        op: BinOp,
    },
    UnOp {
        op: UnOp,
    },
    AppFun {
        arg: Expr,
        env: Env,
    },
    AppArg {
        fun: Value,
    },
    IfExpr {
        then_branch: Expr,
        else_branch: Expr,
        env: Env,
    },
    Let {
        name: Identifier,
        env: Env,
    },
    Seq {
        next: Stmt,
    },
    IfStmt {
        then_branch: Stmt,
        env: Env,
    },
    While {
        cond: Expr,
        body: Stmt,
        env: Env,
    },
    ScopeOut {
        expr: Expr,
    },
    Init,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Value {
    Nat(usize),
    Bool(bool),
    Unit,
    Fun {
        param: Identifier,
        body: Expr,
        env: Env,
    },
    Rec {
        name: Identifier,
        param: Identifier,
        body: Expr,
        env: Env,
    },
}

pub type Env = Vec<(Identifier, Value)>;

impl Machine for ExprStmtMachine {
    type Code = ExprStmtCode;
    type AInput = Vec<usize>;
    type FOutput = usize;
    type SnapShot = ExprStmtMachine;
    type RInput = ();
    type ROutput = PrintEffect;

    fn make(code: Self::Code, ainput: Self::AInput) -> Result<Self, String> {
        Ok(Self {
            state: State::EvalExpr {
                control: code.0.clone().apply_inputs(&ainput),
                env: Vec::new(),
                kont: vec![Frame::Init],
            },
            code,
        })
    }

    fn step(self, _rinput: Self::RInput) -> Result<StepResult<Self>, String> {
        if let State::Return { value, kont } = &self.state
            && matches!(kont.as_slice(), [Frame::Init])
        {
            return Ok(StepResult::Halt {
                output: value.as_nat()?,
            });
        }

        let (state, output) = step_state(self.state)?;
        Ok(StepResult::Continue {
            next: Self {
                code: self.code,
                state,
            },
            output,
        })
    }

    fn snapshot(&self) -> Self::SnapShot {
        self.clone()
    }

    fn restore(snapshot: Self::SnapShot) -> Self {
        snapshot
    }

    fn render(snapshot: Self::SnapShot) -> utils::RenderState {
        let text = serde_json::to_string_pretty(&snapshot)
            .unwrap_or_else(|_| "failed to serialize snapshot".to_string());
        utils::render_state![utils::render_text!(text, title: "expr-stmt-cek")]
    }
}

fn step_state(state: State) -> Result<(State, PrintEffect), String> {
    match state {
        State::EvalExpr {
            control,
            env,
            mut kont,
        } => match control {
            Expr::Nat(value) => Ok((
                State::Return {
                    value: Value::Nat(value),
                    kont,
                },
                PrintEffect(None),
            )),
            Expr::Bool(value) => Ok((
                State::Return {
                    value: Value::Bool(value),
                    kont,
                },
                PrintEffect(None),
            )),
            Expr::Unit => Ok((
                State::Return {
                    value: Value::Unit,
                    kont,
                },
                PrintEffect(None),
            )),
            Expr::Var(name) => Ok((
                State::Return {
                    value: lookup_env(&env, &name)?,
                    kont,
                },
                PrintEffect(None),
            )),
            Expr::BinOp(lhs, op, rhs) => {
                kont.push(Frame::BinOpLeft {
                    op,
                    rhs: *rhs,
                    env: env.clone(),
                });
                Ok((
                    State::EvalExpr {
                        control: *lhs,
                        env,
                        kont,
                    },
                    PrintEffect(None),
                ))
            }
            Expr::UnOp(op, inner) => {
                kont.push(Frame::UnOp { op });
                Ok((
                    State::EvalExpr {
                        control: *inner,
                        env,
                        kont,
                    },
                    PrintEffect(None),
                ))
            }
            Expr::Fun { param, body } => Ok((
                State::Return {
                    value: Value::Fun {
                        param,
                        body: *body,
                        env,
                    },
                    kont,
                },
                PrintEffect(None),
            )),
            Expr::Rec { name, param, body } => Ok((
                State::Return {
                    value: Value::Rec {
                        name,
                        param,
                        body: *body,
                        env,
                    },
                    kont,
                },
                PrintEffect(None),
            )),
            Expr::App(fun, arg) => {
                kont.push(Frame::AppFun {
                    arg: *arg,
                    env: env.clone(),
                });
                Ok((
                    State::EvalExpr {
                        control: *fun,
                        env,
                        kont,
                    },
                    PrintEffect(None),
                ))
            }
            Expr::If {
                cond,
                then_branch,
                else_branch,
            } => {
                kont.push(Frame::IfExpr {
                    then_branch: *then_branch,
                    else_branch: *else_branch,
                    env: env.clone(),
                });
                Ok((
                    State::EvalExpr {
                        control: *cond,
                        env,
                        kont,
                    },
                    PrintEffect(None),
                ))
            }
            Expr::Block { stmt, expr } => {
                kont.push(Frame::ScopeOut { expr: *expr });
                Ok((
                    State::EvalStmt {
                        control: *stmt,
                        env,
                        kont,
                    },
                    PrintEffect(None),
                ))
            }
        },
        State::Return { value, mut kont } => {
            let frame = kont.pop().ok_or_else(|| "empty continuation".to_string())?;
            match frame {
                Frame::Init => {
                    kont.push(Frame::Init);
                    Ok((State::Return { value, kont }, PrintEffect(None)))
                }
                Frame::BinOpLeft { op, rhs, env } => {
                    kont.push(Frame::BinOpRight { lhs: value, op });
                    Ok((
                        State::EvalExpr {
                            control: rhs,
                            env,
                            kont,
                        },
                        PrintEffect(None),
                    ))
                }
                Frame::BinOpRight { lhs, op } => Ok((
                    State::Return {
                        value: eval_binop_value(lhs, op, value)?,
                        kont,
                    },
                    PrintEffect(None),
                )),
                Frame::UnOp { op } => Ok((
                    State::Return {
                        value: eval_unop_value(op, value)?,
                        kont,
                    },
                    PrintEffect(None),
                )),
                Frame::AppFun { arg, env } => {
                    kont.push(Frame::AppArg { fun: value });
                    Ok((
                        State::EvalExpr {
                            control: arg,
                            env,
                            kont,
                        },
                        PrintEffect(None),
                    ))
                }
                Frame::AppArg { fun } => match fun {
                    Value::Fun {
                        param,
                        body,
                        mut env,
                    } => {
                        env.push((param, value));
                        Ok((
                            State::EvalExpr {
                                control: body,
                                env,
                                kont,
                            },
                            PrintEffect(None),
                        ))
                    }
                    Value::Rec {
                        name,
                        param,
                        body,
                        env,
                    } => {
                        let mut next_env = env.clone();
                        next_env.push((
                            name.clone(),
                            Value::Rec {
                                name,
                                param: param.clone(),
                                body: body.clone(),
                                env: env.clone(),
                            },
                        ));
                        next_env.push((param, value));
                        Ok((
                            State::EvalExpr {
                                control: body,
                                env: next_env,
                                kont,
                            },
                            PrintEffect(None),
                        ))
                    }
                    other => Err(format!("application expects a closure, found {other:?}")),
                },
                Frame::IfExpr {
                    then_branch,
                    else_branch,
                    env,
                } => match value {
                    Value::Bool(true) => Ok((
                        State::EvalExpr {
                            control: then_branch,
                            env,
                            kont,
                        },
                        PrintEffect(None),
                    )),
                    Value::Bool(false) => Ok((
                        State::EvalExpr {
                            control: else_branch,
                            env,
                            kont,
                        },
                        PrintEffect(None),
                    )),
                    other => Err(format!("if expects a boolean condition, found {other:?}")),
                },
                Frame::Let { name, mut env } => {
                    env.push((name, value));
                    Ok((State::Done { env, kont }, PrintEffect(None)))
                }
                Frame::IfStmt { then_branch, env } => match value {
                    Value::Bool(true) => Ok((
                        State::EvalStmt {
                            control: then_branch,
                            env,
                            kont,
                        },
                        PrintEffect(None),
                    )),
                    Value::Bool(false) => Ok((State::Done { env, kont }, PrintEffect(None))),
                    other => Err(format!("if expects a boolean condition, found {other:?}")),
                },
                Frame::While { cond, body, env } => match value {
                    Value::Bool(true) => {
                        kont.push(Frame::Seq {
                            next: Stmt::While {
                                cond: Box::new(cond),
                                body: Box::new(body.clone()),
                            },
                        });
                        Ok((
                            State::EvalStmt {
                                control: body,
                                env,
                                kont,
                            },
                            PrintEffect(None),
                        ))
                    }
                    Value::Bool(false) => Ok((State::Done { env, kont }, PrintEffect(None))),
                    other => Err(format!("while expects a boolean condition, found {other:?}")),
                },
                Frame::Seq { .. } | Frame::ScopeOut { .. } => {
                    Err("internal CEK error: statement frame during return".to_string())
                }
            }
        }
        State::EvalStmt {
            control,
            env,
            mut kont,
        } => match control {
            Stmt::Skip => Ok((State::Done { env, kont }, PrintEffect(None))),
            Stmt::Print(value) => Ok((State::Done { env, kont }, PrintEffect(Some(value)))),
            Stmt::Seq(first, second) => {
                kont.push(Frame::Seq { next: *second });
                Ok((
                    State::EvalStmt {
                        control: *first,
                        env,
                        kont,
                    },
                    PrintEffect(None),
                ))
            }
            Stmt::Let { name, expr } => {
                kont.push(Frame::Let {
                    name,
                    env: env.clone(),
                });
                Ok((
                    State::EvalExpr {
                        control: *expr,
                        env,
                        kont,
                    },
                    PrintEffect(None),
                ))
            }
            Stmt::If { cond, then_branch } => {
                kont.push(Frame::IfStmt {
                    then_branch: *then_branch,
                    env: env.clone(),
                });
                Ok((
                    State::EvalExpr {
                        control: *cond,
                        env,
                        kont,
                    },
                    PrintEffect(None),
                ))
            }
            Stmt::While { cond, body } => {
                kont.push(Frame::While {
                    cond: (*cond).clone(),
                    body: (*body).clone(),
                    env: env.clone(),
                });
                Ok((
                    State::EvalExpr {
                        control: *cond,
                        env,
                        kont,
                    },
                    PrintEffect(None),
                ))
            }
        },
        State::Done { env, mut kont } => {
            let frame = kont.pop().ok_or_else(|| "empty continuation".to_string())?;
            match frame {
                Frame::Seq { next } => Ok((
                    State::EvalStmt {
                        control: next,
                        env,
                        kont,
                    },
                    PrintEffect(None),
                )),
                Frame::ScopeOut { expr } => Ok((
                    State::EvalExpr {
                        control: expr,
                        env,
                        kont,
                    },
                    PrintEffect(None),
                )),
                Frame::Init => {
                    kont.push(Frame::Init);
                    Ok((State::Done { env, kont }, PrintEffect(None)))
                }
                other => Err(format!(
                    "internal CEK error: unexpected frame after statement completion: {other:?}"
                )),
            }
        }
    }
}

fn lookup_env(env: &Env, name: &Identifier) -> Result<Value, String> {
    env.iter()
        .rev()
        .find_map(|(key, value)| if key == name { Some(value.clone()) } else { None })
        .ok_or_else(|| format!("unbound variable: {}", name.as_str()))
}

fn eval_binop_value(lhs: Value, op: BinOp, rhs: Value) -> Result<Value, String> {
    match op {
        BinOp::Add => match (lhs, rhs) {
            (Value::Nat(left), Value::Nat(right)) => Ok(Value::Nat(left + right)),
            _ => Err("operator '+' expects naturals".to_string()),
        },
        BinOp::Sub => match (lhs, rhs) {
            (Value::Nat(left), Value::Nat(right)) => left
                .checked_sub(right)
                .map(Value::Nat)
                .ok_or_else(|| "operator '-' expects lhs >= rhs over naturals".to_string()),
            _ => Err("operator '-' expects naturals".to_string()),
        },
        BinOp::And => match (lhs, rhs) {
            (Value::Bool(left), Value::Bool(right)) => Ok(Value::Bool(left && right)),
            _ => Err("operator '&&' expects booleans".to_string()),
        },
    }
}

fn eval_unop_value(op: UnOp, value: Value) -> Result<Value, String> {
    match op {
        UnOp::Inc => match value {
            Value::Nat(number) => Ok(Value::Nat(number + 1)),
            _ => Err("operator 'inc' expects a natural".to_string()),
        },
        UnOp::Dec => match value {
            Value::Nat(number) => number
                .checked_sub(1)
                .map(Value::Nat)
                .ok_or_else(|| "operator 'dec' expects a positive natural".to_string()),
            _ => Err("operator 'dec' expects a natural".to_string()),
        },
        UnOp::Not => match value {
            Value::Bool(flag) => Ok(Value::Bool(!flag)),
            _ => Err("operator 'not' expects a boolean".to_string()),
        },
    }
}

impl Value {
    fn as_nat(&self) -> Result<usize, String> {
        match self {
            Value::Nat(value) => Ok(*value),
            _ => Err("final value is not a natural number".to_string()),
        }
    }
}

impl Expr {
    fn apply_inputs(self, inputs: &[usize]) -> Self {
        let mut expr = self;
        for input in inputs {
            expr = Expr::App(Box::new(expr), Box::new(Expr::Nat(*input)));
        }
        expr
    }
}

impl TextCodec for ExprStmtCode {
    fn parse(text: &str) -> Result<Self, String> {
        let tokens = lex(text)?;
        let mut parser = Parser::new(tokens);
        let expr = parser.parse_expr()?;
        parser.expect_eof()?;
        Ok(Self(expr))
    }

    fn write_fmt(&self, f: &mut impl std::fmt::Write) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::fmt::Display for Expr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Expr::Nat(value) => write!(f, "{value}"),
            Expr::Bool(true) => write!(f, "#true"),
            Expr::Bool(false) => write!(f, "#false"),
            Expr::Unit => write!(f, "#unit"),
            Expr::Var(name) => write!(f, "{name}"),
            Expr::BinOp(lhs, op, rhs) => write!(f, "({lhs} {op} {rhs})"),
            Expr::UnOp(op, expr) => write!(f, "({op} {expr})"),
            Expr::Fun { param, body } => write!(f, "(fun {param} => {body})"),
            Expr::Rec { name, param, body } => write!(f, "(rec {name} {param} => {body})"),
            Expr::App(fun, arg) => write!(f, "{fun}({arg})"),
            Expr::If {
                cond,
                then_branch,
                else_branch,
            } => write!(f, "(if {cond} then {then_branch} else {else_branch} fi)"),
            Expr::Block { stmt, expr } => write!(f, "{{ {stmt}; {expr} }}"),
        }
    }
}

impl std::fmt::Display for Stmt {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Stmt::Skip => write!(f, "skip"),
            Stmt::Print(value) => write!(f, "print {value}"),
            Stmt::Seq(first, second) => write!(f, "{first}; {second}"),
            Stmt::Let { name, expr } => write!(f, "let {name} := {expr}"),
            Stmt::If { cond, then_branch } => write!(f, "if {cond} then {then_branch} end"),
            Stmt::While { cond, body } => write!(f, "while {cond} do {body} end"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Token {
    Number(usize),
    Ident(String),
    True,
    False,
    Unit,
    Skip,
    Print,
    Fun,
    Rec,
    Let,
    If,
    Then,
    Else,
    Fi,
    While,
    Do,
    End,
    Inc,
    Dec,
    Not,
    Plus,
    Minus,
    AndAnd,
    Arrow,
    ColonEq,
    LParen,
    RParen,
    LBrace,
    RBrace,
    Semi,
}

fn lex(text: &str) -> Result<Vec<Token>, String> {
    let chars: Vec<char> = text.chars().collect();
    let mut tokens = Vec::new();
    let mut pos = 0usize;

    while pos < chars.len() {
        let ch = chars[pos];
        if ch.is_whitespace() {
            pos += 1;
            continue;
        }

        if ch.is_ascii_digit() {
            let start = pos;
            pos += 1;
            while pos < chars.len() && chars[pos].is_ascii_digit() {
                pos += 1;
            }
            tokens.push(Token::Number(
                text[start..pos]
                    .parse::<usize>()
                    .map_err(|e| e.to_string())?,
            ));
            continue;
        }

        if ch == '#' {
            let start = pos;
            pos += 1;
            while pos < chars.len() && (chars[pos].is_ascii_alphanumeric() || chars[pos] == '_') {
                pos += 1;
            }
            match &text[start..pos] {
                "#true" => tokens.push(Token::True),
                "#false" => tokens.push(Token::False),
                "#unit" => tokens.push(Token::Unit),
                other => return Err(format!("unknown token: {other}")),
            }
            continue;
        }

        if ch.is_ascii_alphabetic() || ch == '_' {
            let start = pos;
            pos += 1;
            while pos < chars.len()
                && (chars[pos].is_ascii_alphanumeric() || chars[pos] == '_' || chars[pos] == '-')
            {
                pos += 1;
            }
            let token = match &text[start..pos] {
                "skip" => Token::Skip,
                "print" => Token::Print,
                "fun" => Token::Fun,
                "rec" => Token::Rec,
                "let" => Token::Let,
                "if" => Token::If,
                "then" => Token::Then,
                "else" => Token::Else,
                "fi" => Token::Fi,
                "while" => Token::While,
                "do" => Token::Do,
                "end" => Token::End,
                "inc" => Token::Inc,
                "dec" => Token::Dec,
                "not" => Token::Not,
                word => Token::Ident(word.to_string()),
            };
            tokens.push(token);
            continue;
        }

        match ch {
            '+' => {
                tokens.push(Token::Plus);
                pos += 1;
            }
            '-' => {
                tokens.push(Token::Minus);
                pos += 1;
            }
            '&' => {
                if chars.get(pos + 1) == Some(&'&') {
                    tokens.push(Token::AndAnd);
                    pos += 2;
                } else {
                    return Err("expected '&&'".to_string());
                }
            }
            '=' => {
                if chars.get(pos + 1) == Some(&'>') {
                    tokens.push(Token::Arrow);
                    pos += 2;
                } else {
                    return Err("expected '=>'".to_string());
                }
            }
            ':' => {
                if chars.get(pos + 1) == Some(&'=') {
                    tokens.push(Token::ColonEq);
                    pos += 2;
                } else {
                    return Err("expected ':='".to_string());
                }
            }
            '(' => {
                tokens.push(Token::LParen);
                pos += 1;
            }
            ')' => {
                tokens.push(Token::RParen);
                pos += 1;
            }
            '{' => {
                tokens.push(Token::LBrace);
                pos += 1;
            }
            '}' => {
                tokens.push(Token::RBrace);
                pos += 1;
            }
            ';' => {
                tokens.push(Token::Semi);
                pos += 1;
            }
            _ => return Err(format!("unexpected character: {ch}")),
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
        Self { tokens, pos: 0 }
    }

    fn expect_eof(&self) -> Result<(), String> {
        if self.pos == self.tokens.len() {
            Ok(())
        } else {
            Err("unexpected trailing tokens".to_string())
        }
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

    fn parse_expr(&mut self) -> Result<Expr, String> {
        if matches!(self.peek(), Some(Token::If)) {
            self.parse_if_expr()
        } else {
            self.parse_binop_expr()
        }
    }

    fn parse_if_expr(&mut self) -> Result<Expr, String> {
        self.expect_token(Token::If)?;
        let cond = self.parse_expr()?;
        self.expect_token(Token::Then)?;
        let then_branch = self.parse_expr()?;
        self.expect_token(Token::Else)?;
        let else_branch = self.parse_expr()?;
        self.expect_token(Token::Fi)?;
        Ok(Expr::If {
            cond: Box::new(cond),
            then_branch: Box::new(then_branch),
            else_branch: Box::new(else_branch),
        })
    }

    fn parse_binop_expr(&mut self) -> Result<Expr, String> {
        let mut expr = self.parse_application()?;
        loop {
            let op = match self.peek() {
                Some(Token::Plus) => BinOp::Add,
                Some(Token::Minus) => BinOp::Sub,
                Some(Token::AndAnd) => BinOp::And,
                _ => break,
            };
            self.next();
            let rhs = self.parse_application()?;
            expr = Expr::BinOp(Box::new(expr), op, Box::new(rhs));
        }
        Ok(expr)
    }

    fn parse_application(&mut self) -> Result<Expr, String> {
        let mut expr = self.parse_prefix()?;
        loop {
            if !matches!(self.peek(), Some(Token::LParen)) {
                break;
            }
            self.next();
            let arg = self.parse_expr()?;
            self.expect_token(Token::RParen)?;
            expr = Expr::App(Box::new(expr), Box::new(arg));
        }
        Ok(expr)
    }

    fn parse_prefix(&mut self) -> Result<Expr, String> {
        match self.peek() {
            Some(Token::Inc) => {
                self.next();
                Ok(Expr::UnOp(UnOp::Inc, Box::new(self.parse_prefix()?)))
            }
            Some(Token::Dec) => {
                self.next();
                Ok(Expr::UnOp(UnOp::Dec, Box::new(self.parse_prefix()?)))
            }
            Some(Token::Not) => {
                self.next();
                Ok(Expr::UnOp(UnOp::Not, Box::new(self.parse_prefix()?)))
            }
            _ => self.parse_atom(),
        }
    }

    fn parse_atom(&mut self) -> Result<Expr, String> {
        match self.next() {
            Some(Token::Number(value)) => Ok(Expr::Nat(value)),
            Some(Token::True) => Ok(Expr::Bool(true)),
            Some(Token::False) => Ok(Expr::Bool(false)),
            Some(Token::Unit) => Ok(Expr::Unit),
            Some(Token::Ident(name)) => Ok(Expr::Var(Identifier::new(name).map_err(|e| e.to_string())?)),
            Some(Token::Fun) => {
                let param = self.parse_identifier()?;
                self.expect_token(Token::Arrow)?;
                let body = self.parse_expr()?;
                Ok(Expr::Fun {
                    param,
                    body: Box::new(body),
                })
            }
            Some(Token::Rec) => {
                let name = self.parse_identifier()?;
                let param = self.parse_identifier()?;
                self.expect_token(Token::Arrow)?;
                let body = self.parse_expr()?;
                Ok(Expr::Rec {
                    name,
                    param,
                    body: Box::new(body),
                })
            }
            Some(Token::LParen) => {
                let expr = self.parse_expr()?;
                self.expect_token(Token::RParen)?;
                Ok(expr)
            }
            Some(Token::LBrace) => {
                let (stmt, expr) = self.parse_block_contents()?;
                self.expect_token(Token::RBrace)?;
                Ok(Expr::Block {
                    stmt: Box::new(stmt),
                    expr: Box::new(expr),
                })
            }
            Some(Token::If) => {
                self.pos -= 1;
                self.parse_if_expr()
            }
            other => Err(format!("unexpected token: {other:?}")),
        }
    }

    fn parse_block_contents(&mut self) -> Result<(Stmt, Expr), String> {
        let stmt = self.parse_simple_stmt()?;
        self.expect_token(Token::Semi)?;

        let checkpoint = self.pos;
        if let Ok((next_stmt, expr)) = self.parse_block_contents() {
            return Ok((Stmt::Seq(Box::new(stmt), Box::new(next_stmt)), expr));
        }
        self.pos = checkpoint;

        let expr = self.parse_expr()?;
        Ok((stmt, expr))
    }

    fn parse_stmt(&mut self) -> Result<Stmt, String> {
        let mut stmt = self.parse_simple_stmt()?;
        while matches!(self.peek(), Some(Token::Semi)) {
            self.next();
            let next = self.parse_simple_stmt()?;
            stmt = Stmt::Seq(Box::new(stmt), Box::new(next));
        }
        Ok(stmt)
    }

    fn parse_simple_stmt(&mut self) -> Result<Stmt, String> {
        match self.next() {
            Some(Token::Skip) => Ok(Stmt::Skip),
            Some(Token::Print) => match self.next() {
                Some(Token::Number(value)) => Ok(Stmt::Print(value)),
                other => Err(format!(
                    "print expects a natural number literal, found {other:?}"
                )),
            },
            Some(Token::Let) => {
                let name = self.parse_identifier()?;
                self.expect_token(Token::ColonEq)?;
                let expr = self.parse_expr()?;
                Ok(Stmt::Let {
                    name,
                    expr: Box::new(expr),
                })
            }
            Some(Token::If) => {
                let cond = self.parse_expr()?;
                self.expect_token(Token::Then)?;
                let then_branch = self.parse_stmt()?;
                self.expect_token(Token::End)?;
                Ok(Stmt::If {
                    cond: Box::new(cond),
                    then_branch: Box::new(then_branch),
                })
            }
            Some(Token::While) => {
                let cond = self.parse_expr()?;
                self.expect_token(Token::Do)?;
                let body = self.parse_stmt()?;
                self.expect_token(Token::End)?;
                Ok(Stmt::While {
                    cond: Box::new(cond),
                    body: Box::new(body),
                })
            }
            other => Err(format!("statement expected, found {other:?}")),
        }
    }

    fn parse_identifier(&mut self) -> Result<Identifier, String> {
        match self.next() {
            Some(Token::Ident(name)) => Identifier::new(name).map_err(|e| e.to_string()),
            other => Err(format!("identifier expected, found {other:?}")),
        }
    }

    fn expect_token(&mut self, expected: Token) -> Result<(), String> {
        let token = self.next();
        if token == Some(expected.clone()) {
            Ok(())
        } else {
            Err(format!("expected {:?}, found {:?}", expected, token))
        }
    }
}
