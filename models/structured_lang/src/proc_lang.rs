use std::collections::{BTreeMap, HashMap};

use serde::{Deserialize, Serialize};
use utils::{Machine, StepResult, number::Number};

#[path = "proc_lang_compiler.rs"]
mod proc_lang_compiler;
#[path = "proc_lang_parser.rs"]
mod proc_lang_parser;
pub use proc_lang_compiler::ProcToFlowIrCompiler;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProcCode(pub Program);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Program {
    pub statics: Vec<String>,
    pub procs: Vec<ProcDef>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProcDef {
    pub name: String,
    pub params: Vec<String>,
    pub locals: Vec<String>,
    pub body: Stmt,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Stmt {
    Nop,
    Seq(Box<Stmt>, Box<Stmt>),
    Assign {
        var: String,
        expr: AExp,
    },
    If {
        cond: BExp,
        body: Box<Stmt>,
    },
    While {
        cond: BExp,
        body: Box<Stmt>,
    },
    Call {
        name: String,
        args: Vec<String>,
        rets: Vec<String>,
    },
    Return {
        vars: Vec<String>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Atom {
    Var(String),
    Imm(Number),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ABinOp {
    Add,
    Sub,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AExp {
    Atom(Atom),
    Bin { lhs: Atom, op: ABinOp, rhs: Atom },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RelOp {
    Lt,
    Eq,
    Gt,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BExp {
    pub lhs: Atom,
    pub op: RelOp,
    pub rhs: Atom,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct GlobalEnv {
    pub vars: BTreeMap<String, Number>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct Frame {
    stmt: Stmt,
    local_env: BTreeMap<String, Number>,
    ret_vars: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProcLangMachine {
    pub code: ProcCode,
    pub stmt: Stmt,
    pub global_env: GlobalEnv,
    pub local_env: BTreeMap<String, Number>,
    call_stack: Vec<Frame>,
    proc_table: HashMap<String, ProcDef>,
}

impl ProcLangMachine {
    fn lookup_var(&self, var: &str) -> Result<Number, String> {
        if let Some(v) = self.local_env.get(var) {
            return Ok(v.clone());
        }
        if let Some(v) = self.global_env.vars.get(var) {
            return Ok(v.clone());
        }
        Err(format!("Unknown variable: {var}"))
    }

    fn assign_var(&mut self, var: &str, value: Number) -> Result<(), String> {
        if let Some(slot) = self.local_env.get_mut(var) {
            *slot = value;
            return Ok(());
        }
        if let Some(slot) = self.global_env.vars.get_mut(var) {
            *slot = value;
            return Ok(());
        }
        Err(format!("Unknown variable: {var}"))
    }

    fn eval_atom(&self, atom: &Atom) -> Result<Number, String> {
        match atom {
            Atom::Var(v) => self.lookup_var(v),
            Atom::Imm(n) => Ok(n.clone()),
        }
    }

    fn eval_aexp(&self, exp: &AExp) -> Result<Number, String> {
        match exp {
            AExp::Atom(a) => self.eval_atom(a),
            AExp::Bin { lhs, op, rhs } => {
                let l = self.eval_atom(lhs)?;
                let r = self.eval_atom(rhs)?;
                Ok(match op {
                    ABinOp::Add => l + r,
                    ABinOp::Sub => l - r,
                })
            }
        }
    }

    fn eval_bexp(&self, exp: &BExp) -> Result<bool, String> {
        let l = self.eval_atom(&exp.lhs)?;
        let r = self.eval_atom(&exp.rhs)?;
        Ok(match exp.op {
            RelOp::Lt => l < r,
            RelOp::Eq => l == r,
            RelOp::Gt => l > r,
        })
    }

    fn with_implicit_return(stmt: Stmt) -> Stmt {
        if stmt.ends_with_return() {
            stmt
        } else {
            Stmt::Seq(Box::new(stmt), Box::new(Stmt::Return { vars: vec![] }))
        }
    }

    fn call_into_proc(
        &mut self,
        name: &str,
        args: &[String],
        rets: &[String],
        continuation: Stmt,
    ) -> Result<Stmt, String> {
        let proc = self
            .proc_table
            .get(name)
            .cloned()
            .ok_or_else(|| format!("Unknown procedure: {name}"))?;

        if args.len() != proc.params.len() {
            return Err(format!(
                "Argument count mismatch for {name}: expected {}, got {}",
                proc.params.len(),
                args.len()
            ));
        }

        let arg_values = args
            .iter()
            .map(|arg| self.lookup_var(arg))
            .collect::<Result<Vec<_>, _>>()?;

        let caller_local = self.local_env.clone();
        self.call_stack.push(Frame {
            stmt: continuation,
            local_env: caller_local,
            ret_vars: rets.to_vec(),
        });

        let mut next_local = BTreeMap::new();
        for (p, v) in proc.params.iter().zip(arg_values) {
            next_local.insert(p.clone(), v);
        }
        for local in &proc.locals {
            next_local.insert(local.clone(), Number::default());
        }
        self.local_env = next_local;

        Ok(Self::with_implicit_return(proc.body))
    }

    fn step_stmt(&mut self, stmt: Stmt) -> Result<Stmt, String> {
        match stmt {
            Stmt::Nop => Ok(Stmt::Nop),
            Stmt::Assign { var, expr } => {
                let value = self.eval_aexp(&expr)?;
                self.assign_var(&var, value)?;
                Ok(Stmt::Nop)
            }
            Stmt::If { cond, body } => {
                if self.eval_bexp(&cond)? {
                    Ok(*body)
                } else {
                    Ok(Stmt::Nop)
                }
            }
            Stmt::While { cond, body } => Ok(Stmt::If {
                cond: cond.clone(),
                body: Box::new(Stmt::Seq(
                    body.clone(),
                    Box::new(Stmt::While { cond, body }),
                )),
            }),
            Stmt::Seq(a, b) => match *a {
                Stmt::Nop => Ok(*b),
                Stmt::Return { vars } => Ok(Stmt::Return { vars }),
                Stmt::Call { name, args, rets } => self.call_into_proc(&name, &args, &rets, *b),
                left => {
                    let next_left = self.step_stmt(left)?;
                    Ok(Stmt::Seq(Box::new(next_left), b))
                }
            },
            Stmt::Call { name, args, rets } => self.call_into_proc(&name, &args, &rets, Stmt::Nop),
            Stmt::Return { vars } => {
                if let Some(frame) = self.call_stack.pop() {
                    if vars.len() != frame.ret_vars.len() {
                        return Err(format!(
                            "Return count mismatch: expected {}, got {}",
                            frame.ret_vars.len(),
                            vars.len()
                        ));
                    }

                    let values = vars
                        .iter()
                        .map(|v| self.lookup_var(v))
                        .collect::<Result<Vec<_>, _>>()?;

                    self.local_env = frame.local_env;
                    for (ret_var, value) in frame.ret_vars.iter().zip(values) {
                        self.assign_var(ret_var, value)?;
                    }
                    Ok(frame.stmt)
                } else {
                    Ok(Stmt::Return { vars })
                }
            }
        }
    }
}

impl Stmt {
    fn ends_with_return(&self) -> bool {
        match self {
            Stmt::Return { .. } => true,
            Stmt::Seq(_, b) => b.ends_with_return(),
            _ => false,
        }
    }
}

impl Machine for ProcLangMachine {
    type Code = ProcCode;
    type AInput = GlobalEnv;
    type FOutput = GlobalEnv;
    type RInput = String;
    type ROutput = String;
    type SnapShot = ProcLangMachine;

    fn make(code: Self::Code, ainput: Self::AInput) -> Result<Self, String> {
        let mut proc_table = HashMap::new();
        let mut static_set = BTreeMap::new();

        for s in &code.0.statics {
            if static_set.insert(s.clone(), ()).is_some() {
                return Err(format!("Duplicate static variable: {s}"));
            }
        }

        for (name, _) in &ainput.vars {
            if !static_set.contains_key(name) {
                return Err(format!(
                    "AInput contains undeclared static variable: {name}"
                ));
            }
        }
        for name in static_set.keys() {
            if !ainput.vars.contains_key(name) {
                return Err(format!("AInput missing static variable: {name}"));
            }
        }

        for proc in &code.0.procs {
            if proc_table.insert(proc.name.clone(), proc.clone()).is_some() {
                return Err(format!("Duplicate procedure name: {}", proc.name));
            }

            let mut seen = BTreeMap::new();
            for p in &proc.params {
                if static_set.contains_key(p) {
                    return Err(format!(
                        "Name conflict in {}: parameter {} conflicts with static",
                        proc.name, p
                    ));
                }
                if seen.insert(p.clone(), ()).is_some() {
                    return Err(format!("Duplicate parameter {} in {}", p, proc.name));
                }
            }
            for l in &proc.locals {
                if static_set.contains_key(l) {
                    return Err(format!(
                        "Name conflict in {}: local {} conflicts with static",
                        proc.name, l
                    ));
                }
                if seen.insert(l.clone(), ()).is_some() {
                    return Err(format!(
                        "Name conflict in {}: local {} conflicts with parameter/local",
                        proc.name, l
                    ));
                }
            }
        }

        let main = proc_table
            .get("main")
            .ok_or_else(|| "Procedure 'main' is required".to_string())?
            .clone();
        if !main.params.is_empty() {
            return Err("main must have no parameters".to_string());
        }

        let mut local_env = BTreeMap::new();
        for l in &main.locals {
            local_env.insert(l.clone(), Number::default());
        }

        Ok(Self {
            code,
            stmt: Self::with_implicit_return(main.body),
            global_env: ainput,
            local_env,
            call_stack: vec![],
            proc_table,
        })
    }

    fn step(mut self, _rinput: Self::RInput) -> Result<StepResult<Self>, String> {
        if matches!(self.stmt, Stmt::Return { .. }) && self.call_stack.is_empty() {
            return Ok(StepResult::Halt {
                output: self.global_env,
            });
        }

        let before = proc_lang_parser::stmt_to_text(&self.stmt);
        let next_stmt = self.step_stmt(self.stmt.clone())?;
        self.stmt = next_stmt;
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
        let globals = snapshot
            .global_env
            .vars
            .iter()
            .map(|(k, v)| format!("{k} = {}", v.to_decimal_string()))
            .collect::<Vec<_>>()
            .join("\n");

        let locals = snapshot
            .local_env
            .iter()
            .map(|(k, v)| format!("{k} = {}", v.to_decimal_string()))
            .collect::<Vec<_>>()
            .join("\n");

        utils::render_state![
            utils::render_text!(proc_lang_parser::stmt_to_text(&snapshot.stmt), title: "stmt"),
            utils::render_text!(globals, title: "global env"),
            utils::render_text!(locals, title: "local env"),
            utils::render_text!(snapshot.call_stack.len().to_string(), title: "call stack depth"),
        ]
    }
}
