use std::collections::{BTreeMap, HashMap, HashSet};

use serde::{Deserialize, Serialize};
use utils::number::Number;
use utils::{Machine, StepResult};

pub type Vreg = String;

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct FlowIrCode(pub Program);

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Program {
    pub statics: Vec<StaticDef>,
    pub regions: Vec<Region>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StaticDef {
    pub label: String,
    pub value: Number,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Region {
    pub label: String,
    pub blocks: Vec<Block>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Block {
    pub label: String,
    pub stmts: Vec<Stmt>,
    pub cont: Cont,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct JumpIf {
    pub cond: Cond,
    pub target: ValueExpr,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Cont {
    Go { ifs: Vec<JumpIf>, target: ValueExpr },
    Enter { ifs: Vec<JumpIf>, target: ValueExpr },
    Halt { ifs: Vec<JumpIf> },
}

impl Cont {
    pub fn ifs(&self) -> &[JumpIf] {
        match self {
            Cont::Go { ifs, .. } => ifs,
            Cont::Enter { ifs, .. } => ifs,
            Cont::Halt { ifs } => ifs,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Cond {
    Lt(ValueExpr, ValueExpr),
    Eq(ValueExpr, ValueExpr),
    Gt(ValueExpr, ValueExpr),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Stmt {
    Nop,
    Load {
        dst: Vreg,
        place: PlaceExpr,
    },
    Assign {
        dst: Vreg,
        src: ValueExpr,
    },
    BinOp {
        dst: Vreg,
        lhs: ValueExpr,
        op: BinOp,
        rhs: ValueExpr,
    },
    Store {
        place: PlaceExpr,
        src: ValueExpr,
    },
    Pop {
        dst: Vreg,
    },
    Push {
        src: ValueExpr,
    },
    LGet {
        dst: Vreg,
    },
    HAlloc {
        size: ValueExpr,
        dst: Vreg,
    },
    HFree {
        handle: ValueExpr,
    },
    Print {
        src: Vreg,
    },
    Input {
        place: PlaceExpr,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BinOp {
    Add,
    Sub,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ValueExpr {
    VReg(Vreg),
    Imm(Number),
    CodeLabel(String),
    Ref(PlaceExpr),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PlaceExpr {
    Label(String),
    Deref(Vreg),
    SAcc(Box<ValueExpr>),
    HAcc {
        handle: Box<ValueExpr>,
        index: Box<ValueExpr>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PlaceKey {
    Static(String),
    Stack(usize),
    Heap { handle: usize, index: usize },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum FlowValue {
    Num(Number),
    CodeLabel(String),
    Key(PlaceKey),
}

impl Default for FlowValue {
    fn default() -> Self {
        Self::Num(Number::default())
    }
}

impl FlowValue {
    pub fn as_number(&self) -> Result<Number, String> {
        match self {
            FlowValue::Num(n) => Ok(n.clone()),
            _ => Err(format!("Expected number, got {}", self.print())),
        }
    }

    pub fn as_usize(&self) -> Result<usize, String> {
        self.as_number()?.as_usize()
    }

    pub fn print(&self) -> String {
        match self {
            FlowValue::Num(n) => n.to_decimal_string(),
            FlowValue::CodeLabel(l) => format!(":{l}"),
            FlowValue::Key(k) => match k {
                PlaceKey::Static(l) => format!("k:@{l}"),
                PlaceKey::Stack(i) => format!("k:s[{i}]"),
                PlaceKey::Heap { handle, index } => format!("k:h[{handle}][{index}]"),
            },
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NameMapping {
    pub static_labels: HashSet<String>,
    pub region_label_to_id: HashMap<String, usize>,
    pub block_labels_in_region: Vec<HashMap<String, usize>>,
    pub entry_region: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlowIrMachine {
    pub code: FlowIrCode,
    pub names: NameMapping,
    pub current_region: usize,
    pub current_block: usize,
    pub current_line: usize,
    pub vregs: BTreeMap<Vreg, FlowValue>,
    pub static_mem: HashMap<String, FlowValue>,
    pub stack: Vec<FlowValue>,
    pub heap: HashMap<usize, Vec<FlowValue>>,
    pub next_handle: usize,
    pub halted: bool,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct StaticEnv {
    pub entries: BTreeMap<String, FlowValue>,
}

impl FlowIrMachine {
    fn output(&self) -> StaticEnv {
        let mut entries = BTreeMap::new();
        for (label, value) in &self.static_mem {
            entries.insert(label.clone(), value.clone());
        }
        StaticEnv { entries }
    }

    fn continue_result(self, output: String) -> StepResult<Self> {
        StepResult::Continue { next: self, output }
    }

    fn halt_result(self) -> StepResult<Self> {
        let output = self.output();
        StepResult::Halt { output }
    }

    fn get_vreg(&self, idx: Vreg) -> FlowValue {
        self.vregs.get(&idx).cloned().unwrap_or_default()
    }

    fn set_vreg(&mut self, idx: Vreg, value: FlowValue) {
        self.vregs.insert(idx, value);
    }

    fn reset_vregs(&mut self) {
        self.vregs.clear();
    }

    fn read_place(&self, place: &PlaceKey) -> Result<FlowValue, String> {
        match place {
            PlaceKey::Static(label) => self
                .static_mem
                .get(label)
                .cloned()
                .ok_or_else(|| format!("Unknown data label: @{label}")),
            PlaceKey::Stack(i) => self
                .stack
                .get(*i)
                .cloned()
                .ok_or_else(|| format!("Invalid stack key: s[{i}]")),
            PlaceKey::Heap { handle, index } => self
                .heap
                .get(handle)
                .and_then(|area| area.get(*index))
                .cloned()
                .ok_or_else(|| format!("Invalid heap key: h[{handle}][{index}]")),
        }
    }

    fn write_place(&mut self, place: &PlaceKey, value: FlowValue) -> Result<(), String> {
        match place {
            PlaceKey::Static(label) => {
                let slot = self
                    .static_mem
                    .get_mut(label)
                    .ok_or_else(|| format!("Unknown data label: @{label}"))?;
                *slot = value;
                Ok(())
            }
            PlaceKey::Stack(i) => {
                if *i >= self.stack.len() {
                    return Err(format!("Invalid stack key: s[{i}]"));
                }
                self.stack[*i] = value;
                Ok(())
            }
            PlaceKey::Heap { handle, index } => {
                let area = self
                    .heap
                    .get_mut(handle)
                    .ok_or_else(|| format!("Unknown heap handle: {handle}"))?;
                if *index >= area.len() {
                    return Err(format!("Invalid heap key: h[{handle}][{index}]"));
                }
                area[*index] = value;
                Ok(())
            }
        }
    }

    fn eval_place_key(&self, place: &PlaceExpr) -> Result<PlaceKey, String> {
        match place {
            PlaceExpr::Label(label) => {
                if self.names.static_labels.contains(label) {
                    Ok(PlaceKey::Static(label.clone()))
                } else {
                    Err(format!("Place is not data label: @{label}"))
                }
            }
            PlaceExpr::Deref(vreg) => match self.get_vreg(vreg.clone()) {
                FlowValue::Key(k) => Ok(k),
                other => Err(format!(
                    "deref expects place key in vreg, got {}",
                    other.print()
                )),
            },
            PlaceExpr::SAcc(index) => {
                let i = self.eval_value(index)?.as_usize()?;
                Ok(PlaceKey::Stack(i))
            }
            PlaceExpr::HAcc { handle, index } => {
                let h = self.eval_value(handle)?.as_usize()?;
                let i = self.eval_value(index)?.as_usize()?;
                Ok(PlaceKey::Heap {
                    handle: h,
                    index: i,
                })
            }
        }
    }

    fn eval_value(&self, value: &ValueExpr) -> Result<FlowValue, String> {
        match value {
            ValueExpr::VReg(i) => Ok(self.get_vreg(i.clone())),
            ValueExpr::Imm(n) => Ok(FlowValue::Num(n.clone())),
            ValueExpr::CodeLabel(l) => Ok(FlowValue::CodeLabel(l.clone())),
            ValueExpr::Ref(place) => Ok(FlowValue::Key(self.eval_place_key(place)?)),
        }
    }

    fn eval_cond(&self, cond: &Cond) -> Result<bool, String> {
        let (a, b, op) = match cond {
            Cond::Lt(a, b) => (a, b, "lt"),
            Cond::Eq(a, b) => (a, b, "eq"),
            Cond::Gt(a, b) => (a, b, "gt"),
        };
        let left = self.eval_value(a)?;
        let right = self.eval_value(b)?;
        match (left, right) {
            (FlowValue::Num(l), FlowValue::Num(r)) => Ok(match op {
                "lt" => l < r,
                "eq" => l == r,
                "gt" => l > r,
                _ => false,
            }),
            (l, r) if op == "eq" => Ok(l == r),
            (l, r) => Err(format!(
                "Condition {op} supports only numbers, got {} and {}",
                l.print(),
                r.print()
            )),
        }
    }

    fn eval_target_label(&self, target: &ValueExpr) -> Result<String, String> {
        match self.eval_value(target)? {
            FlowValue::CodeLabel(label) => Ok(label),
            other => Err(format!(
                "jump target expects code label, got {}",
                other.print()
            )),
        }
    }

    fn jump_goto(&mut self, target: &ValueExpr) -> Result<(), String> {
        let label = self.eval_target_label(target)?;
        if let Some(&target_block) = self
            .names
            .block_labels_in_region
            .get(self.current_region)
            .and_then(|m| m.get(&label))
        {
            self.current_block = target_block;
            self.current_line = 0;
            return Ok(());
        }
        Err(format!(
            "goto target must be local block label, got :{label}"
        ))
    }

    fn jump_enter(&mut self, target: &ValueExpr) -> Result<(), String> {
        let label = self.eval_target_label(target)?;
        let target_region = *self
            .names
            .region_label_to_id
            .get(&label)
            .ok_or_else(|| format!("enter target must be region label, got :{label}"))?;
        self.current_region = target_region;
        self.current_block = 0;
        self.current_line = 0;
        self.reset_vregs();
        Ok(())
    }
}

impl Machine for FlowIrMachine {
    type Code = FlowIrCode;
    type AInput = StaticEnv;
    type FOutput = StaticEnv;
    type SnapShot = FlowIrMachine;
    type RInput = String;
    type ROutput = String;

    fn make(code: Self::Code, ainput: Self::AInput) -> Result<Self, String> {
        let names = compile_name_mapping(&code.0)?;
        let entry_region = names.entry_region;

        let mut static_mem = HashMap::new();
        for s in &code.0.statics {
            static_mem.insert(s.label.clone(), FlowValue::Num(s.value.clone()));
        }
        for (k, v) in ainput.entries {
            let slot = static_mem
                .get_mut(&k)
                .ok_or_else(|| format!("Unknown static input label: @{k}"))?;
            *slot = v;
        }

        Ok(Self {
            code,
            names,
            current_region: entry_region,
            current_block: 0,
            current_line: 0,
            vregs: BTreeMap::new(),
            static_mem,
            stack: Vec::new(),
            heap: HashMap::new(),
            next_handle: 1,
            halted: false,
        })
    }

    fn step(self, rinput: Self::RInput) -> Result<StepResult<Self>, String> {
        if self.halted {
            return Ok(self.halt_result());
        }

        let mut next = self;
        let block = next
            .code
            .0
            .regions
            .get(next.current_region)
            .and_then(|region| region.blocks.get(next.current_block))
            .cloned()
            .ok_or_else(|| {
                format!(
                    "Current block out of range: region={} block={}",
                    next.current_region, next.current_block
                )
            })?;

        let stmt_len = block.stmts.len();
        let if_len = block.cont.ifs().len();

        if next.current_line < stmt_len {
            let stmt = block.stmts[next.current_line].clone();
            next.current_line += 1;
            let mut output = String::new();
            match stmt {
                Stmt::Nop => {}
                Stmt::Load { dst, place } => {
                    let key = next.eval_place_key(&place)?;
                    let value = next.read_place(&key)?;
                    next.set_vreg(dst, value);
                }
                Stmt::Assign { dst, src } => {
                    let value = next.eval_value(&src)?;
                    next.set_vreg(dst, value);
                }
                Stmt::BinOp { dst, lhs, op, rhs } => {
                    let l = next.eval_value(&lhs)?;
                    let r = next.eval_value(&rhs)?;
                    let (l, r) = (l.as_number()?, r.as_number()?);
                    let value = match op {
                        BinOp::Add => l + r,
                        BinOp::Sub => l - r,
                    };
                    next.set_vreg(dst, FlowValue::Num(value));
                }
                Stmt::Store { place, src } => {
                    let key = next.eval_place_key(&place)?;
                    let value = next.eval_value(&src)?;
                    next.write_place(&key, value)?;
                }
                Stmt::Pop { dst } => {
                    let value = next
                        .stack
                        .pop()
                        .ok_or_else(|| "pop on empty stack".to_string())?;
                    next.set_vreg(dst, value);
                }
                Stmt::Push { src } => {
                    let value = next.eval_value(&src)?;
                    next.stack.push(value);
                }
                Stmt::LGet { dst } => {
                    next.set_vreg(dst, FlowValue::Num(Number::from(next.stack.len())));
                }
                Stmt::HAlloc { size, dst } => {
                    let size = next.eval_value(&size)?.as_usize()?;
                    let handle = next.next_handle;
                    next.next_handle += 1;
                    next.heap.insert(handle, vec![FlowValue::default(); size]);
                    next.set_vreg(dst, FlowValue::Num(Number::from(handle)));
                }
                Stmt::HFree { handle } => {
                    let handle = next.eval_value(&handle)?.as_usize()?;
                    let removed = next.heap.remove(&handle);
                    if removed.is_none() {
                        return Err(format!("hfree unknown handle: {handle}"));
                    }
                }
                Stmt::Print { src } => {
                    let num = next.get_vreg(src).as_number()?.trimmed_bytes();
                    output = std::str::from_utf8(&num)
                        .map_err(|_| format!("print byte is not valid single-byte UTF-8: {num:?}"))?
                        .to_string();
                }
                Stmt::Input { place } => {
                    let byte: Vec<u8> = rinput.as_bytes().to_vec();
                    let key = next.eval_place_key(&place)?;
                    next.write_place(&key, FlowValue::Num(Number::from(byte)))?;
                }
            }
            return Ok(next.continue_result(output));
        }

        if next.current_line < stmt_len + if_len {
            let if_idx = next.current_line - stmt_len;
            let jump_if = block.cont.ifs()[if_idx].clone();
            next.current_line += 1;
            if next.eval_cond(&jump_if.cond)? {
                next.jump_goto(&jump_if.target)?;
            }
            return Ok(next.continue_result(String::new()));
        }

        if next.current_line == stmt_len + if_len {
            next.current_line += 1;
            match block.cont {
                Cont::Go { target, .. } => next.jump_goto(&target)?,
                Cont::Enter { target, .. } => next.jump_enter(&target)?,
                Cont::Halt { .. } => {
                    next.halted = true;
                    return Ok(next.halt_result());
                }
            }
            return Ok(next.continue_result(String::new()));
        }

        Err(format!(
            "Current line out of range in region={} block={}: {}",
            next.current_region, next.current_block, next.current_line
        ))
    }

    fn snapshot(&self) -> Self::SnapShot {
        self.clone()
    }

    fn restore(snapshot: Self::SnapShot) -> Self {
        snapshot
    }

    fn render(snapshot: Self::SnapShot) -> utils::RenderState {
        let region = snapshot
            .code
            .0
            .regions
            .get(snapshot.current_region)
            .map(|r| r.label.clone())
            .unwrap_or_else(|| "<invalid>".to_string());
        let block = snapshot
            .code
            .0
            .regions
            .get(snapshot.current_region)
            .and_then(|rr| rr.blocks.get(snapshot.current_block))
            .map(|b| b.label.clone())
            .unwrap_or_else(|| "<invalid>".to_string());

        let vreg_rows = snapshot
            .vregs
            .iter()
            .map(|(name, v)| {
                utils::render_row!([
                    utils::render_text!(format!("%{}", name)),
                    utils::render_text!(v.print())
                ])
            })
            .collect::<Vec<_>>();

        let static_rows = snapshot
            .code
            .0
            .statics
            .iter()
            .map(|s| {
                let value = snapshot
                    .static_mem
                    .get(&s.label)
                    .cloned()
                    .unwrap_or_default()
                    .print();
                utils::render_row!([
                    utils::render_text!(format!("@{}", s.label)),
                    utils::render_text!(value)
                ])
            })
            .collect::<Vec<_>>();

        let stack_rows = snapshot
            .stack
            .iter()
            .enumerate()
            .map(|(i, v)| {
                utils::render_row!([
                    utils::render_text!(i.to_string()),
                    utils::render_text!(v.print())
                ])
            })
            .collect::<Vec<_>>();

        let heap_rows = snapshot
            .heap
            .iter()
            .flat_map(|(h, area)| {
                area.iter().enumerate().map(move |(i, v)| {
                    utils::render_row!([
                        utils::render_text!(h.to_string()),
                        utils::render_text!(i.to_string()),
                        utils::render_text!(v.print())
                    ])
                })
            })
            .collect::<Vec<_>>();

        utils::render_state![
            utils::render_text!(format!(":{}", region), title: "current_region"),
            utils::render_text!(format!(":{}", block), title: "current_block"),
            utils::render_text!(snapshot.current_line.to_string(), title: "current_line"),
            utils::render_text!(snapshot.halted.to_string(), title: "halted"),
            utils::render_table!(
                columns: vec![utils::render_text!("vreg"), utils::render_text!("value")],
                rows: vreg_rows,
                title: "vregs"
            ),
            utils::render_table!(
                columns: vec![utils::render_text!("label"), utils::render_text!("value")],
                rows: static_rows,
                title: "static"
            ),
            utils::render_table!(
                columns: vec![utils::render_text!("index"), utils::render_text!("value")],
                rows: stack_rows,
                title: "stack"
            ),
            utils::render_table!(
                columns: vec![
                    utils::render_text!("handle"),
                    utils::render_text!("index"),
                    utils::render_text!("value")
                ],
                rows: heap_rows,
                title: "heap"
            )
        ]
    }
}

fn compile_name_mapping(program: &Program) -> Result<NameMapping, String> {
    let mut static_labels = HashSet::new();
    for s in &program.statics {
        if !static_labels.insert(s.label.clone()) {
            return Err(format!("Duplicate static label: @{}", s.label));
        }
    }

    let mut region_label_to_id = HashMap::new();
    for (region_id, region) in program.regions.iter().enumerate() {
        if region.blocks.is_empty() {
            return Err(format!("Region :{} has no blocks", region.label));
        }
        if region_label_to_id
            .insert(region.label.clone(), region_id)
            .is_some()
        {
            return Err(format!("Duplicate region label: :{}", region.label));
        }
    }

    let mut block_labels_in_region = Vec::new();
    for region in &program.regions {
        let mut local = HashMap::new();
        for (block_idx, block) in region.blocks.iter().enumerate() {
            if local.insert(block.label.clone(), block_idx).is_some() {
                return Err(format!(
                    "Duplicate block label in region :{}: :{}",
                    region.label, block.label
                ));
            }
        }
        block_labels_in_region.push(local);
    }

    let entry_region = *region_label_to_id
        .get("main")
        .ok_or_else(|| "Missing entry region: :main".to_string())?;

    Ok(NameMapping {
        static_labels,
        region_label_to_id,
        block_labels_in_region,
        entry_region,
    })
}
