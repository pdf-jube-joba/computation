use std::collections::{BTreeMap, HashMap};

use serde::{Deserialize, Serialize};
use utils::{Machine, StepResult};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MiniProgCode(pub Program);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Program {
    pub statics: Vec<StaticDecl>,
    pub functions: Vec<FnDecl>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StaticDecl {
    pub name: String,
    pub ty: Type,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FnDecl {
    pub name: String,
    pub block: Block,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Block {
    pub bindings: Vec<(String, Type)>,
    pub stmts: Vec<Stmt>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Type {
    Num,
    U8,
    Bool,
    Unit,
    Ptr,
    Fn,
    Product(Vec<Type>),
    Sum(Vec<Type>),
    Array(Box<Type>, usize),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PlaceExpr {
    Static(String),
    Local(String),
    Field(Box<PlaceExpr>, usize),
    Tag(Box<PlaceExpr>, usize),
    Index(Box<PlaceExpr>, Box<ValueExpr>),
    Deref(Box<ValueExpr>),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ValueExpr {
    Number(usize),
    Char(char),
    Bool(bool),
    Unit,
    NullPtr,
    NullFn,
    BinOp(Box<ValueExpr>, BinOp, Box<ValueExpr>),
    UnOp(UnOp, Box<ValueExpr>),
    Pair(Vec<ValueExpr>),
    Tag(usize, Box<ValueExpr>),
    Load(Box<PlaceExpr>),
    Addr(Box<PlaceExpr>),
    Fn(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BinOp {
    Add,
    Sub,
    Eq,
    Lt,
    And,
    Or,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum UnOp {
    Not,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Stmt {
    Assign { place: PlaceExpr, value: ValueExpr },
    If { cond: ValueExpr, stmt: Box<Stmt> },
    Case {
        tag: usize,
        value: ValueExpr,
        stmt: Box<Stmt>,
    },
    HAlloc { ty: Type, place: PlaceExpr },
    HFree { value: ValueExpr },
    Loop { label: String, stmt: Box<Stmt> },
    Break(String),
    Continue(String),
    Call {
        callee: ValueExpr,
        args: Vec<ValueExpr>,
        rets: Vec<PlaceExpr>,
    },
    Return(Vec<ValueExpr>),
    Block(Block),
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Location {
    pub key: LocationKey,
    pub projections: Vec<Projection>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum LocationKey {
    Heap(usize),
    Static(String),
    Local(usize),
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Projection {
    Field(usize),
    Tag(usize),
    Index(usize),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Value {
    Number(usize),
    Char(char),
    Bool(bool),
    Unit,
    Ptr(Option<Location>),
    FnPtr(Option<String>),
    Aggregate(Vec<Value>),
    Sum(usize, Box<Value>),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Cell {
    pub ty: Type,
    pub value: Value,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct Store {
    pub heap: BTreeMap<usize, Cell>,
    pub statics: BTreeMap<String, Cell>,
    pub locals: Vec<Vec<BTreeMap<usize, Cell>>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Control {
    Next,
    Break(String),
    Continue(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Exec {
    Eval(Stmt),
    Ctrl(Control),
    Ret(Vec<Value>),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum InnerFrame {
    Stmt(Stmt),
    Scope,
    Loop { label: String, stmt: Stmt },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FnFrame {
    pub function_name: String,
    pub env: Vec<BTreeMap<String, usize>>,
    pub ret_locs: Vec<Location>,
    pub inner_k: Vec<InnerFrame>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MiniProgMachine {
    pub code: MiniProgCode,
    pub current_function: String,
    pub exec: Exec,
    pub env: Vec<BTreeMap<String, usize>>,
    pub store: Store,
    pub inner_k: Vec<InnerFrame>,
    pub fn_k: Vec<FnFrame>,
    pub next_local_id: usize,
    pub next_heap_id: usize,
    #[serde(skip)]
    function_table: HashMap<String, FnDecl>,
}

impl Machine for MiniProgMachine {
    type Code = MiniProgCode;
    type AInput = Vec<usize>;
    type FOutput = usize;
    type SnapShot = MiniProgMachine;
    type RInput = ();
    type ROutput = String;

    fn make(code: Self::Code, ainput: Self::AInput) -> Result<Self, String> {
        let function_table = build_function_table(&code.0)?;
        let mut store = Store::default();
        for decl in &code.0.statics {
            store.statics.insert(
                decl.name.clone(),
                Cell {
                    ty: decl.ty.clone(),
                    value: default_value(&decl.ty)?,
                },
            );
        }

        let mut machine = Self {
            code,
            current_function: "main".to_string(),
            exec: Exec::Ctrl(Control::Next),
            env: Vec::new(),
            store,
            inner_k: Vec::new(),
            fn_k: Vec::new(),
            next_local_id: 0,
            next_heap_id: 0,
            function_table,
        };
        let args = ainput.into_iter().map(Value::Number).collect::<Vec<_>>();
        machine.enter_function("main", args)?;
        Ok(machine)
    }

    fn step(mut self, _rinput: Self::RInput) -> Result<StepResult<Self>, String> {
        match self.exec.clone() {
            Exec::Eval(stmt) => self.step_eval(stmt)?,
            Exec::Ctrl(control) => {
                if self.fn_k.is_empty()
                    && self.inner_k.is_empty()
                    && matches!(control, Control::Next)
                    && self.env.is_empty()
                {
                    return Err("main finished without return".to_string());
                }
                self.step_ctrl(control)?
            }
            Exec::Ret(values) => {
                if self.fn_k.is_empty() {
                    if values.len() != 1 {
                        return Err("main must return exactly one value".to_string());
                    }
                    match values.into_iter().next().unwrap() {
                        Value::Number(n) => return Ok(StepResult::Halt { output: n }),
                        _ => return Err("main must return a natural number".to_string()),
                    }
                }
                self.step_ret(values)?
            }
        }
        Ok(StepResult::Continue {
            next: self,
            output: String::new(),
        })
    }

    fn snapshot(&self) -> Self::SnapShot {
        self.clone()
    }

    fn restore(mut snapshot: Self::SnapShot) -> Self {
        snapshot.function_table = build_function_table(&snapshot.code.0).unwrap_or_default();
        snapshot
    }

    fn render(snapshot: Self::SnapShot) -> utils::RenderState {
        crate::mini_prog_render::render_machine(snapshot)
    }
}

impl MiniProgMachine {
    fn step_eval(&mut self, stmt: Stmt) -> Result<(), String> {
        match stmt {
            Stmt::Assign { place, value } => {
                let loc = self.eval_place(&place)?;
                let value = self.eval_value(&value)?;
                self.store_assign(&loc, value)?;
                self.exec = Exec::Ctrl(Control::Next);
            }
            Stmt::If { cond, stmt } => match self.eval_value(&cond)? {
                Value::Bool(true) => self.exec = Exec::Eval(*stmt),
                Value::Bool(false) => self.exec = Exec::Ctrl(Control::Next),
                _ => return Err("if expects a bool".to_string()),
            },
            Stmt::Case { tag, value, stmt } => match self.eval_value(&value)? {
                Value::Sum(found, _) if found == tag => self.exec = Exec::Eval(*stmt),
                Value::Sum(_, _) => self.exec = Exec::Ctrl(Control::Next),
                _ => return Err("case expects a tagged sum".to_string()),
            },
            Stmt::HAlloc { ty, place } => {
                let key = self.next_heap_id;
                self.next_heap_id += 1;
                self.store.heap.insert(
                    key,
                    Cell {
                        ty: ty.clone(),
                        value: default_value(&ty)?,
                    },
                );
                let ptr = Value::Ptr(Some(Location {
                    key: LocationKey::Heap(key),
                    projections: Vec::new(),
                }));
                let loc = self.eval_place(&place)?;
                self.store_assign(&loc, ptr)?;
                self.exec = Exec::Ctrl(Control::Next);
            }
            Stmt::HFree { value } => {
                let loc = match self.eval_value(&value)? {
                    Value::Ptr(Some(loc)) => loc,
                    _ => return Err("hfree expects a heap pointer".to_string()),
                };
                match loc.key {
                    LocationKey::Heap(key) if loc.projections.is_empty() => {
                        if self.store.heap.remove(&key).is_none() {
                            return Err("hfree on unknown heap location".to_string());
                        }
                    }
                    _ => return Err("hfree expects a top-level heap pointer".to_string()),
                }
                self.exec = Exec::Ctrl(Control::Next);
            }
            Stmt::Loop { label, stmt } => {
                self.inner_k.push(InnerFrame::Loop {
                    label,
                    stmt: (*stmt).clone(),
                });
                self.exec = Exec::Ctrl(Control::Next);
            }
            Stmt::Break(label) => self.exec = Exec::Ctrl(Control::Break(label)),
            Stmt::Continue(label) => self.exec = Exec::Ctrl(Control::Continue(label)),
            Stmt::Block(block) => {
                let values = block
                    .bindings
                    .iter()
                    .map(|(_, ty)| default_value(ty))
                    .collect::<Result<Vec<_>, _>>()?;
                self.push_scope(&block.bindings, values)?;
                self.inner_k.push(InnerFrame::Scope);
                self.push_stmt_sequence(&block.stmts);
                self.exec = Exec::Ctrl(Control::Next);
            }
            Stmt::Call { callee, args, rets } => {
                let ret_locs = rets
                    .iter()
                    .map(|place| self.eval_place(place))
                    .collect::<Result<Vec<_>, _>>()?;
                let callee_name = match self.eval_value(&callee)? {
                    Value::FnPtr(Some(name)) => name,
                    _ => return Err("call expects a function pointer".to_string()),
                };
                let arg_values = args
                    .iter()
                    .map(|arg| self.eval_value(arg))
                    .collect::<Result<Vec<_>, _>>()?;
                self.fn_k.push(FnFrame {
                    function_name: self.current_function.clone(),
                    env: self.env.clone(),
                    ret_locs,
                    inner_k: self.inner_k.clone(),
                });
                self.enter_function(&callee_name, arg_values)?;
                self.exec = Exec::Ctrl(Control::Next);
            }
            Stmt::Return(values) => {
                let values = values
                    .iter()
                    .map(|value| self.eval_value(value))
                    .collect::<Result<Vec<_>, _>>()?;
                self.exec = Exec::Ret(values);
            }
        }
        Ok(())
    }

    fn step_ctrl(&mut self, control: Control) -> Result<(), String> {
        let Some(frame) = self.inner_k.pop() else {
            return Err("control state has no continuation".to_string());
        };
        match frame {
            InnerFrame::Scope => {
                self.pop_scope()?;
                self.exec = Exec::Ctrl(control);
            }
            InnerFrame::Stmt(stmt) => match control {
                Control::Next => self.exec = Exec::Eval(stmt),
                Control::Break(label) => self.exec = Exec::Ctrl(Control::Break(label)),
                Control::Continue(label) => self.exec = Exec::Ctrl(Control::Continue(label)),
            },
            InnerFrame::Loop { label, stmt } => match control {
                Control::Next => {
                    self.inner_k.push(InnerFrame::Loop {
                        label,
                        stmt: stmt.clone(),
                    });
                    self.exec = Exec::Eval(stmt);
                }
                Control::Break(target) => {
                    if target == label {
                        self.exec = Exec::Ctrl(Control::Next);
                    } else {
                        self.exec = Exec::Ctrl(Control::Break(target));
                    }
                }
                Control::Continue(target) => {
                    if target == label {
                        self.inner_k.push(InnerFrame::Loop { label, stmt });
                        self.exec = Exec::Ctrl(Control::Next);
                    } else {
                        self.exec = Exec::Ctrl(Control::Break(target));
                    }
                }
            },
        }
        Ok(())
    }

    fn step_ret(&mut self, values: Vec<Value>) -> Result<(), String> {
        let frame = self
            .fn_k
            .pop()
            .ok_or_else(|| "return without caller".to_string())?;
        if values.len() != frame.ret_locs.len() {
            return Err(format!(
                "return count mismatch: expected {}, got {}",
                frame.ret_locs.len(),
                values.len()
            ));
        }

        self.store
            .locals
            .pop()
            .ok_or_else(|| "local store underflow on return".to_string())?;
        for (loc, value) in frame.ret_locs.iter().zip(values.into_iter()) {
            self.store_assign(loc, value)?;
        }
        self.current_function = frame.function_name;
        self.env = frame.env;
        self.inner_k = frame.inner_k;
        self.exec = Exec::Ctrl(Control::Next);
        Ok(())
    }

    fn enter_function(&mut self, name: &str, args: Vec<Value>) -> Result<(), String> {
        let function = self
            .function_table
            .get(name)
            .cloned()
            .ok_or_else(|| format!("unknown function: {name}"))?;
        if function.block.bindings.len() != args.len() {
            return Err(format!(
                "argument count mismatch for {name}: expected {}, got {}",
                function.block.bindings.len(),
                args.len()
            ));
        }

        let mut scope_cells = BTreeMap::new();
        let mut scope_env = BTreeMap::new();
        for ((var, ty), value) in function.block.bindings.iter().zip(args.into_iter()) {
            if !type_match(&value, ty) {
                return Err(format!("argument type mismatch for {var}"));
            }
            let local_id = self.next_local_id;
            self.next_local_id += 1;
            scope_env.insert(var.clone(), local_id);
            scope_cells.insert(
                local_id,
                Cell {
                    ty: ty.clone(),
                    value,
                },
            );
        }

        self.store.locals.push(vec![scope_cells]);
        self.env = vec![scope_env];
        self.inner_k.clear();
        self.push_stmt_sequence(&function.block.stmts);
        self.current_function = function.name;
        Ok(())
    }

    fn push_scope(
        &mut self,
        bindings: &[(String, Type)],
        values: Vec<Value>,
    ) -> Result<(), String> {
        if self.store.locals.is_empty() {
            return Err("push-scope without current function frame".to_string());
        }
        let mut scope_cells = BTreeMap::new();
        let mut scope_env = BTreeMap::new();
        for ((name, ty), value) in bindings.iter().zip(values.into_iter()) {
            if !type_match(&value, ty) {
                return Err(format!("type mismatch for {name}"));
            }
            let local_id = self.next_local_id;
            self.next_local_id += 1;
            scope_env.insert(name.clone(), local_id);
            scope_cells.insert(
                local_id,
                Cell {
                    ty: ty.clone(),
                    value,
                },
            );
        }
        self.env.push(scope_env);
        self.store.locals.last_mut().unwrap().push(scope_cells);
        Ok(())
    }

    fn pop_scope(&mut self) -> Result<(), String> {
        self.env
            .pop()
            .ok_or_else(|| "scope underflow in env".to_string())?;
        self.store
            .locals
            .last_mut()
            .ok_or_else(|| "scope underflow in locals".to_string())?
            .pop()
            .ok_or_else(|| "scope underflow in current function".to_string())?;
        Ok(())
    }

    fn push_stmt_sequence(&mut self, stmts: &[Stmt]) {
        for stmt in stmts.iter().rev() {
            self.inner_k.push(InnerFrame::Stmt(stmt.clone()));
        }
    }

    fn eval_place(&self, place: &PlaceExpr) -> Result<Location, String> {
        match place {
            PlaceExpr::Static(name) => {
                if !self.store.statics.contains_key(name) {
                    return Err(format!("unknown static: {name}"));
                }
                Ok(Location {
                    key: LocationKey::Static(name.clone()),
                    projections: Vec::new(),
                })
            }
            PlaceExpr::Local(name) => {
                for scope in self.env.iter().rev() {
                    if let Some(local_id) = scope.get(name) {
                        return Ok(Location {
                            key: LocationKey::Local(*local_id),
                            projections: Vec::new(),
                        });
                    }
                }
                Err(format!("unknown local: {name}"))
            }
            PlaceExpr::Field(place, index) => {
                let mut loc = self.eval_place(place)?;
                loc.projections.push(Projection::Field(*index));
                Ok(loc)
            }
            PlaceExpr::Tag(place, tag) => {
                let mut loc = self.eval_place(place)?;
                loc.projections.push(Projection::Tag(*tag));
                Ok(loc)
            }
            PlaceExpr::Index(place, value) => {
                let idx = match self.eval_value(value)? {
                    Value::Number(n) => n,
                    _ => return Err("array index must be a number".to_string()),
                };
                let mut loc = self.eval_place(place)?;
                loc.projections.push(Projection::Index(idx));
                Ok(loc)
            }
            PlaceExpr::Deref(value) => match self.eval_value(value)? {
                Value::Ptr(Some(loc)) => Ok(loc),
                Value::Ptr(None) => Err("cannot dereference #null-ptr".to_string()),
                _ => Err("dereference expects a pointer".to_string()),
            },
        }
    }

    fn eval_value(&self, value: &ValueExpr) -> Result<Value, String> {
        match value {
            ValueExpr::Number(n) => Ok(Value::Number(*n)),
            ValueExpr::Char(c) => Ok(Value::Char(*c)),
            ValueExpr::Bool(b) => Ok(Value::Bool(*b)),
            ValueExpr::Unit => Ok(Value::Unit),
            ValueExpr::NullPtr => Ok(Value::Ptr(None)),
            ValueExpr::NullFn => Ok(Value::FnPtr(None)),
            ValueExpr::BinOp(lhs, op, rhs) => {
                let lhs = self.eval_value(lhs)?;
                let rhs = self.eval_value(rhs)?;
                self.eval_binop(*op, lhs, rhs)
            }
            ValueExpr::UnOp(op, expr) => {
                let expr = self.eval_value(expr)?;
                self.eval_unop(*op, expr)
            }
            ValueExpr::Pair(values) => Ok(Value::Aggregate(
                values
                    .iter()
                    .map(|value| self.eval_value(value))
                    .collect::<Result<Vec<_>, _>>()?,
            )),
            ValueExpr::Tag(tag, value) => Ok(Value::Sum(*tag, Box::new(self.eval_value(value)?))),
            ValueExpr::Load(place) => {
                let loc = self.eval_place(place)?;
                Ok(self.lookup_location(&loc)?.1)
            }
            ValueExpr::Addr(place) => Ok(Value::Ptr(Some(self.eval_place(place)?))),
            ValueExpr::Fn(name) => {
                if self.function_table.contains_key(name) {
                    Ok(Value::FnPtr(Some(name.clone())))
                } else {
                    Err(format!("unknown function: {name}"))
                }
            }
        }
    }

    fn eval_binop(&self, op: BinOp, lhs: Value, rhs: Value) -> Result<Value, String> {
        match op {
            BinOp::Add => match (lhs, rhs) {
                (Value::Number(l), Value::Number(r)) => Ok(Value::Number(l + r)),
                _ => Err("operator '+' expects numbers".to_string()),
            },
            BinOp::Sub => match (lhs, rhs) {
                (Value::Number(l), Value::Number(r)) => l
                    .checked_sub(r)
                    .map(Value::Number)
                    .ok_or_else(|| "operator '-' underflowed".to_string()),
                _ => Err("operator '-' expects numbers".to_string()),
            },
            BinOp::Eq => match (lhs, rhs) {
                (Value::Number(l), Value::Number(r)) => Ok(Value::Bool(l == r)),
                (Value::Char(l), Value::Char(r)) => Ok(Value::Bool(l == r)),
                (Value::Bool(l), Value::Bool(r)) => Ok(Value::Bool(l == r)),
                (Value::Unit, Value::Unit) => Ok(Value::Bool(true)),
                (Value::Ptr(l), Value::Ptr(r)) => Ok(Value::Bool(l == r)),
                (Value::FnPtr(l), Value::FnPtr(r)) => Ok(Value::Bool(l == r)),
                _ => Err("operator '==' expects primitive, ptr, or fn values".to_string()),
            },
            BinOp::Lt => match (lhs, rhs) {
                (Value::Number(l), Value::Number(r)) => Ok(Value::Bool(l < r)),
                _ => Err("operator '<' expects numbers".to_string()),
            },
            BinOp::And => match (lhs, rhs) {
                (Value::Bool(l), Value::Bool(r)) => Ok(Value::Bool(l && r)),
                _ => Err("operator '&&' expects bools".to_string()),
            },
            BinOp::Or => match (lhs, rhs) {
                (Value::Bool(l), Value::Bool(r)) => Ok(Value::Bool(l || r)),
                _ => Err("operator '||' expects bools".to_string()),
            },
        }
    }

    fn eval_unop(&self, op: UnOp, value: Value) -> Result<Value, String> {
        match op {
            UnOp::Not => match value {
                Value::Bool(value) => Ok(Value::Bool(!value)),
                _ => Err("operator '!' expects a bool".to_string()),
            },
        }
    }

    fn lookup_location(&self, loc: &Location) -> Result<(Type, Value), String> {
        let cell = self.get_cell_by_key(&loc.key)?;
        project_cell(&cell.ty, &cell.value, &loc.projections)
    }

    fn store_assign(&mut self, loc: &Location, value: Value) -> Result<(), String> {
        let cell = self.get_cell_by_key(&loc.key)?;
        let updated = update_projection(&cell.ty, &cell.value, &loc.projections, value)?;
        self.set_cell_by_key(
            &loc.key,
            Cell {
                ty: cell.ty,
                value: updated,
            },
        )
    }

    fn get_cell_by_key(&self, key: &LocationKey) -> Result<Cell, String> {
        match key {
            LocationKey::Heap(id) => self
                .store
                .heap
                .get(id)
                .cloned()
                .ok_or_else(|| format!("unknown heap location: {id}")),
            LocationKey::Static(name) => self
                .store
                .statics
                .get(name)
                .cloned()
                .ok_or_else(|| format!("unknown static location: {name}")),
            LocationKey::Local(id) => {
                for function_scopes in self.store.locals.iter().rev() {
                    for scope in function_scopes.iter().rev() {
                        if let Some(cell) = scope.get(id) {
                            return Ok(cell.clone());
                        }
                    }
                }
                Err(format!("unknown local location: {id}"))
            }
        }
    }

    fn set_cell_by_key(&mut self, key: &LocationKey, cell: Cell) -> Result<(), String> {
        match key {
            LocationKey::Heap(id) => self
                .store
                .heap
                .get_mut(id)
                .map(|slot| *slot = cell)
                .ok_or_else(|| format!("unknown heap location: {id}")),
            LocationKey::Static(name) => self
                .store
                .statics
                .get_mut(name)
                .map(|slot| *slot = cell)
                .ok_or_else(|| format!("unknown static location: {name}")),
            LocationKey::Local(id) => {
                for function_scopes in self.store.locals.iter_mut().rev() {
                    for scope in function_scopes.iter_mut().rev() {
                        if let Some(slot) = scope.get_mut(id) {
                            *slot = cell;
                            return Ok(());
                        }
                    }
                }
                Err(format!("unknown local location: {id}"))
            }
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

fn type_match(value: &Value, ty: &Type) -> bool {
    match (value, ty) {
        (Value::Number(_), Type::Num) => true,
        (Value::Number(n), Type::U8) => *n <= u8::MAX as usize,
        (Value::Bool(_), Type::Bool) => true,
        (Value::Unit, Type::Unit) => true,
        (Value::Ptr(_), Type::Ptr) => true,
        (Value::FnPtr(_), Type::Fn) => true,
        (Value::Aggregate(values), Type::Product(types)) => {
            values.len() == types.len()
                && values
                    .iter()
                    .zip(types.iter())
                    .all(|(value, ty)| type_match(value, ty))
        }
        (Value::Aggregate(values), Type::Array(ty, len)) => {
            values.len() == *len && values.iter().all(|value| type_match(value, ty))
        }
        (Value::Sum(tag, value), Type::Sum(types)) => types
            .get(*tag)
            .map(|ty| type_match(value, ty))
            .unwrap_or(false),
        _ => false,
    }
}

fn default_value(ty: &Type) -> Result<Value, String> {
    Ok(match ty {
        Type::Num | Type::U8 => Value::Number(0),
        Type::Bool => Value::Bool(false),
        Type::Unit => Value::Unit,
        Type::Ptr => Value::Ptr(None),
        Type::Fn => Value::FnPtr(None),
        Type::Product(types) => Value::Aggregate(
            types
                .iter()
                .map(default_value)
                .collect::<Result<Vec<_>, _>>()?,
        ),
        Type::Array(ty, len) => Value::Aggregate(
            (0..*len)
                .map(|_| default_value(ty))
                .collect::<Result<Vec<_>, _>>()?,
        ),
        Type::Sum(types) => {
            let first = types
                .first()
                .ok_or_else(|| "sum type must have at least one variant".to_string())?;
            Value::Sum(0, Box::new(default_value(first)?))
        }
    })
}

fn project_cell(ty: &Type, value: &Value, projections: &[Projection]) -> Result<(Type, Value), String> {
    if projections.is_empty() {
        return Ok((ty.clone(), value.clone()));
    }
    match (&projections[0], ty, value) {
        (Projection::Field(index), Type::Product(types), Value::Aggregate(values)) => {
            let field_ty = types
                .get(*index)
                .ok_or_else(|| "projection out of bounds".to_string())?;
            let field_value = values
                .get(*index)
                .ok_or_else(|| "projection out of bounds".to_string())?;
            project_cell(field_ty, field_value, &projections[1..])
        }
        (Projection::Index(index), Type::Array(elem, len), Value::Aggregate(values)) => {
            if *index >= *len {
                return Err("projection out of bounds".to_string());
            }
            let field_value = values
                .get(*index)
                .ok_or_else(|| "projection out of bounds".to_string())?;
            project_cell(elem, field_value, &projections[1..])
        }
        (Projection::Tag(index), Type::Sum(types), Value::Sum(found, value)) => {
            if *found != *index {
                return Err("sum tag mismatch".to_string());
            }
            let variant_ty = types
                .get(*index)
                .ok_or_else(|| "sum tag out of bounds".to_string())?;
            project_cell(variant_ty, value, &projections[1..])
        }
        _ => Err("invalid place projection".to_string()),
    }
}

fn update_projection(
    ty: &Type,
    current: &Value,
    projections: &[Projection],
    new_value: Value,
) -> Result<Value, String> {
    if projections.is_empty() {
        if type_match(&new_value, ty) {
            return Ok(new_value);
        }
        return Err("assigned value does not match location type".to_string());
    }

    match (&projections[0], ty, current) {
        (Projection::Field(index), Type::Product(types), Value::Aggregate(values)) => {
            let mut values = values.clone();
            let field_ty = types
                .get(*index)
                .ok_or_else(|| "field projection out of bounds".to_string())?;
            let field_value = values
                .get(*index)
                .cloned()
                .ok_or_else(|| "field projection out of bounds".to_string())?;
            values[*index] =
                update_projection(field_ty, &field_value, &projections[1..], new_value)?;
            Ok(Value::Aggregate(values))
        }
        (Projection::Index(index), Type::Array(elem_ty, len), Value::Aggregate(values)) => {
            if *index >= *len {
                return Err("array index out of bounds".to_string());
            }
            let mut values = values.clone();
            let field_value = values
                .get(*index)
                .cloned()
                .ok_or_else(|| "array index out of bounds".to_string())?;
            values[*index] =
                update_projection(elem_ty, &field_value, &projections[1..], new_value)?;
            Ok(Value::Aggregate(values))
        }
        (Projection::Tag(index), Type::Sum(types), Value::Sum(found, value)) => {
            if *found != *index {
                return Err("sum tag mismatch".to_string());
            }
            let variant_ty = types
                .get(*index)
                .ok_or_else(|| "sum tag out of bounds".to_string())?;
            Ok(Value::Sum(
                *found,
                Box::new(update_projection(variant_ty, value, &projections[1..], new_value)?),
            ))
        }
        _ => Err("invalid assignment target".to_string()),
    }
}

impl std::fmt::Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Type::Num => write!(f, "#num"),
            Type::U8 => write!(f, "#u8"),
            Type::Bool => write!(f, "#bool"),
            Type::Unit => write!(f, "#unit"),
            Type::Ptr => write!(f, "#ptr"),
            Type::Fn => write!(f, "#fn"),
            Type::Product(types) => {
                for (index, ty) in types.iter().enumerate() {
                    if index > 0 {
                        write!(f, " * ")?;
                    }
                    write!(f, "{ty}")?;
                }
                Ok(())
            }
            Type::Sum(types) => {
                for (index, ty) in types.iter().enumerate() {
                    if index > 0 {
                        write!(f, " + ")?;
                    }
                    write!(f, "{ty}")?;
                }
                Ok(())
            }
            Type::Array(ty, len) => write!(f, "[{ty}; {len}]"),
        }
    }
}

impl std::fmt::Display for PlaceExpr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PlaceExpr::Static(name) => write!(f, "static {name}"),
            PlaceExpr::Local(name) => write!(f, "local {name}"),
            PlaceExpr::Field(place, index) => write!(f, "{place}.{index}"),
            PlaceExpr::Tag(place, tag) => write!(f, "{place}?{tag}"),
            PlaceExpr::Index(place, value) => write!(f, "{place}[{value}]"),
            PlaceExpr::Deref(value) => write!(f, "({value}) #loc"),
        }
    }
}

impl std::fmt::Display for ValueExpr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ValueExpr::Number(n) => write!(f, "{n}"),
            ValueExpr::Char(c) => write!(f, "'{c}'"),
            ValueExpr::Bool(true) => write!(f, "#true"),
            ValueExpr::Bool(false) => write!(f, "#false"),
            ValueExpr::Unit => write!(f, "#unit"),
            ValueExpr::NullPtr => write!(f, "#null-ptr"),
            ValueExpr::NullFn => write!(f, "#null-fn"),
            ValueExpr::BinOp(lhs, op, rhs) => write!(f, "{lhs} {rhs} {op}"),
            ValueExpr::UnOp(op, expr) => write!(f, "{expr} {op}"),
            ValueExpr::Pair(values) => {
                for (index, value) in values.iter().enumerate() {
                    if index > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{value}")?;
                }
                write!(f, " pair({})", values.len())
            }
            ValueExpr::Tag(tag, value) => write!(f, "{value} tag({tag})"),
            ValueExpr::Load(place) => write!(f, "ld {place}"),
            ValueExpr::Addr(place) => write!(f, "{place} #addr"),
            ValueExpr::Fn(name) => write!(f, "fn {name}"),
        }
    }
}

impl std::fmt::Display for BinOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BinOp::Add => write!(f, "+"),
            BinOp::Sub => write!(f, "-"),
            BinOp::Eq => write!(f, "=="),
            BinOp::Lt => write!(f, "<"),
            BinOp::And => write!(f, "&&"),
            BinOp::Or => write!(f, "||"),
        }
    }
}

impl std::fmt::Display for UnOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UnOp::Not => write!(f, "!"),
        }
    }
}

impl std::fmt::Display for Stmt {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Stmt::Assign { place, value } => write!(f, "assign {place} := {value}"),
            Stmt::If { cond, stmt } => write!(f, "if {cond} {stmt}"),
            Stmt::Case { tag, value, stmt } => write!(f, "case {tag} of {value} {stmt}"),
            Stmt::HAlloc { ty, place } => write!(f, "halloc {ty} -> {place}"),
            Stmt::HFree { value } => write!(f, "hfree {value}"),
            Stmt::Loop { label, stmt } => write!(f, "loop {label}: {stmt}"),
            Stmt::Break(label) => write!(f, "break {label}"),
            Stmt::Continue(label) => write!(f, "continue {label}"),
            Stmt::Call { callee, args, rets } => {
                write!(f, "call {callee}(")?;
                for (index, arg) in args.iter().enumerate() {
                    if index > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{arg}")?;
                }
                write!(f, ") -> ")?;
                for (index, ret) in rets.iter().enumerate() {
                    if index > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{ret}")?;
                }
                Ok(())
            }
            Stmt::Return(values) => {
                write!(f, "return")?;
                if !values.is_empty() {
                    write!(f, " ")?;
                }
                for (index, value) in values.iter().enumerate() {
                    if index > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{value}")?;
                }
                Ok(())
            }
            Stmt::Block(block) => write!(f, "block {block}"),
        }
    }
}

impl std::fmt::Display for Block {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "(")?;
        for (index, (name, ty)) in self.bindings.iter().enumerate() {
            if index > 0 {
                write!(f, ", ")?;
            }
            write!(f, "{name}: {ty}")?;
        }
        write!(f, ") {{ ")?;
        for (index, stmt) in self.stmts.iter().enumerate() {
            if index > 0 {
                write!(f, "; ")?;
            }
            write!(f, "{stmt}")?;
        }
        write!(f, " }}")
    }
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Number(n) => write!(f, "{n}"),
            Value::Char(c) => write!(f, "'{c}'"),
            Value::Bool(true) => write!(f, "#true"),
            Value::Bool(false) => write!(f, "#false"),
            Value::Unit => write!(f, "#unit"),
            Value::Ptr(None) => write!(f, "#null-ptr"),
            Value::Ptr(Some(loc)) => write!(f, "{loc}"),
            Value::FnPtr(None) => write!(f, "#null-fn"),
            Value::FnPtr(Some(name)) => write!(f, "fn {name}"),
            Value::Aggregate(values) => {
                write!(f, "(")?;
                for (index, value) in values.iter().enumerate() {
                    if index > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{value}")?;
                }
                write!(f, ")")
            }
            Value::Sum(tag, value) => write!(f, "tag({tag}, {value})"),
        }
    }
}

impl std::fmt::Display for Location {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.key {
            LocationKey::Heap(id) => write!(f, "heap#{id}")?,
            LocationKey::Static(name) => write!(f, "static {name}")?,
            LocationKey::Local(id) => write!(f, "local#{id}")?,
        }
        for proj in &self.projections {
            match proj {
                Projection::Field(index) => write!(f, ".{index}")?,
                Projection::Tag(tag) => write!(f, "?{tag}")?,
                Projection::Index(index) => write!(f, "[{index}]")?,
            }
        }
        Ok(())
    }
}
