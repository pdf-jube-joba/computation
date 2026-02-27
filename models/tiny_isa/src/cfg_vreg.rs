use std::collections::HashMap;

use utils::number::Number;
use utils::{Machine, StepResult};

#[derive(Debug, Clone, Default)]
pub struct CfgVRegCode(pub Program);

#[derive(Debug, Clone, Default)]
pub struct Program {
    pub statics: Vec<StaticDef>,
    pub blocks: Vec<Block>,
}

#[derive(Debug, Clone)]
pub struct StaticDef {
    pub label: String,
    pub value: Number,
}

#[derive(Debug, Clone)]
pub struct Block {
    pub label: String,
    pub stmts: Vec<Stmt>,
    pub cont: Cont,
}

#[derive(Debug, Clone)]
pub struct Cont {
    pub ifs: Vec<JumpIf>,
    pub jump: AddrExpr,
}

#[derive(Debug, Clone)]
pub struct JumpIf {
    pub cond: Cond,
    pub target: AddrExpr,
}

#[derive(Debug, Clone)]
pub enum Cond {
    Lt(ValueExpr, ValueExpr),
    Eq(ValueExpr, ValueExpr),
    Gt(ValueExpr, ValueExpr),
}

#[derive(Debug, Clone)]
pub enum Stmt {
    Assign {
        dst: usize,
        src: ValueExpr,
    },
    BinOp {
        dst: usize,
        lhs: ValueExpr,
        op: BinOp,
        rhs: ValueExpr,
    },
    Store {
        place: PlaceExpr,
        src: ValueExpr,
    },
}

#[derive(Debug, Clone, Copy)]
pub enum BinOp {
    Add,
    Sub,
}

#[derive(Debug, Clone)]
pub enum AddrExpr {
    VReg(usize),
    Label(String),
    Imm(Number),
}

#[derive(Debug, Clone)]
pub struct PlaceExpr(pub Box<AddrExpr>);

#[derive(Debug, Clone)]
pub enum ValueExpr {
    VReg(usize),
    Imm(Number),
    Label(String),
    Deref(PlaceExpr),
}

#[derive(Debug, Clone)]
pub(crate) struct CompiledProgram {
    pub(crate) blocks: Vec<Block>,
    pub(crate) entry_block: usize,
    pub(crate) block_index: HashMap<String, usize>,
    pub(crate) static_labels: HashMap<String, usize>,
}

#[derive(Debug, Clone)]
pub struct CfgVRegMachine {
    pub code: CfgVRegCode,
    pub(crate) compiled: CompiledProgram,
    pub current_block: usize,
    pub vregs: Vec<Number>,
    pub memory: Vec<Number>,
    pub halted: bool,
}

impl CfgVRegMachine {
    fn output(&self) -> Vec<Number> {
        self.memory.clone()
    }

    fn continue_result(self) -> StepResult<Self> {
        StepResult::Continue {
            next: self,
            output: (),
        }
    }

    fn halt_result(self) -> StepResult<Self> {
        let snapshot = self.clone();
        let output = snapshot.output();
        StepResult::Halt { snapshot, output }
    }

    fn get_vreg(&self, idx: usize) -> Number {
        self.vregs.get(idx).cloned().unwrap_or_default()
    }

    fn set_vreg(&mut self, idx: usize, value: Number) {
        if idx >= self.vregs.len() {
            self.vregs.resize(idx + 1, Number::default());
        }
        self.vregs[idx] = value;
    }

    fn read_mem(&self, addr: usize) -> Number {
        self.memory.get(addr).cloned().unwrap_or_default()
    }

    fn write_mem(&mut self, addr: usize, value: Number) {
        if addr >= self.memory.len() {
            self.memory.resize(addr + 1, Number::default());
        }
        self.memory[addr] = value;
    }

    fn eval_addr(&self, addr: &AddrExpr) -> Result<Number, String> {
        match addr {
            AddrExpr::VReg(i) => Ok(self.get_vreg(*i)),
            AddrExpr::Label(name) => self
                .compiled
                .static_labels
                .get(name)
                .copied()
                .map(Number::from)
                .or_else(|| {
                    self.compiled
                        .block_index
                        .get(name)
                        .copied()
                        .map(Number::from)
                })
                .ok_or_else(|| format!("Unknown label: @{name}")),
            AddrExpr::Imm(n) => Ok(n.clone()),
        }
    }

    fn eval_place(&self, place: &PlaceExpr) -> Result<usize, String> {
        self.eval_addr(&place.0)?.as_usize()
    }

    fn eval_value(&self, value: &ValueExpr) -> Result<Number, String> {
        match value {
            ValueExpr::VReg(i) => Ok(self.get_vreg(*i)),
            ValueExpr::Imm(n) => Ok(n.clone()),
            ValueExpr::Label(name) => self.eval_addr(&AddrExpr::Label(name.clone())),
            ValueExpr::Deref(place) => {
                let addr = self.eval_place(place)?;
                Ok(self.read_mem(addr))
            }
        }
    }

    fn eval_cond(&self, cond: &Cond) -> Result<bool, String> {
        match cond {
            Cond::Lt(a, b) => Ok(self.eval_value(a)? < self.eval_value(b)?),
            Cond::Eq(a, b) => Ok(self.eval_value(a)? == self.eval_value(b)?),
            Cond::Gt(a, b) => Ok(self.eval_value(a)? > self.eval_value(b)?),
        }
    }

    fn jump_to(&mut self, target: &AddrExpr) -> Result<(), String> {
        let addr = self.eval_addr(target)?.as_usize()?;
        if addr >= self.compiled.blocks.len() {
            return Err(format!("Block address out of range: {addr}"));
        }
        self.current_block = addr;
        Ok(())
    }
}

impl Machine for CfgVRegMachine {
    type Code = CfgVRegCode;
    type AInput = Vec<Number>;
    type FOutput = Vec<Number>;
    type SnapShot = CfgVRegMachine;
    type RInput = ();
    type ROutput = ();

    fn make(code: Self::Code, ainput: Self::AInput) -> Result<Self, String> {
        let compiled = compile_program(&code.0)?;
        let mut memory = Vec::new();
        for s in &code.0.statics {
            memory.push(s.value.clone());
        }
        memory.extend(ainput);

        Ok(Self {
            code,
            current_block: compiled.entry_block,
            compiled,
            vregs: Vec::new(),
            memory,
            halted: false,
        })
    }

    fn step(self, _rinput: Self::RInput) -> Result<StepResult<Self>, String> {
        if self.halted {
            return Ok(self.halt_result());
        }

        let mut next = self;
        let block = next
            .compiled
            .blocks
            .get(next.current_block)
            .cloned()
            .ok_or_else(|| format!("Current block out of range: {}", next.current_block))?;

        for stmt in &block.stmts {
            match stmt {
                Stmt::Assign { dst, src } => {
                    let value = next.eval_value(src)?;
                    next.set_vreg(*dst, value);
                }
                Stmt::BinOp { dst, lhs, op, rhs } => {
                    let l = next.eval_value(lhs)?;
                    let r = next.eval_value(rhs)?;
                    let value = match op {
                        BinOp::Add => l + r,
                        BinOp::Sub => l - r,
                    };
                    next.set_vreg(*dst, value);
                }
                Stmt::Store { place, src } => {
                    let addr = next.eval_place(place)?;
                    let value = next.eval_value(src)?;
                    next.write_mem(addr, value);
                }
            }
        }

        for j in &block.cont.ifs {
            if next.eval_cond(&j.cond)? {
                next.jump_to(&j.target)?;
                return Ok(next.continue_result());
            }
        }

        next.jump_to(&block.cont.jump)?;
        Ok(next.continue_result())
    }

    fn current(&self) -> Self::SnapShot {
        self.clone()
    }
}

fn compile_program(program: &Program) -> Result<CompiledProgram, String> {
    let mut block_index = HashMap::new();
    for (i, block) in program.blocks.iter().enumerate() {
        if block_index.insert(block.label.clone(), i).is_some() {
            return Err(format!("Duplicate block label: @{}", block.label));
        }
    }
    let mut static_labels = HashMap::new();
    for (i, s) in program.statics.iter().enumerate() {
        if static_labels.insert(s.label.clone(), i).is_some() {
            return Err(format!("Duplicate static label: @{}", s.label));
        }
    }
    let entry_block = *block_index
        .get("main")
        .ok_or_else(|| "Missing entry block: @main".to_string())?;

    Ok(CompiledProgram {
        blocks: program.blocks.clone(),
        entry_block,
        block_index,
        static_labels,
    })
}
