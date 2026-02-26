use serde_json::json;
use utils::number::Number;
use utils::{TextCodec, json_text};

use crate::symbolic_asm::{
    AsmCode, AsmInput, AsmOutput, Inst, RawProgramAst, RawValue, SymbolicAsmMachine,
};

impl TextCodec for AsmCode {
    fn parse(text: &str) -> Result<Self, String> {
        Ok(Self(RawProgramAst::parse_source(text)?))
    }

    fn write_fmt(&self, f: &mut impl std::fmt::Write) -> std::fmt::Result {
        self.0.write_fmt(f)
    }
}

impl TextCodec for AsmInput {
    fn parse(text: &str) -> Result<Self, String> {
        let mut lines = text.lines();
        let reg_line = lines
            .find(|line| !line.trim().is_empty())
            .ok_or_else(|| "Expected first line with 8 registers".to_string())?;

        let reg_parts: Vec<_> = reg_line
            .split(',')
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .collect();
        if reg_parts.len() != 8 {
            return Err("Expected 8 registers on the first line (comma separated)".to_string());
        }

        let regs: [Number; 8] = std::array::from_fn(|i| {
            Number::parse(reg_parts[i]).expect("validated in parse; unreachable")
        });

        let mut data_extension = Vec::new();
        for line in lines {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }
            data_extension.push(Number::parse(line)?);
        }

        Ok(Self {
            regs,
            data_extension,
        })
    }

    fn write_fmt(&self, f: &mut impl std::fmt::Write) -> std::fmt::Result {
        for (i, reg) in self.regs.iter().enumerate() {
            if i > 0 {
                write!(f, ", ")?;
            }
            reg.write_fmt(f)?;
        }
        for value in &self.data_extension {
            writeln!(f)?;
            value.write_fmt(f)?;
        }
        Ok(())
    }
}

impl TextCodec for AsmOutput {
    fn parse(text: &str) -> Result<Self, String> {
        let mut lines = text.lines();
        let reg_line = lines
            .find(|line| !line.trim().is_empty())
            .ok_or_else(|| "Expected first line with 8 registers".to_string())?;

        let reg_parts: Vec<_> = reg_line
            .split(',')
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .collect();
        if reg_parts.len() != 8 {
            return Err("Expected 8 registers on the first line (comma separated)".to_string());
        }

        let regs: [Number; 8] = std::array::from_fn(|i| {
            Number::parse(reg_parts[i]).expect("validated in parse; unreachable")
        });

        let mut data_memory = Vec::new();
        for line in lines {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }
            data_memory.push(Number::parse(line)?);
        }

        Ok(Self { regs, data_memory })
    }

    fn write_fmt(&self, f: &mut impl std::fmt::Write) -> std::fmt::Result {
        for (i, reg) in self.regs.iter().enumerate() {
            if i > 0 {
                write!(f, ", ")?;
            }
            reg.write_fmt(f)?;
        }
        for value in &self.data_memory {
            writeln!(f)?;
            value.write_fmt(f)?;
        }
        Ok(())
    }
}

impl From<SymbolicAsmMachine> for serde_json::Value {
    fn from(machine: SymbolicAsmMachine) -> Self {
        let mut blocks = Vec::new();
        let pc = Some(machine.pc);
        blocks.push(json_text!(machine.pc.to_string(), title: "pc"));
        blocks.push(json_text!(machine.flag.to_string(), title: "flag"));
        blocks.push(json_text!(machine.halted.to_string(), title: "halted"));

        let reg_rows: Vec<serde_json::Value> = machine
            .regs
            .iter()
            .enumerate()
            .map(|(i, value)| {
                json!({"cells":[json_text!(format!("r{i}")), json_text!(value.to_decimal_string())]})
            })
            .collect();
        blocks.push(json!({
            "kind": "table",
            "title": "registers",
            "columns": [json_text!("reg"), json_text!("value")],
            "rows": reg_rows
        }));

        let code_rows: Vec<serde_json::Value> = machine
            .insts
            .iter()
            .enumerate()
            .map(|(addr, inst)| {
                let labels = machine
                    .debug_info
                    .text_labels_by_addr
                    .get(&addr)
                    .cloned()
                    .unwrap_or_default()
                    .join(", ");
                let pc_mark = if pc == Some(addr) { "*" } else { "" };
                json!({"cells":[
                    json_text!(labels),
                    json_text!(addr.to_string()),
                    json_text!(pc_mark),
                    json_text!(inst_to_text(inst))
                ]})
            })
            .collect();
        blocks.push(json!({
            "kind": "table",
            "title": "code",
            "columns": [json_text!("label"), json_text!("addr"), json_text!("pc"), json_text!("inst")],
            "rows": code_rows
        }));

        let data_rows: Vec<serde_json::Value> = machine
            .data_memory
            .iter()
            .enumerate()
            .map(|(addr, value)| {
                let labels = machine
                    .debug_info
                    .data_labels_by_addr
                    .get(&addr)
                    .cloned()
                    .unwrap_or_default()
                    .join(", ");
                let region = if addr < machine.static_data_len {
                    "static"
                } else {
                    "input/ext"
                };
                json!({"cells":[
                    json_text!(labels),
                    json_text!(addr.to_string()),
                    json_text!(region),
                    json_text!(value.to_decimal_string())
                ]})
            })
            .collect();
        blocks.push(json!({
            "kind": "table",
            "title": "data",
            "columns": [json_text!("label"), json_text!("addr"), json_text!("region"), json_text!("value")],
            "rows": data_rows
        }));

        serde_json::Value::Array(blocks)
    }
}

fn raw_value_to_text(value: &RawValue) -> String {
    match value {
        RawValue::Imm(n) => format!("#{}", n.to_decimal_string()),
        RawValue::Label(name) => format!("@{name}"),
        RawValue::Const(name) => name.clone(),
    }
}

fn inst_to_text(inst: &Inst) -> String {
    match inst {
        Inst::Nop => "nop".to_string(),
        Inst::Halt => "halt".to_string(),
        Inst::Rst => "rst".to_string(),
        Inst::Ld { rd, rb } => format!("ld %r{rd} %r{rb}"),
        Inst::St { rs, rb } => format!("st %r{rs} %r{rb}"),
        Inst::Mov { cond_flag, rd, rs } => {
            let suffix = if *cond_flag { ".f" } else { "" };
            format!("mov{suffix} %r{rd} %r{rs}")
        }
        Inst::Ldi { cond_flag, rd, imm } => {
            let suffix = if *cond_flag { ".f" } else { "" };
            format!("ldi{suffix} %r{rd} {}", raw_value_to_text(imm))
        }
        Inst::Add { cond_flag, rd, rs } => {
            let suffix = if *cond_flag { ".f" } else { "" };
            format!("add{suffix} %r{rd} %r{rs}")
        }
        Inst::Sub { cond_flag, rd, rs } => {
            let suffix = if *cond_flag { ".f" } else { "" };
            format!("sub{suffix} %r{rd} %r{rs}")
        }
        Inst::Addi { cond_flag, rd, imm } => {
            let suffix = if *cond_flag { ".f" } else { "" };
            format!("addi{suffix} %r{rd} {}", raw_value_to_text(imm))
        }
        Inst::Subi { cond_flag, rd, imm } => {
            let suffix = if *cond_flag { ".f" } else { "" };
            format!("subi{suffix} %r{rd} {}", raw_value_to_text(imm))
        }
        Inst::Eq { rd, rs } => format!("eq %r{rd} %r{rs}"),
        Inst::Lt { rd, rs } => format!("lt %r{rd} %r{rs}"),
        Inst::Gt { rd, rs } => format!("gt %r{rd} %r{rs}"),
    }
}
