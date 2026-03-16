#[path = "internal_ctrl_parser.rs"]
mod internal_ctrl_parser;
#[path = "internal_ctrl_compiler.rs"]
pub mod internal_ctrl_compiler;

use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};
use utils::identifier::Identifier;
use utils::number::Number;
use utils::{Machine, StepResult};

pub use internal_ctrl_compiler::InternalCtrlToFlowIrCompiler;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InternalCtrlCode {
    pub statics: Vec<Identifier>,
    pub body: Stmt,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Atom {
    Var(Identifier),
    Imm(Number),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ABinOp {
    Add,
    Sub,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AExp {
    Atom(Atom),
    Bin {
        lhs: Atom,
        op: ABinOp,
        rhs: Atom,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RelOp {
    Lt,
    Eq,
    Gt,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BExp {
    pub lhs: Atom,
    pub rel: RelOp,
    pub rhs: Atom,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Stmt {
    Nop,
    Seq(Box<Stmt>, Box<Stmt>),
    Assign {
        var: Identifier,
        expr: AExp,
    },
    If {
        cond: BExp,
        body: Box<Stmt>,
    },
    Break {
        label: Identifier,
        value: Identifier,
    },
    Continue {
        label: Identifier,
    },
    Loop {
        label: Identifier,
        body: Box<Stmt>,
        out: Identifier,
    },
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
enum Exec {
    Stmt(Stmt),
    Result(ControlResult),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
enum ControlResult {
    Normal,
    Break { label: Identifier, value: Number },
    Continue { label: Identifier },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
enum Frame {
    Loop {
        label: Identifier,
        body: Stmt,
        out: Identifier,
    },
    Seq(Stmt),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InternalCtrlMachine {
    pub code: InternalCtrlCode,
    pub env: Environment,
    exec: Exec,
    stack: Vec<Frame>,
}

impl InternalCtrlMachine {
    fn eval_atom(&self, atom: &Atom) -> Number {
        match atom {
            Atom::Var(v) => self.env.get(v),
            Atom::Imm(n) => n.clone(),
        }
    }

    fn eval_aexp(&self, exp: &AExp) -> Number {
        match exp {
            AExp::Atom(atom) => self.eval_atom(atom),
            AExp::Bin { lhs, op, rhs } => {
                let lhs = self.eval_atom(lhs);
                let rhs = self.eval_atom(rhs);
                match op {
                    ABinOp::Add => lhs + rhs,
                    ABinOp::Sub => lhs - rhs,
                }
            }
        }
    }

    fn eval_bexp(&self, exp: &BExp) -> bool {
        let lhs = self.eval_atom(&exp.lhs);
        let rhs = self.eval_atom(&exp.rhs);
        match exp.rel {
            RelOp::Lt => lhs < rhs,
            RelOp::Eq => lhs == rhs,
            RelOp::Gt => lhs > rhs,
        }
    }

    fn step_stmt(&mut self, stmt: Stmt) {
        match stmt {
            Stmt::Nop => self.exec = Exec::Result(ControlResult::Normal),
            Stmt::Seq(lhs, rhs) => {
                self.stack.push(Frame::Seq(*rhs));
                self.exec = Exec::Stmt(*lhs);
            }
            Stmt::Assign { var, expr } => {
                let value = self.eval_aexp(&expr);
                self.env.set(var, value);
                self.exec = Exec::Result(ControlResult::Normal);
            }
            Stmt::If { cond, body } => {
                if self.eval_bexp(&cond) {
                    self.exec = Exec::Stmt(*body);
                } else {
                    self.exec = Exec::Result(ControlResult::Normal);
                }
            }
            Stmt::Break { label, value } => {
                let value = self.env.get(&value);
                self.exec = Exec::Result(ControlResult::Break { label, value });
            }
            Stmt::Continue { label } => {
                self.exec = Exec::Result(ControlResult::Continue { label });
            }
            Stmt::Loop { label, body, out } => {
                self.stack.push(Frame::Loop {
                    label,
                    body: (*body).clone(),
                    out,
                });
                self.exec = Exec::Result(ControlResult::Normal);
            }
        }
    }

    fn unwind(&mut self, result: ControlResult) -> Result<(), String> {
        match self.stack.pop() {
            None => match result {
                ControlResult::Normal => Ok(()),
                ControlResult::Break { label, .. } => {
                    Err(format!("break outside loop: {}", label.as_str()))
                }
                ControlResult::Continue { label } => {
                    Err(format!("continue outside loop: {}", label.as_str()))
                }
            },
            Some(Frame::Seq(next)) => match result {
                ControlResult::Normal => {
                    self.exec = Exec::Stmt(next);
                    Ok(())
                }
                other => {
                    self.exec = Exec::Result(other);
                    Ok(())
                }
            },
            Some(Frame::Loop { label, body, out }) => match result {
                ControlResult::Normal => {
                    self.stack.push(Frame::Loop { label, body: body.clone(), out });
                    self.exec = Exec::Stmt(body);
                    Ok(())
                }
                ControlResult::Continue { label: got } if got == label => {
                    self.stack.push(Frame::Loop { label, body: body.clone(), out });
                    self.exec = Exec::Result(ControlResult::Normal);
                    Ok(())
                }
                ControlResult::Break { label: got, value } if got == label => {
                    self.env.set(out, value);
                    self.exec = Exec::Result(ControlResult::Normal);
                    Ok(())
                }
                other => {
                    self.exec = Exec::Result(other);
                    Ok(())
                }
            },
        }
    }

    fn exec_to_text(exec: &Exec) -> String {
        match exec {
            Exec::Stmt(stmt) => internal_ctrl_parser::stmt_to_text(stmt),
            Exec::Result(ControlResult::Normal) => "()".to_string(),
            Exec::Result(ControlResult::Break { label, value }) => {
                format!("break {} {}", label.as_str(), value.to_decimal_string())
            }
            Exec::Result(ControlResult::Continue { label }) => {
                format!("continue {}", label.as_str())
            }
        }
    }
}

impl Machine for InternalCtrlMachine {
    type Code = InternalCtrlCode;
    type AInput = Environment;
    type FOutput = Environment;
    type SnapShot = InternalCtrlMachine;
    type RInput = ();
    type ROutput = String;

    fn make(code: Self::Code, ainput: Self::AInput) -> Result<Self, String> {
        let mut statics = BTreeSet::new();
        for ident in &code.statics {
            if !statics.insert(ident.as_str().to_string()) {
                return Err(format!("duplicate static variable: {}", ident.as_str()));
            }
        }
        for (name, _) in &ainput.vars {
            if !statics.contains(name.as_str()) {
                return Err(format!("ainput contains undeclared static variable: {}", name.as_str()));
            }
        }
        for name in &statics {
            if !ainput.vars.iter().any(|(k, _)| k.as_str() == name) {
                return Err(format!("ainput missing static variable: {name}"));
            }
        }
        Ok(Self {
            code: code.clone(),
            env: ainput,
            exec: Exec::Stmt(code.body),
            stack: vec![],
        })
    }

    fn step(mut self, _rinput: Self::RInput) -> Result<StepResult<Self>, String> {
        let before = Self::exec_to_text(&self.exec);
        match self.exec.clone() {
            Exec::Stmt(stmt) => self.step_stmt(stmt),
            Exec::Result(result) => {
                self.unwind(result)?;
                if matches!(self.exec, Exec::Result(ControlResult::Normal)) && self.stack.is_empty() {
                    return Ok(StepResult::Halt { output: self.env });
                }
            }
        }
        Ok(StepResult::Continue {
            next: self,
            output: before,
        })
    }

    fn snapshot(&self) -> Self::SnapShot {
        self.clone()
    }

    fn restore(snapshot: Self::SnapShot) -> Self {
        snapshot
    }

    fn render(snapshot: Self::SnapShot) -> utils::RenderState {
        let env = snapshot
            .env
            .vars
            .iter()
            .map(|(k, v)| format!("{} = {}", k.as_str(), v.to_decimal_string()))
            .collect::<Vec<_>>()
            .join("\n");
        let stack = snapshot
            .stack
            .iter()
            .map(|frame| match frame {
                Frame::Loop { label, out, .. } => {
                    format!("loop {} -> {}", label.as_str(), out.as_str())
                }
                Frame::Seq(stmt) => format!("seq {}", internal_ctrl_parser::stmt_to_text(stmt)),
            })
            .collect::<Vec<_>>()
            .join("\n");
        utils::render_state![
            utils::render_text!(Self::exec_to_text(&snapshot.exec), title: "exec"),
            utils::render_text!(env, title: "env"),
            utils::render_text!(stack, title: "stack"),
        ]
    }
}
