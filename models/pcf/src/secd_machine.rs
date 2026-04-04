use serde::{Deserialize, Serialize};
use utils::{Machine, StepResult, TextCodec, identifier::Identifier};

use crate::syntax::{BinOp, Expr, PrintEffect, UnOp};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SecdMachine {
    pub code: SecdCode,
    pub stack: Vec<Value>,
    pub env: Env,
    pub control: Vec<Instr>,
    pub dump: Vec<DumpFrame>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SecdCode(pub Vec<Instr>);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Instr {
    PushNat(usize),
    PushBool(bool),
    PushUnit,
    Print(usize),
    Acc(Identifier),
    MkCls {
        param: Identifier,
        code: Vec<Instr>,
    },
    MkRec {
        name: Identifier,
        param: Identifier,
        code: Vec<Instr>,
    },
    BinOp(BinOp),
    UnOp(UnOp),
    Ap,
    Ret,
    If {
        then_code: Vec<Instr>,
        else_code: Vec<Instr>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DumpFrame {
    pub stack: Vec<Value>,
    pub env: Env,
    pub control: Vec<Instr>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Value {
    Nat(usize),
    Bool(bool),
    Unit,
    Fun {
        param: Identifier,
        code: Vec<Instr>,
        env: Env,
    },
    Rec {
        name: Identifier,
        param: Identifier,
        code: Vec<Instr>,
        env: Env,
    },
}

pub type Env = Vec<(Identifier, Value)>;

impl TextCodec for SecdCode {
    fn parse(text: &str) -> Result<Self, String> {
        serde_json::from_str(text).map_err(|e| e.to_string())
    }

    fn write_fmt(&self, f: &mut impl std::fmt::Write) -> std::fmt::Result {
        let text = serde_json::to_string_pretty(self).map_err(|_| std::fmt::Error)?;
        write!(f, "{text}")
    }
}

impl Machine for SecdMachine {
    type Code = SecdCode;
    type AInput = Vec<usize>;
    type FOutput = usize;
    type SnapShot = SecdMachine;
    type RInput = ();
    type ROutput = PrintEffect;

    fn make(code: Self::Code, ainput: Self::AInput) -> Result<Self, String> {
        let mut control = code.0.clone();
        for input in ainput {
            control.push(Instr::PushNat(input));
            control.push(Instr::Ap);
        }
        Ok(Self {
            code,
            stack: Vec::new(),
            env: Vec::new(),
            control,
            dump: Vec::new(),
        })
    }

    fn step(mut self, _rinput: Self::RInput) -> Result<StepResult<Self>, String> {
        if self.control.is_empty() && self.dump.is_empty() {
            let value = self
                .stack
                .pop()
                .ok_or_else(|| "empty stack at halt".to_string())?;
            return Ok(StepResult::Halt {
                output: value.as_nat()?,
            });
        }

        let output = self.step_once()?;
        Ok(StepResult::Continue { next: self, output })
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
        utils::render_state![utils::render_text!(text, title: "secd")]
    }
}

impl SecdMachine {
    fn step_once(&mut self) -> Result<PrintEffect, String> {
        let instr = if self.control.is_empty() {
            return Err("no instruction to execute".to_string());
        } else {
            self.control.remove(0)
        };

        match instr {
            Instr::PushNat(value) => self.stack.push(Value::Nat(value)),
            Instr::PushBool(value) => self.stack.push(Value::Bool(value)),
            Instr::PushUnit => self.stack.push(Value::Unit),
            Instr::Print(value) => {
                self.stack.push(Value::Unit);
                return Ok(PrintEffect(Some(value)));
            }
            Instr::Acc(name) => self.stack.push(lookup_env(&self.env, &name)?),
            Instr::MkCls { param, code } => self.stack.push(Value::Fun {
                param,
                code,
                env: self.env.clone(),
            }),
            Instr::MkRec { name, param, code } => self.stack.push(Value::Rec {
                name,
                param,
                code,
                env: self.env.clone(),
            }),
            Instr::BinOp(op) => {
                let rhs = self.pop_value()?;
                let lhs = self.pop_value()?;
                self.stack.push(eval_binop(lhs, op, rhs)?);
            }
            Instr::UnOp(op) => {
                let value = self.pop_value()?;
                self.stack.push(eval_unop(op, value)?);
            }
            Instr::Ap => {
                let arg = self.pop_value()?;
                let fun = self.pop_value()?;
                match fun {
                    Value::Fun { param, code, env } => {
                        let frame = DumpFrame {
                            stack: self.stack.clone(),
                            env: self.env.clone(),
                            control: self.control.clone(),
                        };
                        self.dump.push(frame);
                        self.stack.clear();
                        self.env = env;
                        self.env.push((param, arg));
                        self.control = code;
                    }
                    Value::Rec {
                        name,
                        param,
                        code,
                        env,
                    } => {
                        let frame = DumpFrame {
                            stack: self.stack.clone(),
                            env: self.env.clone(),
                            control: self.control.clone(),
                        };
                        let mut next_env = env.clone();
                        next_env.push((
                            name.clone(),
                            Value::Rec {
                                name,
                                param: param.clone(),
                                code: code.clone(),
                                env: env.clone(),
                            },
                        ));
                        next_env.push((param, arg));
                        self.dump.push(frame);
                        self.stack.clear();
                        self.env = next_env;
                        self.control = code;
                    }
                    other => return Err(format!("application expects a closure, found {other:?}")),
                }
            }
            Instr::Ret => {
                let value = self.pop_value()?;
                let frame = self
                    .dump
                    .pop()
                    .ok_or_else(|| "ret without a caller frame".to_string())?;
                self.stack = frame.stack;
                self.stack.push(value);
                self.env = frame.env;
                self.control = frame.control;
            }
            Instr::If {
                then_code,
                else_code,
            } => {
                let cond = self.pop_value()?;
                match cond {
                    Value::Bool(true) => {
                        let mut next = then_code;
                        next.extend(self.control.clone());
                        self.control = next;
                    }
                    Value::Bool(false) => {
                        let mut next = else_code;
                        next.extend(self.control.clone());
                        self.control = next;
                    }
                    other => {
                        return Err(format!("if expects a boolean condition, found {other:?}"));
                    }
                }
            }
        }

        Ok(PrintEffect(None))
    }

    fn pop_value(&mut self) -> Result<Value, String> {
        self.stack
            .pop()
            .ok_or_else(|| "stack underflow".to_string())
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

fn eval_binop(lhs: Value, op: BinOp, rhs: Value) -> Result<Value, String> {
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

fn eval_unop(op: UnOp, value: Value) -> Result<Value, String> {
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

pub fn compile_expr(expr: &Expr) -> Vec<Instr> {
    match expr {
        Expr::Nat(value) => vec![Instr::PushNat(*value)],
        Expr::Bool(value) => vec![Instr::PushBool(*value)],
        Expr::Unit => vec![Instr::PushUnit],
        Expr::Print(value) => vec![Instr::Print(*value)],
        Expr::Var(name) => vec![Instr::Acc(name.clone())],
        Expr::BinOp(lhs, op, rhs) => {
            let mut code = compile_expr(lhs);
            code.extend(compile_expr(rhs));
            code.push(Instr::BinOp(*op));
            code
        }
        Expr::UnOp(op, expr) => {
            let mut code = compile_expr(expr);
            code.push(Instr::UnOp(*op));
            code
        }
        Expr::Fun { param, body } => {
            let mut body_code = compile_expr(body);
            body_code.push(Instr::Ret);
            vec![Instr::MkCls {
                param: param.clone(),
                code: body_code,
            }]
        }
        Expr::Rec { name, param, body } => {
            let mut body_code = compile_expr(body);
            body_code.push(Instr::Ret);
            vec![Instr::MkRec {
                name: name.clone(),
                param: param.clone(),
                code: body_code,
            }]
        }
        Expr::App(fun, arg) => {
            let mut code = compile_expr(fun);
            code.extend(compile_expr(arg));
            code.push(Instr::Ap);
            code
        }
        Expr::If {
            cond,
            then_branch,
            else_branch,
        } => {
            let mut code = compile_expr(cond);
            code.push(Instr::If {
                then_code: compile_expr(then_branch),
                else_code: compile_expr(else_branch),
            });
            code
        }
    }
}
