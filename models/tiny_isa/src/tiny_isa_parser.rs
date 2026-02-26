use serde_json::json;
use utils::{TextCodec, json_text};

use crate::tiny_isa::{DecodedInst, MachineCode, RegisterMemoryImage, TinyIsaMachine};
use utils::number::Number;

impl TextCodec for MachineCode {
    fn parse(text: &str) -> Result<Self, String> {
        let mut code = Vec::new();
        for line in text.lines() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }
            code.push(Number::parse(line)?);
        }
        Ok(Self(code))
    }

    fn write_fmt(&self, f: &mut impl std::fmt::Write) -> std::fmt::Result {
        for (i, n) in self.0.iter().enumerate() {
            if i > 0 {
                writeln!(f)?;
            }
            n.write_fmt(f)?;
        }
        Ok(())
    }
}

impl TextCodec for RegisterMemoryImage {
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

        let register: [Number; 8] = std::array::from_fn(|i| {
            Number::parse(reg_parts[i]).expect("validated in parse; unreachable")
        });

        let mut memory = Vec::new();
        for line in lines {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }
            memory.push(Number::parse(line)?);
        }

        Ok(Self { register, memory })
    }

    fn write_fmt(&self, f: &mut impl std::fmt::Write) -> std::fmt::Result {
        for (i, reg) in self.register.iter().enumerate() {
            if i > 0 {
                write!(f, ", ")?;
            }
            reg.write_fmt(f)?;
        }
        for mem in &self.memory {
            writeln!(f)?;
            mem.write_fmt(f)?;
        }
        Ok(())
    }
}

impl From<TinyIsaMachine> for serde_json::Value {
    fn from(machine: TinyIsaMachine) -> Self {
        let mut blocks = Vec::new();
        let pc = machine.regs[0].as_usize().ok();
        blocks.push(json_text!(
            pc.map(|v| v.to_string()).unwrap_or_else(|| "<overflow>".to_string()),
            title: "pc"
        ));
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
            .memory
            .iter()
            .take(machine.code_len)
            .enumerate()
            .map(|(addr, word)| {
                let pc_mark = if pc == Some(addr) { "*" } else { "" };
                let decoded = TinyIsaMachine::decode(word)
                    .map(|inst| decoded_inst_to_text(&inst))
                    .unwrap_or_else(|e| format!("ERR: {e}"));
                json!({"cells":[
                    json_text!(addr.to_string()),
                    json_text!(pc_mark),
                    json_text!(word.to_decimal_string()),
                    json_text!(decoded)
                ]})
            })
            .collect();
        blocks.push(json!({
            "kind": "table",
            "title": "code",
            "columns": [json_text!("addr"), json_text!("pc"), json_text!("word"), json_text!("decode")],
            "rows": code_rows
        }));

        let data_rows: Vec<serde_json::Value> = machine
            .memory
            .iter()
            .skip(machine.code_len)
            .enumerate()
            .map(|(offset, value)| {
                let addr = machine.code_len + offset;
                let pc_mark = if pc == Some(addr) { "*" } else { "" };
                json!({"cells":[
                    json_text!(offset.to_string()),
                    json_text!(addr.to_string()),
                    json_text!(pc_mark),
                    json_text!(value.to_decimal_string())
                ]})
            })
            .collect();
        blocks.push(json!({
            "kind": "table",
            "title": "data",
            "columns": [json_text!("offset"), json_text!("addr"), json_text!("pc"), json_text!("value")],
            "rows": data_rows
        }));

        serde_json::Value::Array(blocks)
    }
}

fn decoded_inst_to_text(inst: &DecodedInst) -> String {
    match inst {
        DecodedInst::Nop => "nop".to_string(),
        DecodedInst::Halt => "halt".to_string(),
        DecodedInst::Rst => "rst".to_string(),
        DecodedInst::Ld { rd, rb } => format!("ld r{rd} r{rb}"),
        DecodedInst::St { rs, rb } => format!("st r{rs} r{rb}"),
        DecodedInst::Mov { cond_flag, rd, rs } => if *cond_flag {
            format!("mov.f r{rd} r{rs}")
        } else {
            format!("mov r{rd} r{rs}")
        },
        DecodedInst::Ldi { cond_flag, rd, imm } => if *cond_flag {
            format!("ldi.f r{rd} #{}", imm.to_decimal_string())
        } else {
            format!("ldi r{rd} #{}", imm.to_decimal_string())
        },
        DecodedInst::Add { cond_flag, rd, rs } => if *cond_flag {
            format!("add.f r{rd} r{rs}")
        } else {
            format!("add r{rd} r{rs}")
        },
        DecodedInst::Sub { cond_flag, rd, rs } => if *cond_flag {
            format!("sub.f r{rd} r{rs}")
        } else {
            format!("sub r{rd} r{rs}")
        },
        DecodedInst::Addi { cond_flag, rd, imm } => if *cond_flag {
            format!("addi.f r{rd} #{}", imm.to_decimal_string())
        } else {
            format!("addi r{rd} #{}", imm.to_decimal_string())
        },
        DecodedInst::Subi { cond_flag, rd, imm } => if *cond_flag {
            format!("subi.f r{rd} #{}", imm.to_decimal_string())
        } else {
            format!("subi r{rd} #{}", imm.to_decimal_string())
        },
        DecodedInst::Eq { rd, rs } => format!("eq r{rd} r{rs}"),
        DecodedInst::Lt { rd, rs } => format!("lt r{rd} r{rs}"),
        DecodedInst::Gt { rd, rs } => format!("gt r{rd} r{rs}"),
    }
}
