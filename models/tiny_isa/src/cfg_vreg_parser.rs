use serde_json::json;
use utils::number::Number;
use utils::{TextCodec, json_text};

use crate::cfg_vreg::*;

impl TextCodec for CfgVRegCode {
    fn parse(text: &str) -> Result<Self, String> {
        Ok(Self(parse_program(text)?))
    }

    fn write_fmt(&self, f: &mut impl std::fmt::Write) -> std::fmt::Result {
        for s in &self.0.statics {
            writeln!(f, "@{} {}", s.label, s.value.to_decimal_string())?;
        }
        for block in &self.0.blocks {
            writeln!(f, "@{} {{", block.label)?;
            for stmt in &block.stmts {
                writeln!(f, "  {}", stmt_to_text(stmt))?;
            }
            for j in &block.cont.ifs {
                writeln!(
                    f,
                    "  if {} then {};",
                    cond_to_text(&j.cond),
                    addr_to_text(&j.target)
                )?;
            }
            writeln!(f, "  goto {};", addr_to_text(&block.cont.jump))?;
            writeln!(f, "}}")?;
        }
        Ok(())
    }
}

impl From<CfgVRegMachine> for serde_json::Value {
    fn from(machine: CfgVRegMachine) -> Self {
        let mut blocks = Vec::new();
        let block_label = machine
            .compiled
            .blocks
            .get(machine.current_block)
            .map(|b| b.label.clone())
            .unwrap_or_else(|| "<invalid>".to_string());
        blocks.push(json_text!(block_label, title: "current_block"));
        blocks.push(json_text!(machine.current_block.to_string(), title: "block_addr"));
        blocks.push(json_text!(machine.halted.to_string(), title: "halted"));

        let vreg_rows: Vec<serde_json::Value> = machine
            .vregs
            .iter()
            .enumerate()
            .map(|(i, v)| {
                json!({ "cells": [json_text!(format!("v{i}")), json_text!(v.to_decimal_string())] })
            })
            .collect();
        blocks.push(json!({
            "kind": "table",
            "title": "vregs",
            "columns": [json_text!("vreg"), json_text!("value")],
            "rows": vreg_rows
        }));

        let code_rows: Vec<serde_json::Value> = machine
            .compiled
            .blocks
            .iter()
            .enumerate()
            .map(|(i, b)| {
                let marker = if i == machine.current_block { "*" } else { "" };
                json!({
                    "cells": [
                        json_text!(b.label.clone()),
                        json_text!(i.to_string()),
                        json_text!(marker),
                        json_text!(block_summary_text(b))
                    ]
                })
            })
            .collect();
        blocks.push(json!({
            "kind": "table",
            "title": "blocks",
            "columns": [json_text!("label"), json_text!("addr"), json_text!("pc"), json_text!("summary")],
            "rows": code_rows
        }));

        let data_rows: Vec<serde_json::Value> = machine
            .memory
            .iter()
            .enumerate()
            .map(|(i, v)| {
                let label = machine
                    .compiled
                    .static_labels
                    .iter()
                    .filter_map(|(name, &addr)| (addr == i).then_some(name.clone()))
                    .collect::<Vec<_>>()
                    .join(", ");
                let region = if i < machine.code.0.statics.len() {
                    "static"
                } else {
                    "input/ext"
                };
                json!({
                    "cells": [
                        json_text!(label),
                        json_text!(i.to_string()),
                        json_text!(region),
                        json_text!(v.to_decimal_string())
                    ]
                })
            })
            .collect();
        blocks.push(json!({
            "kind": "table",
            "title": "memory",
            "columns": [json_text!("label"), json_text!("addr"), json_text!("region"), json_text!("value")],
            "rows": data_rows
        }));

        serde_json::Value::Array(blocks)
    }
}

fn parse_program(text: &str) -> Result<Program, String> {
    let mut lines = text
        .lines()
        .map(|l| l.split("//").next().unwrap_or("").trim().to_string())
        .collect::<Vec<_>>();
    lines.retain(|l| !l.is_empty());

    let mut i = 0usize;
    let mut statics = Vec::new();
    let mut blocks = Vec::new();

    while i < lines.len() {
        let line = &lines[i];
        if is_block_start(line) {
            let (block, next_i) = parse_block(&lines, i)?;
            blocks.push(block);
            i = next_i;
            continue;
        }
        if let Some((label, value)) = parse_static_line(line)? {
            statics.push(StaticDef { label, value });
            i += 1;
            continue;
        }
        return Err(format!("Unexpected top-level line: {}", line));
    }

    Ok(Program { statics, blocks })
}

fn is_block_start(line: &str) -> bool {
    line.starts_with('@') && line.ends_with('{')
}

fn parse_block(lines: &[String], start: usize) -> Result<(Block, usize), String> {
    let header = &lines[start];
    let label = header
        .strip_prefix('@')
        .and_then(|s| s.strip_suffix('{'))
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .ok_or_else(|| format!("Invalid block header: {}", header))?
        .to_string();

    let mut i = start + 1;
    let mut stmts = Vec::new();
    let mut ifs = Vec::new();
    let mut final_jump: Option<AddrExpr> = None;

    while i < lines.len() {
        let line = lines[i].trim();
        if line == "}" {
            break;
        }
        if let Some(j) = parse_jump_if(line)? {
            ifs.push(j);
            i += 1;
            continue;
        }
        if let Some(target) = parse_goto(line)? {
            if final_jump.is_some() {
                return Err(format!("Multiple final goto in block @{}", label));
            }
            final_jump = Some(target);
            i += 1;
            continue;
        }
        if final_jump.is_some() {
            return Err(format!("Statement after goto in block @{}", label));
        }
        if let Some(stmt) = parse_stmt(line)? {
            stmts.push(stmt);
            i += 1;
            continue;
        }
        return Err(format!("Invalid line in block @{}: {}", label, line));
    }

    if i >= lines.len() || lines[i].trim() != "}" {
        return Err(format!("Unclosed block: @{}", label));
    }
    let jump = final_jump.ok_or_else(|| format!("Block @{} missing final goto", label))?;
    Ok((
        Block {
            label,
            stmts,
            cont: Cont { ifs, jump },
        },
        i + 1,
    ))
}

fn parse_static_line(line: &str) -> Result<Option<(String, Number)>, String> {
    let parts: Vec<_> = line.split_whitespace().collect();
    if parts.len() != 2 || !parts[0].starts_with('@') {
        return Ok(None);
    }
    let label = parts[0]
        .strip_prefix('@')
        .ok_or_else(|| "invalid static label".to_string())?;
    if label.ends_with(':') {
        return Ok(None);
    }
    Ok(Some((label.to_string(), parse_number(parts[1])?)))
}

fn parse_stmt(line: &str) -> Result<Option<Stmt>, String> {
    let line = line.trim();
    if !line.ends_with(';') {
        return Ok(None);
    }
    let body = line[..line.len() - 1].trim();
    let Some((lhs, rhs)) = body.split_once(":=") else {
        return Ok(None);
    };
    let lhs = lhs.trim();
    let rhs = rhs.trim();

    if lhs.starts_with("%v") {
        let dst = parse_vreg(lhs)?;
        let tokens: Vec<_> = rhs.split_whitespace().collect();
        if tokens.len() == 1 {
            return Ok(Some(Stmt::Assign {
                dst,
                src: parse_value_expr(tokens[0])?,
            }));
        }
        if tokens.len() == 3 {
            let lhs = parse_value_expr(tokens[0])?;
            let op = match tokens[1] {
                "+" => BinOp::Add,
                "-" => BinOp::Sub,
                _ => return Err(format!("Unsupported op: {}", tokens[1])),
            };
            let rhs = parse_value_expr(tokens[2])?;
            return Ok(Some(Stmt::BinOp { dst, lhs, op, rhs }));
        }
        return Err(format!("Invalid assignment stmt: {}", line));
    }

    if lhs.starts_with('[') {
        let place = parse_place_expr(lhs)?;
        let src = parse_value_expr(rhs)?;
        return Ok(Some(Stmt::Store { place, src }));
    }
    Ok(None)
}

fn parse_jump_if(line: &str) -> Result<Option<JumpIf>, String> {
    let line = line.trim();
    if !line.starts_with("if ") || !line.ends_with(';') {
        return Ok(None);
    }
    let body = &line[..line.len() - 1];
    let rest = body
        .strip_prefix("if ")
        .ok_or_else(|| format!("Invalid if syntax: {}", line))?;
    let Some((cond_s, target_s)) = rest.split_once(" then ") else {
        return Err(format!("Invalid if syntax: {}", line));
    };
    Ok(Some(JumpIf {
        cond: parse_cond(cond_s.trim())?,
        target: parse_addr_expr(target_s.trim())?,
    }))
}

fn parse_goto(line: &str) -> Result<Option<AddrExpr>, String> {
    let line = line.trim();
    if !line.starts_with("goto ") || !line.ends_with(';') {
        return Ok(None);
    }
    let body = &line[..line.len() - 1];
    let target = body
        .strip_prefix("goto ")
        .ok_or_else(|| format!("Invalid goto syntax: {}", line))?;
    Ok(Some(parse_addr_expr(target.trim())?))
}

fn parse_cond(s: &str) -> Result<Cond, String> {
    if let Some((a, b)) = s.split_once(" < ") {
        return Ok(Cond::Lt(
            parse_value_expr(a.trim())?,
            parse_value_expr(b.trim())?,
        ));
    }
    if let Some((a, b)) = s.split_once(" = ") {
        return Ok(Cond::Eq(
            parse_value_expr(a.trim())?,
            parse_value_expr(b.trim())?,
        ));
    }
    if let Some((a, b)) = s.split_once(" > ") {
        return Ok(Cond::Eq(
            parse_value_expr(a.trim())?,
            parse_value_expr(b.trim())?,
        ));
    }
    Err(format!("Invalid condition: {}", s))
}

fn parse_addr_expr(s: &str) -> Result<AddrExpr, String> {
    let s = s.trim();
    if let Some(rest) = s.strip_prefix("&") {
        return Ok(AddrExpr::Ref(Box::new(parse_place_expr(rest.trim())?)));
    }
    if s.starts_with("%v") {
        return Ok(AddrExpr::VReg(parse_vreg(s)?));
    }
    if let Some(label) = s.strip_prefix('@') {
        return Ok(AddrExpr::Label(label.to_string()));
    }
    Err(format!("Invalid addr expr: {}", s))
}

fn parse_place_expr(s: &str) -> Result<PlaceExpr, String> {
    let s = s.trim();
    let inner = s
        .strip_prefix('[')
        .and_then(|x| x.strip_suffix(']'))
        .ok_or_else(|| format!("Invalid place expr: {}", s))?;
    Ok(PlaceExpr(Box::new(parse_addr_expr(inner.trim())?)))
}

fn parse_value_expr(s: &str) -> Result<ValueExpr, String> {
    let s = s.trim();
    if let Some(rest) = s.strip_prefix('*') {
        return Ok(ValueExpr::Deref(parse_place_expr(rest.trim())?));
    }
    if s.starts_with("%v") {
        return Ok(ValueExpr::VReg(parse_vreg(s)?));
    }
    if let Some(n) = s.strip_prefix('#') {
        return Ok(ValueExpr::Imm(parse_number(n)?));
    }
    if s.chars().all(|c| c.is_ascii_digit()) {
        return Ok(ValueExpr::Imm(parse_number(s)?));
    }
    Err(format!("Invalid value expr: {}", s))
}

fn parse_vreg(s: &str) -> Result<usize, String> {
    let n = s
        .trim()
        .strip_prefix("%v")
        .ok_or_else(|| format!("Invalid vreg: {}", s))?;
    n.parse::<usize>().map_err(|e| e.to_string())
}

fn parse_number(s: &str) -> Result<Number, String> {
    Number::parse(s)
}

fn value_to_text(v: &ValueExpr) -> String {
    match v {
        ValueExpr::VReg(i) => format!("%v{}", i),
        ValueExpr::Imm(n) => format!("#{}", n.to_decimal_string()),
        ValueExpr::Deref(p) => format!("*{}", place_to_text(p)),
    }
}

fn addr_to_text(a: &AddrExpr) -> String {
    match a {
        AddrExpr::VReg(i) => format!("%v{}", i),
        AddrExpr::Label(l) => format!("@{}", l),
        AddrExpr::Ref(p) => format!("&{}", place_to_text(p)),
    }
}

fn place_to_text(p: &PlaceExpr) -> String {
    format!("[{}]", addr_to_text(&p.0))
}

fn cond_to_text(c: &Cond) -> String {
    match c {
        Cond::Lt(a, b) => format!("{} < {}", value_to_text(a), value_to_text(b)),
        Cond::Eq(a, b) => format!("{} = {}", value_to_text(a), value_to_text(b)),
        Cond::Gt(a, b) => format!("{} > {}", value_to_text(a), value_to_text(b)),
    }
}

fn stmt_to_text(s: &Stmt) -> String {
    match s {
        Stmt::Assign { dst, src } => format!("%v{} := {};", dst, value_to_text(src)),
        Stmt::BinOp { dst, lhs, op, rhs } => {
            let op = match op {
                BinOp::Add => "+",
                BinOp::Sub => "-",
            };
            format!(
                "%v{} := {} {} {};",
                dst,
                value_to_text(lhs),
                op,
                value_to_text(rhs)
            )
        }
        Stmt::Store { place, src } => {
            format!("{} := {};", place_to_text(place), value_to_text(src))
        }
    }
}

fn block_summary_text(block: &Block) -> String {
    let mut parts = Vec::new();
    for s in &block.stmts {
        parts.push(stmt_to_text(s));
    }
    for j in &block.cont.ifs {
        parts.push(format!(
            "if {} then {};",
            cond_to_text(&j.cond),
            addr_to_text(&j.target)
        ));
    }
    parts.push(format!("goto {};", addr_to_text(&block.cont.jump)));
    parts.join(" ")
}
