use serde::{Deserialize, Serialize};
use utils::{Machine, StepResult, identifier::Identifier};

use crate::syntax::{BinOp, Expr, ExprCode, PrintEffect, UnOp, substitute};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExprMachine {
    pub code: ExprCode,
    pub expr: Expr,
}

impl Machine for ExprMachine {
    type Code = ExprCode;
    type AInput = Vec<usize>;
    type FOutput = usize;
    type SnapShot = ExprMachine;
    type RInput = ();
    type ROutput = PrintEffect;

    fn make(code: Self::Code, ainput: Self::AInput) -> Result<Self, String> {
        Ok(Self {
            expr: code.0.clone().apply_inputs(&ainput),
            code,
        })
    }

    fn step(self, _rinput: Self::RInput) -> Result<StepResult<Self>, String> {
        if self.expr.is_value() {
            return Ok(StepResult::Halt {
                output: self.expr.expect_nat()?,
            });
        }

        let (expr, output) = small_step(self.expr)?;
        Ok(StepResult::Continue {
            next: Self {
                code: self.code,
                expr,
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
        crate::expr_render::render_machine(snapshot)
    }
}

fn small_step(expr: Expr) -> Result<(Expr, PrintEffect), String> {
    match expr {
        Expr::Print(value) => Ok((Expr::Unit, PrintEffect(Some(value)))),
        Expr::BinOp(lhs, op, rhs) => {
            if !lhs.is_value() {
                let (next_lhs, output) = small_step(*lhs)?;
                return Ok((Expr::BinOp(Box::new(next_lhs), op, rhs), output));
            }
            if !rhs.is_value() {
                let (next_rhs, output) = small_step(*rhs)?;
                return Ok((Expr::BinOp(lhs, op, Box::new(next_rhs)), output));
            }
            Ok((eval_binop(&lhs, op, &rhs)?, PrintEffect(None)))
        }
        Expr::UnOp(op, inner) => {
            if !inner.is_value() {
                let (next_inner, output) = small_step(*inner)?;
                return Ok((Expr::UnOp(op, Box::new(next_inner)), output));
            }
            Ok((eval_unop(op, &inner)?, PrintEffect(None)))
        }
        Expr::App(fun, arg) => {
            if !fun.is_value() {
                let (next_fun, output) = small_step(*fun)?;
                return Ok((Expr::App(Box::new(next_fun), arg), output));
            }
            if !arg.is_value() {
                let (next_arg, output) = small_step(*arg)?;
                return Ok((Expr::App(fun, Box::new(next_arg)), output));
            }
            match *fun {
                Expr::Fun { param, body } => {
                    Ok((substitute(&body, &param, &arg), PrintEffect(None)))
                }
                Expr::Rec { name, param, body } => {
                    let rec_value = Expr::Rec {
                        name: name.clone(),
                        param: param.clone(),
                        body: body.clone(),
                    };
                    let body = substitute(&body, &name, &rec_value);
                    Ok((substitute(&body, &param, &arg), PrintEffect(None)))
                }
                other => Err(format!(
                    "application expects a function value, found {other}"
                )),
            }
        }
        Expr::If {
            cond,
            then_branch,
            else_branch,
        } => {
            if !cond.is_value() {
                let (next_cond, output) = small_step(*cond)?;
                return Ok((
                    Expr::If {
                        cond: Box::new(next_cond),
                        then_branch,
                        else_branch,
                    },
                    output,
                ));
            }
            match *cond {
                Expr::Bool(true) => Ok((*then_branch, PrintEffect(None))),
                Expr::Bool(false) => Ok((*else_branch, PrintEffect(None))),
                other => Err(format!("if expects a boolean condition, found {other}")),
            }
        }
        Expr::Var(name) => Err(format!("unbound variable: {}", name.as_str())),
        Expr::Nat(_) | Expr::Bool(_) | Expr::Unit | Expr::Fun { .. } | Expr::Rec { .. } => {
            Err("cannot step a value".to_string())
        }
    }
}

pub fn eval_binop(lhs: &Expr, op: BinOp, rhs: &Expr) -> Result<Expr, String> {
    match op {
        BinOp::Add => match (lhs, rhs) {
            (Expr::Nat(left), Expr::Nat(right)) => Ok(Expr::Nat(left + right)),
            _ => Err("operator '+' expects naturals".to_string()),
        },
        BinOp::Sub => match (lhs, rhs) {
            (Expr::Nat(left), Expr::Nat(right)) => left
                .checked_sub(*right)
                .map(Expr::Nat)
                .ok_or_else(|| "operator '-' expects lhs >= rhs over naturals".to_string()),
            _ => Err("operator '-' expects naturals".to_string()),
        },
        BinOp::And => match (lhs, rhs) {
            (Expr::Bool(left), Expr::Bool(right)) => Ok(Expr::Bool(*left && *right)),
            _ => Err("operator '&&' expects booleans".to_string()),
        },
    }
}

pub fn eval_unop(op: UnOp, value: &Expr) -> Result<Expr, String> {
    match op {
        UnOp::Inc => match value {
            Expr::Nat(number) => Ok(Expr::Nat(number + 1)),
            _ => Err("operator 'inc' expects a natural".to_string()),
        },
        UnOp::Dec => match value {
            Expr::Nat(number) => number
                .checked_sub(1)
                .map(Expr::Nat)
                .ok_or_else(|| "operator 'dec' expects a positive natural".to_string()),
            _ => Err("operator 'dec' expects a natural".to_string()),
        },
        UnOp::Not => match value {
            Expr::Bool(flag) => Ok(Expr::Bool(!flag)),
            _ => Err("operator 'not' expects a boolean".to_string()),
        },
    }
}

pub fn bind(env: &mut Vec<(Identifier, Expr)>, name: Identifier, value: Expr) {
    env.push((name, value));
}
