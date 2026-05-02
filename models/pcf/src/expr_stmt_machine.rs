use serde::{Deserialize, Serialize};
use utils::{Machine, StepResult, identifier::Identifier};

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
        crate::expr_stmt_render::render_machine(snapshot)
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
                    other => Err(format!(
                        "while expects a boolean condition, found {other:?}"
                    )),
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
        .find_map(|(key, value)| {
            if key == name {
                Some(value.clone())
            } else {
                None
            }
        })
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
