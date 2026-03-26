use serde::{Deserialize, Serialize};
use utils::{Machine, StepResult, identifier::Identifier};

use crate::expr_machine::{eval_binop, eval_unop};
use crate::syntax::{BinOp, Expr, ExprCode, PrintEffect, UnOp};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CekMachine {
    pub code: ExprCode,
    pub state: CekState,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CekState {
    Eval {
        control: Expr,
        env: Env,
        kont: Vec<Frame>,
    },
    Return {
        value: Value,
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
    If {
        then_branch: Expr,
        else_branch: Expr,
        env: Env,
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

impl Machine for CekMachine {
    type Code = ExprCode;
    type AInput = Vec<usize>;
    type FOutput = usize;
    type SnapShot = CekMachine;
    type RInput = ();
    type ROutput = PrintEffect;

    fn make(code: Self::Code, ainput: Self::AInput) -> Result<Self, String> {
        Ok(Self {
            state: CekState::Eval {
                control: code.0.clone().apply_inputs(&ainput),
                env: Vec::new(),
                kont: vec![Frame::Init],
            },
            code,
        })
    }

    fn step(self, _rinput: Self::RInput) -> Result<StepResult<Self>, String> {
        if let CekState::Return { value, kont } = &self.state
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
        utils::render_state![utils::render_text!(text, title: "cek")]
    }
}

fn step_state(state: CekState) -> Result<(CekState, PrintEffect), String> {
    match state {
        CekState::Eval {
            control,
            env,
            mut kont,
        } => match control {
            Expr::Nat(value) => Ok((
                CekState::Return {
                    value: Value::Nat(value),
                    kont,
                },
                PrintEffect(None),
            )),
            Expr::Bool(value) => Ok((
                CekState::Return {
                    value: Value::Bool(value),
                    kont,
                },
                PrintEffect(None),
            )),
            Expr::Unit => Ok((
                CekState::Return {
                    value: Value::Unit,
                    kont,
                },
                PrintEffect(None),
            )),
            Expr::Print(value) => Ok((
                CekState::Return {
                    value: Value::Unit,
                    kont,
                },
                PrintEffect(Some(value)),
            )),
            Expr::Var(name) => Ok((
                CekState::Return {
                    value: lookup_env(&env, &name)?,
                    kont,
                },
                PrintEffect(None),
            )),
            Expr::BinOp(lhs, op, rhs) => {
                kont.push(Frame::BinOpLeft { op, rhs: *rhs, env });
                Ok((
                    CekState::Eval {
                        control: *lhs,
                        env: env_from_last_frame(&kont)?,
                        kont,
                    },
                    PrintEffect(None),
                ))
            }
            Expr::UnOp(op, expr) => {
                kont.push(Frame::UnOp { op });
                Ok((
                    CekState::Eval {
                        control: *expr,
                        env,
                        kont,
                    },
                    PrintEffect(None),
                ))
            }
            Expr::Fun { param, body } => Ok((
                CekState::Return {
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
                CekState::Return {
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
                    CekState::Eval {
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
                kont.push(Frame::If {
                    then_branch: *then_branch,
                    else_branch: *else_branch,
                    env: env.clone(),
                });
                Ok((
                    CekState::Eval {
                        control: *cond,
                        env,
                        kont,
                    },
                    PrintEffect(None),
                ))
            }
        },
        CekState::Return { value, mut kont } => {
            let frame = kont.pop().ok_or_else(|| "empty continuation".to_string())?;
            match frame {
                Frame::Init => {
                    kont.push(Frame::Init);
                    Ok((CekState::Return { value, kont }, PrintEffect(None)))
                }
                Frame::BinOpLeft { op, rhs, env } => {
                    kont.push(Frame::BinOpRight { lhs: value, op });
                    Ok((
                        CekState::Eval {
                            control: rhs,
                            env,
                            kont,
                        },
                        PrintEffect(None),
                    ))
                }
                Frame::BinOpRight { lhs, op } => Ok((
                    CekState::Return {
                        value: expr_to_value(eval_binop(
                            &value_to_expr(&lhs),
                            op,
                            &value_to_expr(&value),
                        )?)?,
                        kont,
                    },
                    PrintEffect(None),
                )),
                Frame::UnOp { op } => Ok((
                    CekState::Return {
                        value: expr_to_value(eval_unop(op, &value_to_expr(&value))?)?,
                        kont,
                    },
                    PrintEffect(None),
                )),
                Frame::AppFun { arg, env } => {
                    kont.push(Frame::AppArg { fun: value });
                    Ok((
                        CekState::Eval {
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
                            CekState::Eval {
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
                            CekState::Eval {
                                control: body,
                                env: next_env,
                                kont,
                            },
                            PrintEffect(None),
                        ))
                    }
                    other => Err(format!("application expects a closure, found {other:?}")),
                },
                Frame::If {
                    then_branch,
                    else_branch,
                    env,
                } => match value {
                    Value::Bool(true) => Ok((
                        CekState::Eval {
                            control: then_branch,
                            env,
                            kont,
                        },
                        PrintEffect(None),
                    )),
                    Value::Bool(false) => Ok((
                        CekState::Eval {
                            control: else_branch,
                            env,
                            kont,
                        },
                        PrintEffect(None),
                    )),
                    other => Err(format!("if expects a boolean condition, found {other:?}")),
                },
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

fn value_to_expr(value: &Value) -> Expr {
    match value {
        Value::Nat(number) => Expr::Nat(*number),
        Value::Bool(flag) => Expr::Bool(*flag),
        Value::Unit => Expr::Unit,
        Value::Fun { param, body, .. } => Expr::Fun {
            param: param.clone(),
            body: Box::new(body.clone()),
        },
        Value::Rec {
            name, param, body, ..
        } => Expr::Rec {
            name: name.clone(),
            param: param.clone(),
            body: Box::new(body.clone()),
        },
    }
}

fn expr_to_value(expr: Expr) -> Result<Value, String> {
    match expr {
        Expr::Nat(number) => Ok(Value::Nat(number)),
        Expr::Bool(flag) => Ok(Value::Bool(flag)),
        Expr::Unit => Ok(Value::Unit),
        Expr::Fun { .. } | Expr::Rec { .. } => {
            Err("plain expr closures cannot be converted without an environment".to_string())
        }
        other => Err(format!("expected value expression, found {other}")),
    }
}

fn env_from_last_frame(kont: &[Frame]) -> Result<Env, String> {
    match kont.last() {
        Some(Frame::BinOpLeft { env, .. }) => Ok(env.clone()),
        _ => Err("internal CEK error: missing binop frame env".to_string()),
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
