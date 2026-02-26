use std::collections::HashMap;

use utils::number::Number;
use utils::{Machine, StepResult, TextCodec};

#[derive(Debug, Clone, Default)]
pub struct AsmCode(pub RawProgramAst);

#[derive(Debug, Clone, Default)]
pub struct RawProgramAst {
    pub items: Vec<RawItem>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RawItem {
    Directive(RawDirective),
    Label(String),
    Inst(RawInst),
    DataValue(RawValue),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RawDirective {
    Text,
    Data,
    Equ { name: String, value: Number },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RawInst {
    pub mnemonic: String,
    pub cond_flag: bool,
    pub operands: Vec<RawOperand>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RawOperand {
    Reg(RawReg),
    Value(RawValue),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RawReg(pub usize);

impl RawReg {
    fn parse(token: &str) -> Result<Self, String> {
        let t = token.trim();
        let t = t.strip_prefix('%').unwrap_or(t);
        let t = t.strip_prefix('r').unwrap_or(t);
        let reg = t
            .parse::<usize>()
            .map_err(|_| format!("Invalid register token: {token}"))?;
        if reg >= 8 {
            return Err(format!("Register out of range (expected 0..=7): {reg}"));
        }
        Ok(Self(reg))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RawValue {
    Imm(Number),
    Label(String),
    Const(String),
}

impl RawProgramAst {
    pub(crate) fn parse_source(text: &str) -> Result<Self, String> {
        let mut items = Vec::new();
        let mut section = AsmSection::Text;
        for raw_line in text.lines() {
            let line = raw_line.split("//").next().unwrap_or("").trim();
            if line.is_empty() {
                continue;
            }
            let item = if let Some(label) = RawItem::parse_label_line(line)? {
                RawItem::Label(label)
            } else if line.starts_with('.') {
                let directive = RawDirective::parse_line(line)?;
                match directive {
                    RawDirective::Text => section = AsmSection::Text,
                    RawDirective::Data => section = AsmSection::Data,
                    RawDirective::Equ { .. } => {}
                }
                RawItem::Directive(directive)
            } else {
                match section {
                    AsmSection::Text => RawItem::Inst(RawInst::parse_line(line)?),
                    AsmSection::Data => RawItem::DataValue(RawValue::parse_line(line)?),
                }
            };
            items.push(item);
        }
        Ok(Self { items })
    }

    pub(crate) fn write_fmt(&self, f: &mut impl std::fmt::Write) -> std::fmt::Result {
        for (i, item) in self.items.iter().enumerate() {
            if i > 0 {
                writeln!(f)?;
            }
            item.write_fmt(f)?;
        }
        Ok(())
    }
}

impl RawItem {
    fn parse_label_line(line: &str) -> Result<Option<String>, String> {
        let line = line.trim();
        if let Some(rest) = line.strip_prefix('@') {
            let Some(rest) = rest.strip_suffix(':') else {
                return Ok(None);
            };
            let label = rest.trim();
            if label.is_empty() {
                return Err("Empty label".to_string());
            }
            return Ok(Some(label.to_string()));
        }
        Ok(None)
    }

    fn write_fmt(&self, f: &mut impl std::fmt::Write) -> std::fmt::Result {
        match self {
            Self::Directive(d) => d.write_fmt(f),
            Self::Label(name) => write!(f, "@{}:", name),
            Self::Inst(inst) => inst.write_fmt(f),
            Self::DataValue(value) => value.write_fmt(f),
        }
    }
}

impl RawDirective {
    fn parse_line(line: &str) -> Result<Self, String> {
        let parts: Vec<_> = line.split_whitespace().collect();
        match parts.as_slice() {
            [".text"] => Ok(Self::Text),
            [".data"] => Ok(Self::Data),
            [".equ", name, imm] => Ok(Self::Equ {
                name: (*name).to_string(),
                value: {
                    let imm = imm.strip_prefix('#').unwrap_or(imm);
                    Number::parse(imm)?
                },
            }),
            _ => Err(format!("Unsupported directive syntax: {line}")),
        }
    }

    fn write_fmt(&self, f: &mut impl std::fmt::Write) -> std::fmt::Result {
        match self {
            Self::Text => write!(f, ".text"),
            Self::Data => write!(f, ".data"),
            Self::Equ { name, value } => write!(f, ".equ {} #{}", name, value.to_decimal_string()),
        }
    }
}

impl RawInst {
    fn parse_operand(token: &str) -> Result<RawOperand, String> {
        let t = token.trim();
        if t.is_empty() {
            return Err("Empty operand".to_string());
        }
        if t.starts_with('%') {
            return Ok(RawOperand::Reg(RawReg::parse(t)?));
        }
        Ok(RawOperand::Value(RawValue::parse_token(t)?))
    }

    fn parse_line(line: &str) -> Result<Self, String> {
        let line = line.trim();
        let line = line.strip_suffix(';').unwrap_or(line).trim();
        if line.is_empty() {
            return Err("Empty instruction line".to_string());
        }
        let parts: Vec<_> = line.split_whitespace().collect();
        let (mnemonic, cond_flag) = if let Some(base) = parts[0].strip_suffix(".f") {
            (base.to_string(), true)
        } else {
            (parts[0].to_string(), false)
        };
        let operands = parts[1..]
            .iter()
            .map(|p| Self::parse_operand(p))
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Self {
            mnemonic,
            cond_flag,
            operands,
        })
    }

    fn write_fmt(&self, f: &mut impl std::fmt::Write) -> std::fmt::Result {
        write!(f, "{}", self.mnemonic)?;
        if self.cond_flag {
            write!(f, ".f")?;
        }
        for op in &self.operands {
            write!(f, " ")?;
            match op {
                RawOperand::Reg(RawReg(r)) => write!(f, "%r{}", r)?,
                RawOperand::Value(value) => value.write_fmt(f)?,
            }
        }
        write!(f, ";")
    }
}

impl RawValue {
    fn parse_token(token: &str) -> Result<Self, String> {
        let token = token.trim();
        if token.is_empty() {
            return Err("Empty value".to_string());
        }
        if let Some(t) = token.strip_prefix('#') {
            return Ok(Self::Imm(Number::parse(t)?));
        }
        if let Some(label) = token.strip_prefix('@') {
            if label.is_empty() {
                return Err("Empty label value".to_string());
            }
            return Ok(Self::Label(label.to_string()));
        }
        if token.chars().all(|c| c.is_ascii_digit()) {
            return Ok(Self::Imm(Number::parse(token)?));
        }
        Ok(Self::Const(token.to_string()))
    }

    fn parse_line(line: &str) -> Result<Self, String> {
        let token = line.trim().strip_suffix(';').unwrap_or(line.trim()).trim();
        if token.is_empty() {
            return Err("Empty data value".to_string());
        }
        if token.contains(char::is_whitespace) {
            return Err("Data value line must contain a single value".to_string());
        }
        Self::parse_token(token)
    }

    fn write_fmt(&self, f: &mut impl std::fmt::Write) -> std::fmt::Result {
        match self {
            Self::Imm(n) => write!(f, "{}", n.to_decimal_string()),
            Self::Label(name) => write!(f, "@{}", name),
            Self::Const(name) => write!(f, "{}", name),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Inst {
    Nop,
    Halt,
    Rst,
    Ld {
        rd: usize,
        rb: usize,
    },
    St {
        rs: usize,
        rb: usize,
    },
    Mov {
        cond_flag: bool,
        rd: usize,
        rs: usize,
    },
    Ldi {
        cond_flag: bool,
        rd: usize,
        imm: RawValue,
    },
    Add {
        cond_flag: bool,
        rd: usize,
        rs: usize,
    },
    Sub {
        cond_flag: bool,
        rd: usize,
        rs: usize,
    },
    Addi {
        cond_flag: bool,
        rd: usize,
        imm: RawValue,
    },
    Subi {
        cond_flag: bool,
        rd: usize,
        imm: RawValue,
    },
    Eq {
        rd: usize,
        rs: usize,
    },
    Lt {
        rd: usize,
        rs: usize,
    },
    Gt {
        rd: usize,
        rs: usize,
    },
}


#[derive(Debug, Clone, Default)]
pub struct AsmInput {
    pub regs: [Number; 8],
    // Runtime extension appended after the static `.data` part from `Code`.
    pub data_extension: Vec<Number>,
}


#[derive(Debug, Clone, Default)]
pub struct AsmOutput {
    pub regs: [Number; 8],
    // Full data memory seen by the machine: static `.data` + runtime extension.
    pub data_memory: Vec<Number>,
}


#[derive(Debug, Clone)]
pub struct SymbolicAsmMachine {
    pub code: AsmCode,
    pub insts: Vec<Inst>,
    pub symbols: SymbolTables,
    pub debug_info: AsmDebugInfo,
    pub regs: [Number; 8],
    pub flag: bool,
    pub pc: usize,
    pub static_data_len: usize,
    pub static_data: Vec<Number>,
    pub data_memory: Vec<Number>,
    pub halted: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AsmSection {
    Text,
    Data,
}

#[derive(Debug, Clone, Default)]
pub struct SymbolTables {
    pub consts: HashMap<String, Number>,
    pub text_labels: HashMap<String, usize>,
    pub data_labels: HashMap<String, usize>,
    pub code_len: usize,
    pub data_len: usize,
}

#[derive(Debug, Clone, Default)]
pub struct AsmDebugInfo {
    pub text_labels_by_addr: HashMap<usize, Vec<String>>,
    pub data_labels_by_addr: HashMap<usize, Vec<String>>,
}

#[derive(Debug, Clone, Default)]
struct PreprocessResult {
    insts: Vec<Inst>,
    static_data: Vec<Number>,
    symbols: SymbolTables,
    debug_info: AsmDebugInfo,
}

impl SymbolicAsmMachine {
    fn output(&self) -> AsmOutput {
        AsmOutput {
            regs: self.regs.clone(),
            data_memory: self.data_memory.clone(),
        }
    }

    fn sync_reg0_from_pc(&mut self) {
        self.regs[0] = Number::from(self.pc);
    }

    fn set_pc_from_number(&mut self, value: Number) -> Result<(), String> {
        self.regs[0] = value;
        self.pc = self.regs[0].as_usize()?;
        Ok(())
    }

    fn advance_pc(&mut self) {
        self.pc += 1;
        self.sync_reg0_from_pc();
    }

    fn read_data(&self, addr: usize) -> Number {
        self.data_memory.get(addr).cloned().unwrap_or_default()
    }

    fn write_data(&mut self, addr: usize, value: Number) {
        if addr >= self.data_memory.len() {
            self.data_memory.resize(addr + 1, Number::default());
        }
        self.data_memory[addr] = value;
    }

    fn resolve_inst_value(&self, value: &RawValue, pc: usize) -> Result<Number, String> {
        Self::resolve_raw_value(value, &self.symbols, AsmSection::Text, pc)
    }

    fn write_reg_with_pc_rule(&mut self, rd: usize, value: Number) -> Result<(), String> {
        if rd == 0 {
            self.set_pc_from_number(value)
        } else {
            self.regs[rd] = value;
            self.advance_pc();
            Ok(())
        }
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

    fn preprocess(code: &AsmCode) -> Result<PreprocessResult, String> {
        let tables = Self::pass1_collect_symbols(&code.0)?;
        let (insts, static_data) = Self::pass2_lower(&code.0, &tables)?;
        let debug_info = Self::build_debug_info(&tables);
        Ok(PreprocessResult {
            insts,
            static_data,
            symbols: tables,
            debug_info,
        })
    }

    fn pass1_collect_symbols(ast: &RawProgramAst) -> Result<SymbolTables, String> {
        let mut tables = SymbolTables::default();
        let mut section = AsmSection::Text;

        for item in &ast.items {
            match item {
                RawItem::Directive(RawDirective::Text) => section = AsmSection::Text,
                RawItem::Directive(RawDirective::Data) => section = AsmSection::Data,
                RawItem::Directive(RawDirective::Equ { name, value }) => {
                    Self::ensure_symbol_name_available(&tables, name)?;
                    tables.consts.insert(name.clone(), value.clone());
                }
                RawItem::Label(name) => {
                    Self::ensure_symbol_name_available(&tables, name)?;
                    match section {
                        AsmSection::Text => {
                            tables.text_labels.insert(name.clone(), tables.code_len);
                        }
                        AsmSection::Data => {
                            tables.data_labels.insert(name.clone(), tables.data_len);
                        }
                    }
                }
                RawItem::Inst(_) => {
                    if section != AsmSection::Text {
                        return Err("Instruction found outside .text section".to_string());
                    }
                    tables.code_len += 1;
                }
                RawItem::DataValue(_) => {
                    if section != AsmSection::Data {
                        return Err("Data value found outside .data section".to_string());
                    }
                    tables.data_len += 1;
                }
            }
        }

        Ok(tables)
    }

    fn pass2_lower(
        ast: &RawProgramAst,
        tables: &SymbolTables,
    ) -> Result<(Vec<Inst>, Vec<Number>), String> {
        let mut section = AsmSection::Text;
        let mut data_pos = 0usize;
        let mut insts = Vec::with_capacity(tables.code_len);
        let mut data = Vec::with_capacity(tables.data_len);

        for item in &ast.items {
            match item {
                RawItem::Directive(RawDirective::Text) => section = AsmSection::Text,
                RawItem::Directive(RawDirective::Data) => section = AsmSection::Data,
                RawItem::Directive(RawDirective::Equ { .. }) => {}
                RawItem::Label(_) => {}
                RawItem::Inst(raw) => {
                    if section != AsmSection::Text {
                        return Err("Instruction found outside .text section".to_string());
                    }
                    let inst = Self::lower_inst(raw)?;
                    insts.push(inst);
                }
                RawItem::DataValue(raw) => {
                    if section != AsmSection::Data {
                        return Err("Data value found outside .data section".to_string());
                    }
                    let value = Self::resolve_raw_value(raw, tables, AsmSection::Data, data_pos)?;
                    data.push(value);
                    data_pos += 1;
                }
            }
        }

        Ok((insts, data))
    }

    fn build_debug_info(tables: &SymbolTables) -> AsmDebugInfo {
        let mut debug_info = AsmDebugInfo::default();
        for (label, &addr) in &tables.text_labels {
            debug_info
                .text_labels_by_addr
                .entry(addr)
                .or_default()
                .push(label.clone());
        }
        for (label, &addr) in &tables.data_labels {
            debug_info
                .data_labels_by_addr
                .entry(addr)
                .or_default()
                .push(label.clone());
        }
        for labels in debug_info.text_labels_by_addr.values_mut() {
            labels.sort();
        }
        for labels in debug_info.data_labels_by_addr.values_mut() {
            labels.sort();
        }
        debug_info
    }

    fn ensure_symbol_name_available(tables: &SymbolTables, name: &str) -> Result<(), String> {
        if Self::is_reserved_symbol(name) {
            return Err(format!("Reserved symbol name cannot be redefined: @{name}"));
        }
        if tables.consts.contains_key(name)
            || tables.text_labels.contains_key(name)
            || tables.data_labels.contains_key(name)
        {
            return Err(format!("Duplicate symbol definition: {name}"));
        }
        Ok(())
    }

    fn is_reserved_symbol(name: &str) -> bool {
        matches!(name, "THIS" | "CODE_LEN" | "DATA_LEN")
    }

    fn resolve_label_symbol(
        name: &str,
        tables: &SymbolTables,
        section: AsmSection,
        this_pos: usize,
    ) -> Result<Number, String> {
        match name {
            "THIS" => return Ok(Number::from(this_pos)),
            "CODE_LEN" => return Ok(Number::from(tables.code_len)),
            "DATA_LEN" => return Ok(Number::from(tables.data_len)),
            _ => {}
        }

        if let Some(pos) = tables.text_labels.get(name) {
            return Ok(Number::from(*pos));
        }
        if let Some(pos) = tables.data_labels.get(name) {
            return Ok(Number::from(*pos));
        }

        let section_name = match section {
            AsmSection::Text => ".text",
            AsmSection::Data => ".data",
        };
        Err(format!(
            "Unknown label reference in {section_name}: @{name}"
        ))
    }

    fn resolve_const_symbol(name: &str, tables: &SymbolTables) -> Result<Number, String> {
        tables
            .consts
            .get(name)
            .cloned()
            .ok_or_else(|| format!("Unknown const reference: {name}"))
    }

    fn resolve_raw_value(
        value: &RawValue,
        tables: &SymbolTables,
        section: AsmSection,
        this_pos: usize,
    ) -> Result<Number, String> {
        match value {
            RawValue::Imm(n) => Ok(n.clone()),
            RawValue::Label(name) => Self::resolve_label_symbol(name, tables, section, this_pos),
            RawValue::Const(name) => Self::resolve_const_symbol(name, tables),
        }
    }

    fn expect_value_operand(op: &RawOperand, arg_name: &str) -> Result<RawValue, String> {
        match op {
            RawOperand::Value(v) => Ok(v.clone()),
            RawOperand::Reg(_) => Err(format!("Expected value operand for {arg_name}")),
        }
    }

    fn expect_reg_operand(op: &RawOperand, arg_name: &str) -> Result<usize, String> {
        match op {
            RawOperand::Reg(RawReg(r)) => Ok(*r),
            _ => Err(format!("Expected register operand for {arg_name}")),
        }
    }

    fn lower_inst(raw: &RawInst) -> Result<Inst, String> {
        let ops = &raw.operands;
        let m = raw.mnemonic.as_str();
        let cf = raw.cond_flag;

        match m {
            "nop" => {
                if cf {
                    return Err("nop.f is not supported".to_string());
                }
                if !ops.is_empty() {
                    return Err("nop takes no operands".to_string());
                }
                Ok(Inst::Nop)
            }
            "halt" => {
                if cf {
                    return Err("halt.f is not supported".to_string());
                }
                if !ops.is_empty() {
                    return Err("halt takes no operands".to_string());
                }
                Ok(Inst::Halt)
            }
            "rst" => {
                if cf {
                    return Err("rst.f is not supported".to_string());
                }
                if !ops.is_empty() {
                    return Err("rst takes no operands".to_string());
                }
                Ok(Inst::Rst)
            }
            "ld" => {
                if cf {
                    return Err("ld.f is not supported".to_string());
                }
                if ops.len() != 2 {
                    return Err("ld requires: ld <rd> <rb>".to_string());
                }
                Ok(Inst::Ld {
                    rd: Self::expect_reg_operand(&ops[0], "rd")?,
                    rb: Self::expect_reg_operand(&ops[1], "rb")?,
                })
            }
            "st" => {
                if cf {
                    return Err("st.f is not supported".to_string());
                }
                if ops.len() != 2 {
                    return Err("st requires: st <rs> <rb>".to_string());
                }
                Ok(Inst::St {
                    rs: Self::expect_reg_operand(&ops[0], "rs")?,
                    rb: Self::expect_reg_operand(&ops[1], "rb")?,
                })
            }
            "mov" => {
                if ops.len() != 2 {
                    return Err("mov requires: mov[.f] <rd> <rs>".to_string());
                }
                Ok(Inst::Mov {
                    cond_flag: cf,
                    rd: Self::expect_reg_operand(&ops[0], "rd")?,
                    rs: Self::expect_reg_operand(&ops[1], "rs")?,
                })
            }
            "jump" => {
                if ops.len() != 1 {
                    return Err("jump requires: jump[.f] <reg|value>".to_string());
                }
                match &ops[0] {
                    RawOperand::Reg(RawReg(rs)) => Ok(Inst::Mov {
                        cond_flag: cf,
                        rd: 0,
                        rs: *rs,
                    }),
                    _ => Ok(Inst::Ldi {
                        cond_flag: cf,
                        rd: 0,
                        imm: Self::expect_value_operand(&ops[0], "target")?,
                    }),
                }
            }
            "ldi" => {
                if ops.len() != 2 {
                    return Err("ldi requires: ldi[.f] <rd> <value>".to_string());
                }
                Ok(Inst::Ldi {
                    cond_flag: cf,
                    rd: Self::expect_reg_operand(&ops[0], "rd")?,
                    imm: Self::expect_value_operand(&ops[1], "imm")?,
                })
            }
            "add" => {
                if ops.len() != 2 {
                    return Err("add requires: add[.f] <rd> <reg|value>".to_string());
                }
                let rd = Self::expect_reg_operand(&ops[0], "rd")?;
                match &ops[1] {
                    RawOperand::Reg(RawReg(rs)) => Ok(Inst::Add {
                        cond_flag: cf,
                        rd,
                        rs: *rs,
                    }),
                    _ => Ok(Inst::Addi {
                        cond_flag: cf,
                        rd,
                        imm: Self::expect_value_operand(&ops[1], "rhs")?,
                    }),
                }
            }
            "sub" => {
                if ops.len() != 2 {
                    return Err("sub requires: sub[.f] <rd> <reg|value>".to_string());
                }
                let rd = Self::expect_reg_operand(&ops[0], "rd")?;
                match &ops[1] {
                    RawOperand::Reg(RawReg(rs)) => Ok(Inst::Sub {
                        cond_flag: cf,
                        rd,
                        rs: *rs,
                    }),
                    _ => Ok(Inst::Subi {
                        cond_flag: cf,
                        rd,
                        imm: Self::expect_value_operand(&ops[1], "rhs")?,
                    }),
                }
            }
            "addi" => {
                if ops.len() != 2 {
                    return Err("addi requires: addi[.f] <rd> <value>".to_string());
                }
                Ok(Inst::Addi {
                    cond_flag: cf,
                    rd: Self::expect_reg_operand(&ops[0], "rd")?,
                    imm: Self::expect_value_operand(&ops[1], "imm")?,
                })
            }
            "subi" => {
                if ops.len() != 2 {
                    return Err("subi requires: subi[.f] <rd> <value>".to_string());
                }
                Ok(Inst::Subi {
                    cond_flag: cf,
                    rd: Self::expect_reg_operand(&ops[0], "rd")?,
                    imm: Self::expect_value_operand(&ops[1], "imm")?,
                })
            }
            "eq" => {
                if cf {
                    return Err("eq.f is not supported".to_string());
                }
                if ops.len() != 2 {
                    return Err("eq requires: eq <rd> <rs>".to_string());
                }
                Ok(Inst::Eq {
                    rd: Self::expect_reg_operand(&ops[0], "rd")?,
                    rs: Self::expect_reg_operand(&ops[1], "rs")?,
                })
            }
            "lt" => {
                if cf {
                    return Err("lt.f is not supported".to_string());
                }
                if ops.len() != 2 {
                    return Err("lt requires: lt <rd> <rs>".to_string());
                }
                Ok(Inst::Lt {
                    rd: Self::expect_reg_operand(&ops[0], "rd")?,
                    rs: Self::expect_reg_operand(&ops[1], "rs")?,
                })
            }
            "gt" => {
                if cf {
                    return Err("gt.f is not supported".to_string());
                }
                if ops.len() != 2 {
                    return Err("gt requires: gt <rd> <rs>".to_string());
                }
                Ok(Inst::Gt {
                    rd: Self::expect_reg_operand(&ops[0], "rd")?,
                    rs: Self::expect_reg_operand(&ops[1], "rs")?,
                })
            }
            _ => Err(format!(
                "Unsupported symbolic instruction in preprocess: {}",
                raw.mnemonic
            )),
        }
    }
}

impl Machine for SymbolicAsmMachine {
    type Code = AsmCode;
    type AInput = AsmInput;
    type FOutput = AsmOutput;
    type SnapShot = SymbolicAsmMachine;
    type RInput = ();
    type ROutput = ();

    fn make(code: Self::Code, ainput: Self::AInput) -> Result<Self, String> {
        let pre = Self::preprocess(&code)?;
        let static_data_len = pre.static_data.len();
        let mut data_memory = pre.static_data.clone();
        data_memory.extend(ainput.data_extension);
        let pc = ainput.regs[0].as_usize()?;
        Ok(Self {
            code,
            insts: pre.insts,
            symbols: pre.symbols,
            debug_info: pre.debug_info,
            regs: ainput.regs,
            flag: false,
            pc,
            static_data_len,
            static_data: pre.static_data,
            data_memory,
            halted: false,
        })
    }

    fn step(self, _rinput: Self::RInput) -> Result<StepResult<Self>, String> {
        if self.halted {
            return Ok(self.halt_result());
        }

        let mut next = self;

        if next.pc >= next.insts.len() {
            next.halted = true;
            next.sync_reg0_from_pc();
            return Ok(next.halt_result());
        }

        let pc = next.pc;
        let inst = next.insts[pc].clone();

        match inst {
            Inst::Nop => {
                next.advance_pc();
                Ok(next.continue_result())
            }
            Inst::Halt => {
                next.halted = true;
                next.sync_reg0_from_pc();
                Ok(next.halt_result())
            }
            Inst::Rst => {
                next.flag = false;
                next.advance_pc();
                Ok(next.continue_result())
            }
            Inst::Ld { rd, rb } => {
                let addr = next.regs[rb].as_usize()?;
                let value = next.read_data(addr);
                next.write_reg_with_pc_rule(rd, value)?;
                Ok(next.continue_result())
            }
            Inst::St { rs, rb } => {
                let addr = next.regs[rb].as_usize()?;
                let value = next.regs[rs].clone();
                next.write_data(addr, value);
                next.advance_pc();
                Ok(next.continue_result())
            }
            Inst::Mov { cond_flag, rd, rs } => {
                if cond_flag && !next.flag {
                    next.advance_pc();
                } else {
                    let value = next.regs[rs].clone();
                    next.write_reg_with_pc_rule(rd, value)?;
                }
                Ok(next.continue_result())
            }
            Inst::Ldi { cond_flag, rd, imm } => {
                if cond_flag && !next.flag {
                    next.advance_pc();
                } else {
                    let value = next.resolve_inst_value(&imm, pc)?;
                    next.write_reg_with_pc_rule(rd, value)?;
                }
                Ok(next.continue_result())
            }
            Inst::Add { cond_flag, rd, rs } => {
                if cond_flag && !next.flag {
                    next.advance_pc();
                } else {
                    let value = next.regs[rd].clone() + next.regs[rs].clone();
                    next.write_reg_with_pc_rule(rd, value)?;
                }
                Ok(next.continue_result())
            }
            Inst::Sub { cond_flag, rd, rs } => {
                if cond_flag && !next.flag {
                    next.advance_pc();
                } else {
                    let value = next.regs[rd].clone() - next.regs[rs].clone();
                    next.write_reg_with_pc_rule(rd, value)?;
                }
                Ok(next.continue_result())
            }
            Inst::Addi { cond_flag, rd, imm } => {
                if cond_flag && !next.flag {
                    next.advance_pc();
                } else {
                    let rhs = next.resolve_inst_value(&imm, pc)?;
                    let value = next.regs[rd].clone() + rhs;
                    next.write_reg_with_pc_rule(rd, value)?;
                }
                Ok(next.continue_result())
            }
            Inst::Subi { cond_flag, rd, imm } => {
                if cond_flag && !next.flag {
                    next.advance_pc();
                } else {
                    let rhs = next.resolve_inst_value(&imm, pc)?;
                    let value = next.regs[rd].clone() - rhs;
                    next.write_reg_with_pc_rule(rd, value)?;
                }
                Ok(next.continue_result())
            }
            Inst::Eq { rd, rs } => {
                next.flag = next.regs[rd] == next.regs[rs];
                next.advance_pc();
                Ok(next.continue_result())
            }
            Inst::Lt { rd, rs } => {
                next.flag = next.regs[rd] < next.regs[rs];
                next.advance_pc();
                Ok(next.continue_result())
            }
            Inst::Gt { rd, rs } => {
                next.flag = next.regs[rd] > next.regs[rs];
                next.advance_pc();
                Ok(next.continue_result())
            }
        }
    }

    fn current(&self) -> Self::SnapShot {
        self.clone()
    }
}
