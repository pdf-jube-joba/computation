#[path = "expr_lang_compiler.rs"]
pub mod expr_lang_compiler;
#[path = "expr_lang_parser.rs"]
pub mod expr_lang_parser;

use serde::{Deserialize, Serialize};
use utils::identifier::Identifier;
use utils::number::Number;
use utils::{Machine, StepResult};

pub use expr_lang_compiler::ExprLangToFlowIrCompiler;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExprCode(pub Stmt);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AExp {
    Var(Identifier),
    Num(Number),
    BinOp {
        lhs: Box<AExp>,
        op: ABinOp,
        rhs: Box<AExp>,
    },
    IfThenElse {
        cond: Box<BExp>,
        then_exp: Box<AExp>,
        else_exp: Box<AExp>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ABinOp {
    Add,
    Sub,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum BExp {
    Rel {
        lhs: Box<AExp>,
        rel: RelOp,
        rhs: Box<AExp>,
    },
    Or(Box<BExp>, Box<BExp>),
    Not(Box<BExp>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RelOp {
    Lt,
    Eq,
    Gt,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Stmt {
    Nop,
    Assign { var: Identifier, expr: AExp },
    Seq(Box<Stmt>, Box<Stmt>),
    If { cond: BExp, body: Box<Stmt> },
    While { cond: BExp, body: Box<Stmt> },
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Environment {
    pub vars: Vec<(Identifier, Number)>,
}

impl Environment {
    pub fn get(&self, var: &Identifier) -> Number {
        self.vars
            .iter()
            .find_map(|(k, v)| if k == var { Some(v.clone()) } else { None })
            .unwrap_or_default()
    }

    pub fn set(&mut self, var: Identifier, value: Number) {
        if let Some((_, v)) = self.vars.iter_mut().find(|(k, _)| *k == var) {
            *v = value;
        } else {
            self.vars.push((var, value));
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExprLangMachine {
    pub code: ExprCode,
    pub stmt: Stmt,
    pub env: Environment,
}

impl ExprLangMachine {
    fn eval_aexp(exp: &AExp, env: &Environment) -> Result<Number, String> {
        match exp {
            AExp::Var(v) => Ok(env.get(v)),
            AExp::Num(n) => Ok(n.clone()),
            AExp::BinOp { lhs, op, rhs } => {
                let l = Self::eval_aexp(lhs, env)?;
                let r = Self::eval_aexp(rhs, env)?;
                Ok(match op {
                    ABinOp::Add => l + r,
                    ABinOp::Sub => l - r,
                })
            }
            AExp::IfThenElse {
                cond,
                then_exp,
                else_exp,
            } => {
                if Self::eval_bexp(cond, env)? {
                    Self::eval_aexp(then_exp, env)
                } else {
                    Self::eval_aexp(else_exp, env)
                }
            }
        }
    }

    fn eval_bexp(exp: &BExp, env: &Environment) -> Result<bool, String> {
        match exp {
            BExp::Rel { lhs, rel, rhs } => {
                let l = Self::eval_aexp(lhs, env)?;
                let r = Self::eval_aexp(rhs, env)?;
                Ok(match rel {
                    RelOp::Lt => l < r,
                    RelOp::Eq => l == r,
                    RelOp::Gt => l > r,
                })
            }
            BExp::Or(a, b) => Ok(Self::eval_bexp(a, env)? || Self::eval_bexp(b, env)?),
            BExp::Not(b) => Ok(!Self::eval_bexp(b, env)?),
        }
    }

    fn small_step(stmt: Stmt, mut env: Environment) -> Result<(Stmt, Environment), String> {
        match stmt {
            Stmt::Nop => Ok((Stmt::Nop, env)),
            Stmt::Assign { var, expr } => {
                let value = Self::eval_aexp(&expr, &env)?;
                env.set(var, value);
                Ok((Stmt::Nop, env))
            }
            Stmt::Seq(a, b) => {
                if matches!(&*a, Stmt::Nop) {
                    Ok((*b, env))
                } else {
                    let (next_a, next_env) = Self::small_step(*a, env)?;
                    Ok((Stmt::Seq(Box::new(next_a), b), next_env))
                }
            }
            Stmt::If { cond, body } => {
                if Self::eval_bexp(&cond, &env)? {
                    Ok((*body, env))
                } else {
                    Ok((Stmt::Nop, env))
                }
            }
            Stmt::While { cond, body } => Ok((
                Stmt::If {
                    cond: cond.clone(),
                    body: Box::new(Stmt::Seq(
                        body.clone(),
                        Box::new(Stmt::While { cond, body }),
                    )),
                },
                env,
            )),
        }
    }
}

impl Machine for ExprLangMachine {
    type Code = ExprCode;
    type AInput = Environment;
    type FOutput = Environment;
    type SnapShot = ExprLangMachine;
    type RInput = ();
    type ROutput = ();

    fn make(code: Self::Code, ainput: Self::AInput) -> Result<Self, String> {
        Ok(Self {
            stmt: code.0.clone(),
            code,
            env: ainput,
        })
    }

    fn step(self, _rinput: Self::RInput) -> Result<StepResult<Self>, String> {
        if matches!(self.stmt, Stmt::Nop) {
            let output = self.env.clone();
            return Ok(StepResult::Halt { output });
        }

        let (stmt, env) = Self::small_step(self.stmt, self.env)?;
        Ok(StepResult::Continue {
            next: Self {
                code: self.code,
                stmt,
                env,
            },
            output: (),
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
        utils::render_state![utils::render_text!(text, title: "snapshot")]
    }
}
