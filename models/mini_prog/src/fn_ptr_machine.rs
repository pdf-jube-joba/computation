use std::collections::{BTreeMap, HashMap};

use serde::{Deserialize, Serialize};
use utils::{Machine, StepResult, TextCodec};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FnPtrCode(pub Program);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Program {
    pub functions: Vec<FnDecl>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FnDecl {
    pub name: String,
    pub params: Vec<String>,
    pub body: Vec<Stmt>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Stmt {
    Assign {
        place: PlaceExpr,
        value: ValueExpr,
    },
    Ifz {
        cond: ValueExpr,
        then_branch: Box<Stmt>,
        else_branch: Box<Stmt>,
    },
    Call {
        name: String,
        args: Vec<ValueExpr>,
    },
    Return,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PlaceExpr {
    Var(String),
    Deref(Box<ValueExpr>),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ValueExpr {
    Number(i64),
    NullPtr,
    BinOp {
        lhs: Box<ValueExpr>,
        op: BinOp,
        rhs: Box<ValueExpr>,
    },
    Load(Box<PlaceExpr>),
    Addr(Box<PlaceExpr>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BinOp {
    Add,
    Sub,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Location {
    pub frame_id: usize,
    pub slot: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Value {
    Number(i64),
    Location(Location),
    NullPtr,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FrameStore {
    pub frame_id: usize,
    pub values: BTreeMap<Location, Value>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CallFrame {
    pub env: BTreeMap<String, Location>,
    pub inner_k: Vec<Stmt>,
    pub function_name: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FnPtrMachine {
    pub code: FnPtrCode,
    pub current_function: String,
    pub current_stmt: Option<Stmt>,
    pub env: BTreeMap<String, Location>,
    pub store: Vec<FrameStore>,
    pub inner_k: Vec<Stmt>,
    pub fn_k: Vec<CallFrame>,
    pub next_frame_id: usize,
    #[serde(skip)]
    function_table: HashMap<String, FnDecl>,
}

impl TextCodec for Value {
    fn parse(text: &str) -> Result<Self, String> {
        let trimmed = text.trim();
        if trimmed == "#null-ptr" {
            return Ok(Self::NullPtr);
        }
        if let Some(rest) = trimmed
            .strip_prefix("loc(")
            .and_then(|s| s.strip_suffix(')'))
        {
            let (frame_id, slot) = rest
                .split_once(',')
                .ok_or_else(|| "expected loc(frame,slot)".to_string())?;
            return Ok(Self::Location(Location {
                frame_id: frame_id
                    .trim()
                    .parse::<usize>()
                    .map_err(|e| e.to_string())?,
                slot: slot.trim().parse::<usize>().map_err(|e| e.to_string())?,
            }));
        }
        Ok(Self::Number(
            trimmed.parse::<i64>().map_err(|e| e.to_string())?,
        ))
    }

    fn write_fmt(&self, f: &mut impl std::fmt::Write) -> std::fmt::Result {
        write!(f, "{self}")
    }
}

impl Machine for FnPtrMachine {
    type Code = FnPtrCode;
    type AInput = ();
    type FOutput = String;
    type SnapShot = FnPtrMachine;
    type RInput = ();
    type ROutput = String;

    fn make(code: Self::Code, _ainput: Self::AInput) -> Result<Self, String> {
        let function_table = build_function_table(&code.0)?;
        let main = function_table
            .get("main")
            .cloned()
            .ok_or_else(|| "function 'main' is missing".to_string())?;

        let (current_stmt, inner_k) = split_body(&main.body)?;
        let frame_id = 0;
        let mut values = BTreeMap::new();
        let mut env = BTreeMap::new();
        for (slot, param) in main.params.iter().enumerate() {
            let loc = Location { frame_id, slot };
            env.insert(param.clone(), loc);
            values.insert(loc, Value::Number(0));
        }

        Ok(Self {
            code,
            current_function: main.name,
            current_stmt: Some(current_stmt),
            env,
            store: vec![FrameStore { frame_id, values }],
            inner_k,
            fn_k: Vec::new(),
            next_frame_id: 1,
            function_table,
        })
    }

    fn step(mut self, _rinput: Self::RInput) -> Result<StepResult<Self>, String> {
        let Some(stmt) = self.current_stmt.clone() else {
            let next = self
                .inner_k
                .pop()
                .ok_or_else(|| "control reached end of function body without return".to_string())?;
            self.current_stmt = Some(next);
            return Ok(StepResult::Continue {
                next: self,
                output: String::new(),
            });
        };

        match stmt {
            Stmt::Assign { place, value } => {
                let loc = self.eval_place(&place)?;
                let value = self.eval_value(&value)?;
                self.store_assign(loc, value)?;
                self.current_stmt = None;
                Ok(StepResult::Continue {
                    next: self,
                    output: String::new(),
                })
            }
            Stmt::Ifz {
                cond,
                then_branch,
                else_branch,
            } => {
                let cond_value = self.eval_value(&cond)?;
                let selected = match cond_value {
                    Value::Number(0) => *then_branch,
                    Value::Number(_) => *else_branch,
                    other => return Err(format!("ifz expects a number, found {other}")),
                };
                self.current_stmt = Some(selected);
                Ok(StepResult::Continue {
                    next: self,
                    output: String::new(),
                })
            }
            Stmt::Call { name, args } => {
                let arg_values = args
                    .iter()
                    .map(|arg| self.eval_value(arg))
                    .collect::<Result<Vec<_>, _>>()?;
                let callee = self
                    .function_table
                    .get(&name)
                    .cloned()
                    .ok_or_else(|| format!("unknown function: {name}"))?;

                if arg_values.len() != callee.params.len() {
                    return Err(format!(
                        "argument count mismatch for {name}: expected {}, got {}",
                        callee.params.len(),
                        arg_values.len()
                    ));
                }

                self.fn_k.push(CallFrame {
                    env: self.env.clone(),
                    inner_k: self.inner_k.clone(),
                    function_name: self.current_function.clone(),
                });

                let frame_id = self.next_frame_id;
                self.next_frame_id += 1;

                let mut env = BTreeMap::new();
                let mut values = BTreeMap::new();
                for (slot, (param, value)) in callee.params.iter().zip(arg_values).enumerate() {
                    let loc = Location { frame_id, slot };
                    env.insert(param.clone(), loc);
                    values.insert(loc, value);
                }

                let (current_stmt, inner_k) = split_body(&callee.body)?;
                self.current_function = callee.name;
                self.current_stmt = Some(current_stmt);
                self.env = env;
                self.inner_k = inner_k;
                self.store.push(FrameStore { frame_id, values });

                Ok(StepResult::Continue {
                    next: self,
                    output: String::new(),
                })
            }
            Stmt::Return => {
                let finished_frame = self
                    .store
                    .pop()
                    .ok_or_else(|| "return on empty store stack".to_string())?;

                if let Some(frame) = self.fn_k.pop() {
                    self.current_function = frame.function_name;
                    self.env = frame.env;
                    self.inner_k = frame.inner_k;
                    self.current_stmt = None;
                    Ok(StepResult::Continue {
                        next: self,
                        output: String::new(),
                    })
                } else {
                    Ok(StepResult::Halt {
                        output: self.format_final_state(&finished_frame),
                    })
                }
            }
        }
    }

    fn snapshot(&self) -> Self::SnapShot {
        self.clone()
    }

    fn restore(mut snapshot: Self::SnapShot) -> Self {
        snapshot.function_table = build_function_table(&snapshot.code.0).unwrap_or_default();
        snapshot
    }

    fn render(snapshot: Self::SnapShot) -> utils::RenderState {
        crate::fn_ptr_render::render_machine(snapshot)
    }
}

impl FnPtrMachine {
    fn eval_place(&self, place: &PlaceExpr) -> Result<Location, String> {
        match place {
            PlaceExpr::Var(name) => self
                .env
                .get(name)
                .copied()
                .ok_or_else(|| format!("unknown variable: {name}")),
            PlaceExpr::Deref(value) => match self.eval_value(value)? {
                Value::Location(loc) => Ok(loc),
                Value::NullPtr => Err("cannot dereference #null-ptr".to_string()),
                Value::Number(_) => Err("dereference expects a location value".to_string()),
            },
        }
    }

    fn eval_value(&self, value: &ValueExpr) -> Result<Value, String> {
        match value {
            ValueExpr::Number(n) => Ok(Value::Number(*n)),
            ValueExpr::NullPtr => Ok(Value::NullPtr),
            ValueExpr::BinOp { lhs, op, rhs } => {
                let lhs = self.expect_number(self.eval_value(lhs)?)?;
                let rhs = self.expect_number(self.eval_value(rhs)?)?;
                Ok(Value::Number(match op {
                    BinOp::Add => lhs + rhs,
                    BinOp::Sub => lhs - rhs,
                }))
            }
            ValueExpr::Load(place) => {
                let loc = self.eval_place(place)?;
                self.store_lookup(loc)
            }
            ValueExpr::Addr(place) => Ok(Value::Location(self.eval_place(place)?)),
        }
    }

    fn expect_number(&self, value: Value) -> Result<i64, String> {
        match value {
            Value::Number(n) => Ok(n),
            other => Err(format!("expected a number, found {other}")),
        }
    }

    fn store_lookup(&self, loc: Location) -> Result<Value, String> {
        for frame in self.store.iter().rev() {
            if let Some(value) = frame.values.get(&loc) {
                return Ok(value.clone());
            }
        }
        Err(format!("unknown location: {loc}"))
    }

    fn store_assign(&mut self, loc: Location, value: Value) -> Result<(), String> {
        for frame in self.store.iter_mut().rev() {
            if let Some(slot) = frame.values.get_mut(&loc) {
                *slot = value;
                return Ok(());
            }
        }
        Err(format!("unknown location: {loc}"))
    }

    fn format_final_state(&self, finished_frame: &FrameStore) -> String {
        let mut out = String::new();
        out.push_str("halted\n");
        out.push_str(&format!("current_function: {}\n", self.current_function));
        out.push_str(&format!("finished_frame: {}\n", finished_frame.frame_id));
        for (loc, value) in &finished_frame.values {
            out.push_str(&format!("  {loc} = {value}\n"));
        }
        out.push_str("remaining_store:\n");
        if self.store.is_empty() {
            out.push_str("  <empty>\n");
        } else {
            for frame in &self.store {
                out.push_str(&format!("  frame {}:\n", frame.frame_id));
                for (loc, value) in &frame.values {
                    out.push_str(&format!("    {loc} = {value}\n"));
                }
            }
        }
        out
    }
}

impl std::fmt::Display for Program {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (index, function) in self.functions.iter().enumerate() {
            if index > 0 {
                writeln!(f)?;
                writeln!(f)?;
            }
            write!(f, "{function}")?;
        }
        Ok(())
    }
}

impl std::fmt::Display for FnDecl {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "fn {}(", self.name)?;
        for (index, param) in self.params.iter().enumerate() {
            if index > 0 {
                write!(f, ", ")?;
            }
            write!(f, "{param}")?;
        }
        write!(f, ") {{ ")?;
        for (index, stmt) in self.body.iter().enumerate() {
            if index > 0 {
                write!(f, "; ")?;
            }
            write!(f, "{stmt}")?;
        }
        write!(f, " }}")
    }
}

impl std::fmt::Display for Stmt {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Stmt::Assign { place, value } => write!(f, "assign {place} := {value}"),
            Stmt::Ifz {
                cond,
                then_branch,
                else_branch,
            } => write!(f, "ifz {cond} then {then_branch} else {else_branch} end"),
            Stmt::Call { name, args } => {
                write!(f, "call {name}(")?;
                for (index, arg) in args.iter().enumerate() {
                    if index > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{arg}")?;
                }
                write!(f, ")")
            }
            Stmt::Return => write!(f, "return"),
        }
    }
}

impl std::fmt::Display for PlaceExpr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PlaceExpr::Var(name) => write!(f, "{name}"),
            PlaceExpr::Deref(value) => write!(f, "({value}) #loc"),
        }
    }
}

impl std::fmt::Display for ValueExpr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ValueExpr::Number(n) => write!(f, "{n}"),
            ValueExpr::NullPtr => write!(f, "#null-ptr"),
            ValueExpr::BinOp { lhs, op, rhs } => write!(f, "({lhs} {op} {rhs})"),
            ValueExpr::Load(place) => write!(f, "ld {place}"),
            ValueExpr::Addr(place) => write!(f, "{place} #addr"),
        }
    }
}

impl std::fmt::Display for BinOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BinOp::Add => write!(f, "+"),
            BinOp::Sub => write!(f, "-"),
        }
    }
}

impl std::fmt::Display for Location {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "loc({}, {})", self.frame_id, self.slot)
    }
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Number(n) => write!(f, "{n}"),
            Value::Location(loc) => write!(f, "{loc}"),
            Value::NullPtr => write!(f, "#null-ptr"),
        }
    }
}

fn build_function_table(program: &Program) -> Result<HashMap<String, FnDecl>, String> {
    let mut table = HashMap::new();
    for function in &program.functions {
        if table
            .insert(function.name.clone(), function.clone())
            .is_some()
        {
            return Err(format!("duplicate function: {}", function.name));
        }
    }
    Ok(table)
}

fn split_body(body: &[Stmt]) -> Result<(Stmt, Vec<Stmt>), String> {
    let (first, rest) = body
        .split_first()
        .ok_or_else(|| "function body must not be empty".to_string())?;
    Ok((first.clone(), rest.iter().rev().cloned().collect()))
}
