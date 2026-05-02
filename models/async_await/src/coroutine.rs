#[path = "coroutine_parser.rs"]
mod coroutine_parser;
#[path = "coroutine_render.rs"]
mod coroutine_render;

use std::collections::{BTreeMap, HashMap, VecDeque};

use serde::{Deserialize, Serialize};
use utils::number::Number;
use utils::{Machine, StepResult};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CoroutineCode(pub Program);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Program {
    pub functions: Vec<FnDecl>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FnDecl {
    pub name: String,
    pub body: Vec<Stmt>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Stmt {
    Assign { var: String, expr: AExp },
    IfGoto { cond: BExp, offset: isize },
    Goto { offset: isize },
    Call { name: String },
    Run { name: String, id_var: String },
    Yield,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AAtom {
    Var(String),
    Imm(Number),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ABinOp {
    Add,
    Sub,
    Mul,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AExp {
    Atom(AAtom),
    Bin {
        lhs: Box<AExp>,
        op: ABinOp,
        rhs: Box<AExp>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RelOp {
    Eq,
    Ne,
    Lt,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum BExp {
    Not(Box<BExp>),
    Rel {
        lhs: AExp,
        op: RelOp,
        rhs: AExp,
    },
    And(Box<BExp>, Box<BExp>),
    Or(Box<BExp>, Box<BExp>),
    Done(String),
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct GlobalEnv {
    pub vars: BTreeMap<String, Number>,
    pub task_ids: BTreeMap<String, Option<usize>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Frame {
    pub function: String,
    pub pc: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CoroutineMachine {
    pub code: CoroutineCode,
    pub env: GlobalEnv,
    pub workers: Vec<Option<usize>>,
    pub queue: VecDeque<usize>,
    pub tasks: Vec<Vec<Frame>>,
    #[serde(skip)]
    function_table: HashMap<String, FnDecl>,
}

impl GlobalEnv {
    fn get_var(&self, name: &str) -> Number {
        self.vars.get(name).cloned().unwrap_or_default()
    }

    fn set_var(&mut self, name: String, value: Number) {
        self.vars.insert(name, value);
    }

    fn get_task_id(&self, name: &str) -> Result<usize, String> {
        match self.task_ids.get(name) {
            Some(Some(id)) => Ok(*id),
            Some(None) => Err(format!("task id ${name} is not initialized")),
            None => Err(format!("unknown task id ${name}")),
        }
    }

    fn set_task_id(&mut self, name: String, value: usize) {
        self.task_ids.insert(name, Some(value));
    }
}

impl CoroutineMachine {
    fn eval_aexp(&self, exp: &AExp) -> Result<Number, String> {
        match exp {
            AExp::Atom(AAtom::Var(name)) => Ok(self.env.get_var(name)),
            AExp::Atom(AAtom::Imm(value)) => Ok(value.clone()),
            AExp::Bin { lhs, op, rhs } => {
                let lhs = self.eval_aexp(lhs)?;
                let rhs = self.eval_aexp(rhs)?;
                Ok(match op {
                    ABinOp::Add => lhs + rhs,
                    ABinOp::Sub => lhs - rhs,
                    ABinOp::Mul => Number::from(lhs.as_usize()? * rhs.as_usize()?),
                })
            }
        }
    }

    fn eval_bexp(&self, exp: &BExp) -> Result<bool, String> {
        match exp {
            BExp::Not(inner) => Ok(!self.eval_bexp(inner)?),
            BExp::Rel { lhs, op, rhs } => {
                let lhs = self.eval_aexp(lhs)?;
                let rhs = self.eval_aexp(rhs)?;
                Ok(match op {
                    RelOp::Eq => lhs == rhs,
                    RelOp::Ne => lhs != rhs,
                    RelOp::Lt => lhs < rhs,
                })
            }
            BExp::And(lhs, rhs) => Ok(self.eval_bexp(lhs)? && self.eval_bexp(rhs)?),
            BExp::Or(lhs, rhs) => Ok(self.eval_bexp(lhs)? || self.eval_bexp(rhs)?),
            BExp::Done(id_var) => {
                let task_id = self.env.get_task_id(id_var)?;
                Ok(self.tasks.get(task_id).is_some_and(|stack| stack.is_empty()))
            }
        }
    }

    fn current_frame(&self, task_id: usize) -> Result<&Frame, String> {
        self.tasks
            .get(task_id)
            .and_then(|stack| stack.last())
            .ok_or_else(|| format!("task {task_id} has no current frame"))
    }

    fn current_stmt(&self, task_id: usize) -> Result<Stmt, String> {
        let frame = self.current_frame(task_id)?;
        let function = self
            .function_table
            .get(&frame.function)
            .ok_or_else(|| format!("unknown function: {}", frame.function))?;
        function
            .body
            .get(frame.pc)
            .cloned()
            .ok_or_else(|| format!("pc {} out of bounds in {}", frame.pc, frame.function))
    }

    fn shift_pc(&mut self, task_id: usize, offset: isize) -> Result<(), String> {
        let stack = self
            .tasks
            .get_mut(task_id)
            .ok_or_else(|| format!("unknown task {task_id}"))?;
        let frame = stack
            .last_mut()
            .ok_or_else(|| format!("task {task_id} has empty stack"))?;
        let next = frame.pc as isize + offset;
        if next < 0 {
            return Err(format!(
                "pc became negative in {}: {} + {offset}",
                frame.function, frame.pc
            ));
        }
        frame.pc = next as usize;
        Ok(())
    }

    fn build_function_table(code: &CoroutineCode) -> Result<HashMap<String, FnDecl>, String> {
        let mut table = HashMap::new();
        for function in &code.0.functions {
            if table.insert(function.name.clone(), function.clone()).is_some() {
                return Err(format!("duplicate function: {}", function.name));
            }
        }
        if !table.contains_key("main") {
            return Err("missing main function".to_string());
        }
        Ok(table)
    }
}

impl Machine for CoroutineMachine {
    type Code = CoroutineCode;
    type AInput = usize;
    type FOutput = GlobalEnv;
    type SnapShot = CoroutineMachine;
    type RInput = usize;
    type ROutput = String;

    fn make(code: Self::Code, agent_count: Self::AInput) -> Result<Self, String> {
        let function_table = Self::build_function_table(&code)?;
        if agent_count == 0 {
            return Err("agent count must be positive".to_string());
        }
        Ok(Self {
            code,
            env: GlobalEnv::default(),
            workers: vec![None; agent_count],
            queue: VecDeque::from([0]),
            tasks: vec![vec![Frame {
                function: "main".to_string(),
                pc: 0,
            }]],
            function_table,
        })
    }

    fn step(mut self, agent: Self::RInput) -> Result<StepResult<Self>, String> {
        if agent >= self.workers.len() {
            return Err(format!("agent {agent} out of range"));
        }

        let output = match self.workers[agent] {
            None => {
                let Some(task_id) = self.queue.pop_front() else {
                    return Ok(StepResult::Halt { output: self.env });
                };
                self.workers[agent] = Some(task_id);
                format!("agent {agent}: dequeue task {task_id}")
            }
            Some(task_id) if self.tasks.get(task_id).is_some_and(|stack| stack.is_empty()) => {
                self.workers[agent] = None;
                format!("agent {agent}: release task {task_id}")
            }
            Some(task_id) => {
                let frame = self.current_frame(task_id)?.clone();
                let function = self
                    .function_table
                    .get(&frame.function)
                    .ok_or_else(|| format!("unknown function: {}", frame.function))?;
                if frame.pc == function.body.len() {
                    self.tasks
                        .get_mut(task_id)
                        .ok_or_else(|| format!("unknown task {task_id}"))?
                        .pop();
                    format!("agent {agent}: return from {}", frame.function)
                } else {
                    let stmt = self.current_stmt(task_id)?;
                    let text = coroutine_parser::stmt_to_text(&stmt);
                    match stmt {
                        Stmt::Assign { var, expr } => {
                            let value = self.eval_aexp(&expr)?;
                            self.env.set_var(var, value);
                            self.shift_pc(task_id, 1)?;
                        }
                        Stmt::IfGoto { cond, offset } => {
                            let delta = if self.eval_bexp(&cond)? { offset } else { 1 };
                            self.shift_pc(task_id, delta)?;
                        }
                        Stmt::Goto { offset } => {
                            self.shift_pc(task_id, offset)?;
                        }
                        Stmt::Call { name } => {
                            if !self.function_table.contains_key(&name) {
                                return Err(format!("unknown function: {name}"));
                            }
                            let stack = self
                                .tasks
                                .get_mut(task_id)
                                .ok_or_else(|| format!("unknown task {task_id}"))?;
                            let caller = stack
                                .last_mut()
                                .ok_or_else(|| format!("task {task_id} has empty stack"))?;
                            caller.pc += 1;
                            stack.push(Frame {
                                function: name,
                                pc: 0,
                            });
                        }
                        Stmt::Run { name, id_var } => {
                            if !self.function_table.contains_key(&name) {
                                return Err(format!("unknown function: {name}"));
                            }
                            let new_task_id = self.tasks.len();
                            self.env.set_task_id(id_var, new_task_id);
                            self.shift_pc(task_id, 1)?;
                            self.tasks.push(vec![Frame {
                                function: name,
                                pc: 0,
                            }]);
                            self.queue.push_back(new_task_id);
                        }
                        Stmt::Yield => {
                            self.shift_pc(task_id, 1)?;
                            self.workers[agent] = None;
                            self.queue.push_back(task_id);
                        }
                    }
                    format!("agent {agent}: {text}")
                }
            }
        };

        Ok(StepResult::Continue { next: self, output })
    }

    fn snapshot(&self) -> Self::SnapShot {
        self.clone()
    }

    fn restore(snapshot: Self::SnapShot) -> Self {
        let function_table = Self::build_function_table(&snapshot.code)
            .expect("snapshot should contain valid coroutine code");
        Self {
            function_table,
            ..snapshot
        }
    }

    fn render(snapshot: Self::SnapShot) -> utils::RenderState {
        coroutine_render::render_machine(snapshot)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use utils::TextCodec;

    #[test]
    fn run_and_done_work() {
        let code = CoroutineCode::parse(
            "fn main { run child -> $t; if done $t goto +2; yield; x <- 1 }\nfn child { y <- 1 }",
        )
        .unwrap();
        let machine = CoroutineMachine::make(code, 1).unwrap();

        let mut machine = machine;
        let output = loop {
            match machine.step(0).unwrap() {
                StepResult::Continue { next, .. } => machine = next,
                StepResult::Halt { output } => break output,
            }
        };

        assert_eq!(output.get_var("x"), Number::from(1usize));
        assert_eq!(output.get_var("y"), Number::from(1usize));
    }
}
