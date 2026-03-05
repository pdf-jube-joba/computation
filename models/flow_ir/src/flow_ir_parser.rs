use serde_json::json;
use utils::number::Number;
use utils::{TextCodec, json_text};

use crate::flow_ir::*;

impl TextCodec for FlowIrCode {
    fn parse(text: &str) -> Result<Self, String> {
        Ok(Self(parse_program(text)?))
    }

    fn write_fmt(&self, f: &mut impl std::fmt::Write) -> std::fmt::Result {
        for s in &self.0.statics {
            writeln!(f, "@{} {}", s.label, s.value.to_decimal_string())?;
        }
        for region in &self.0.regions {
            writeln!(f, ":{} {{", region.label)?;
            for block in &region.blocks {
                writeln!(f, "  :{} {{", block.label)?;
                for stmt in &block.stmts {
                    writeln!(f, "    {}", stmt_to_text(stmt))?;
                }
                for j in block.cont.ifs() {
                    writeln!(
                        f,
                        "    if {} then {};",
                        cond_to_text(&j.cond),
                        value_to_text(&j.target)
                    )?;
                }
                match &block.cont {
                    Cont::Go { target, .. } => {
                        writeln!(f, "    goto {};", value_to_text(target))?;
                    }
                    Cont::Enter { target, .. } => {
                        writeln!(f, "    enter {};", value_to_text(target))?;
                    }
                    Cont::Halt { .. } => {
                        writeln!(f, "    halt;")?;
                    }
                }
                writeln!(f, "  }}")?;
            }
            writeln!(f, "}}")?;
        }
        Ok(())
    }
}

impl TextCodec for FlowValue {
    fn parse(text: &str) -> Result<Self, String> {
        let s = text.trim();
        if let Some(label) = s.strip_prefix(':') {
            if label.is_empty() {
                return Err("Empty code label".to_string());
            }
            return Ok(FlowValue::CodeLabel(label.to_string()));
        }
        if let Some(label) = s.strip_prefix("k:@") {
            if label.is_empty() {
                return Err("Empty static key".to_string());
            }
            return Ok(FlowValue::Key(PlaceKey::Static(label.to_string())));
        }
        if let Some(label) = s.strip_prefix('@') {
            if label.is_empty() {
                return Err("Empty static key".to_string());
            }
            return Ok(FlowValue::Key(PlaceKey::Static(label.to_string())));
        }
        if let Some(body) = s.strip_prefix("k:s[") {
            let idx = body
                .strip_suffix(']')
                .ok_or_else(|| format!("Invalid stack key: {s}"))?
                .parse::<usize>()
                .map_err(|e| e.to_string())?;
            return Ok(FlowValue::Key(PlaceKey::Stack(idx)));
        }
        if let Some(body) = s.strip_prefix("k:h[") {
            let Some((h, rest)) = body.split_once("][") else {
                return Err(format!("Invalid heap key: {s}"));
            };
            let i = rest
                .strip_suffix(']')
                .ok_or_else(|| format!("Invalid heap key: {s}"))?;
            let handle = h.parse::<usize>().map_err(|e| e.to_string())?;
            let index = i.parse::<usize>().map_err(|e| e.to_string())?;
            return Ok(FlowValue::Key(PlaceKey::Heap { handle, index }));
        }
        Ok(FlowValue::Num(Number::parse(s)?))
    }

    fn write_fmt(&self, f: &mut impl std::fmt::Write) -> std::fmt::Result {
        write!(f, "{}", self.print())
    }
}

impl TextCodec for StaticEnv {
    fn parse(text: &str) -> Result<Self, String> {
        let mut entries = std::collections::BTreeMap::new();
        for line in text.lines() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }
            let Some((left, right)) = line.split_once('=') else {
                return Err(format!("Invalid static input line: {line}"));
            };
            let key = left.trim().to_string();
            if key.is_empty() {
                return Err("Empty static input key".to_string());
            }
            let value = FlowValue::parse(right.trim())?;
            entries.insert(key, value);
        }
        Ok(StaticEnv { entries })
    }

    fn write_fmt(&self, f: &mut impl std::fmt::Write) -> std::fmt::Result {
        for (k, v) in &self.entries {
            writeln!(f, "{} = {}", k, v.print())?;
        }
        Ok(())
    }
}

impl From<FlowIrMachine> for serde_json::Value {
    fn from(machine: FlowIrMachine) -> Self {
        let mut blocks = Vec::new();
        let region = machine
            .code
            .0
            .regions
            .get(machine.current_region)
            .map(|r| r.label.clone())
            .unwrap_or_else(|| "<invalid>".to_string());
        let block = machine
            .code
            .0
            .regions
            .get(machine.current_region)
            .and_then(|rr| rr.blocks.get(machine.current_block))
            .map(|b| b.label.clone())
            .unwrap_or_else(|| "<invalid>".to_string());

        blocks.push(json_text!(format!(":{}", region), title: "current_region"));
        blocks.push(json_text!(format!(":{}", block), title: "current_block"));
        blocks.push(json_text!(machine.halted.to_string(), title: "halted"));

        let vreg_rows: Vec<serde_json::Value> = machine
            .vregs
            .iter()
            .map(|(name, v)| json!({ "cells": [json_text!(format!("%{}", name)), json_text!(v.print())] }))
            .collect();
        blocks.push(json!({
            "kind": "table",
            "title": "vregs",
            "columns": [json_text!("vreg"), json_text!("value")],
            "rows": vreg_rows
        }));

        let static_rows: Vec<serde_json::Value> = machine
            .code
            .0
            .statics
            .iter()
            .map(|s| {
                let value = machine
                    .static_mem
                    .get(&s.label)
                    .cloned()
                    .unwrap_or_default()
                    .print();
                json!({ "cells": [json_text!(format!("@{}", s.label)), json_text!(value)] })
            })
            .collect();
        blocks.push(json!({
            "kind": "table",
            "title": "static",
            "columns": [json_text!("label"), json_text!("value")],
            "rows": static_rows
        }));

        let stack_rows: Vec<serde_json::Value> = machine
            .stack
            .iter()
            .enumerate()
            .map(|(i, v)| json!({ "cells": [json_text!(i.to_string()), json_text!(v.print())] }))
            .collect();
        blocks.push(json!({
            "kind": "table",
            "title": "stack",
            "columns": [json_text!("index"), json_text!("value")],
            "rows": stack_rows
        }));

        let heap_rows: Vec<serde_json::Value> = machine
            .heap
            .iter()
            .flat_map(|(h, area)| {
                area.iter().enumerate().map(move |(i, v)| {
                    json!({ "cells": [json_text!(h.to_string()), json_text!(i.to_string()), json_text!(v.print())] })
                })
            })
            .collect();
        blocks.push(json!({
            "kind": "table",
            "title": "heap",
            "columns": [json_text!("handle"), json_text!("index"), json_text!("value")],
            "rows": heap_rows
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
    let mut regions = Vec::new();
    while i < lines.len() {
        let line = lines[i].trim();
        if let Some((label, value)) = parse_static_line(line)? {
            statics.push(StaticDef { label, value });
            i += 1;
            continue;
        }
        if is_header(line) {
            let (region, next_i) = parse_region(&lines, i)?;
            regions.push(region);
            i = next_i;
            continue;
        }
        return Err(format!("Unexpected top-level line: {line}"));
    }
    Ok(Program { statics, regions })
}

fn parse_region(lines: &[String], start: usize) -> Result<(Region, usize), String> {
    let label = parse_header_label(lines[start].trim())?;
    let mut i = start + 1;
    let mut blocks = Vec::new();

    while i < lines.len() {
        let line = lines[i].trim();
        if line == "}" {
            break;
        }
        if is_header(line) {
            let (block, next_i) = parse_block(lines, i)?;
            blocks.push(block);
            i = next_i;
            continue;
        }
        return Err(format!("Invalid region body line :{}: {}", label, line));
    }

    if i >= lines.len() || lines[i].trim() != "}" {
        return Err(format!("Unclosed region: :{}", label));
    }
    if blocks.is_empty() {
        return Err(format!("Region :{} has no blocks", label));
    }

    Ok((Region { label, blocks }, i + 1))
}

fn parse_block(lines: &[String], start: usize) -> Result<(Block, usize), String> {
    let label = parse_header_label(lines[start].trim())?;
    let mut i = start + 1;
    let mut stmts = Vec::new();
    let mut ifs = Vec::new();
    let mut cont = None;

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
        if cont.is_none() {
            if let Some(c) = parse_cont(line, &ifs)? {
                cont = Some(c);
                i += 1;
                continue;
            }
            if let Some(stmt) = parse_stmt(line)? {
                stmts.push(stmt);
                i += 1;
                continue;
            }
            return Err(format!("Invalid line in block :{}: {}", label, line));
        }
        return Err(format!(
            "Line exists after terminator in block :{}: {}",
            label, line
        ));
    }

    if i >= lines.len() || lines[i].trim() != "}" {
        return Err(format!("Unclosed block: :{}", label));
    }
    let cont = cont.ok_or_else(|| format!("Missing terminator in block :{}", label))?;
    Ok((Block { label, stmts, cont }, i + 1))
}

fn parse_static_line(line: &str) -> Result<Option<(String, Number)>, String> {
    if is_header(line) {
        return Ok(None);
    }
    let parts: Vec<_> = line.split_whitespace().collect();
    if parts.len() != 2 || !parts[0].starts_with('@') {
        return Ok(None);
    }
    let label = parts[0]
        .strip_prefix('@')
        .ok_or_else(|| "Invalid static label".to_string())?
        .to_string();
    let value = Number::parse(parts[1])?;
    Ok(Some((label, value)))
}

fn is_header(line: &str) -> bool {
    line.starts_with(':') && line.ends_with('{')
}

fn parse_header_label(line: &str) -> Result<String, String> {
    line.strip_prefix(':')
        .and_then(|s| s.strip_suffix('{'))
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
        .ok_or_else(|| format!("Invalid header: {line}"))
}

fn parse_stmt(line: &str) -> Result<Option<Stmt>, String> {
    let line = line.trim();
    if line == "nop;" {
        return Ok(Some(Stmt::Nop));
    }
    if let Some(rest) = line.strip_prefix("pop ") {
        let body = trim_stmt_end(rest)?;
        return Ok(Some(Stmt::Pop {
            dst: parse_vreg(body)?,
        }));
    }
    if let Some(rest) = line.strip_prefix("push ") {
        let body = trim_stmt_end(rest)?;
        return Ok(Some(Stmt::Push {
            src: parse_value_expr(body)?,
        }));
    }
    if let Some(rest) = line.strip_prefix("lget ") {
        let body = trim_stmt_end(rest)?;
        return Ok(Some(Stmt::LGet {
            dst: parse_vreg(body)?,
        }));
    }
    if let Some(rest) = line.strip_prefix("halloc(") {
        let body = trim_stmt_end(rest)?;
        let close = body
            .find(')')
            .ok_or_else(|| format!("halloc missing ')': {line}"))?;
        let size = parse_value_expr(body[..close].trim())?;
        let dst = parse_vreg(body[close + 1..].trim())?;
        return Ok(Some(Stmt::HAlloc { size, dst }));
    }
    if let Some(rest) = line.strip_prefix("hfree ") {
        let body = trim_stmt_end(rest)?;
        return Ok(Some(Stmt::HFree {
            handle: parse_value_expr(body)?,
        }));
    }

    if !line.ends_with(';') {
        return Ok(None);
    }
    let body = &line[..line.len() - 1];
    let Some((lhs, rhs)) = body.split_once(":=") else {
        return Ok(None);
    };
    let lhs = lhs.trim();
    let rhs = rhs.trim();

    if lhs.starts_with('%') {
        let dst = parse_vreg(lhs)?;
        if let Some(rest) = rhs.strip_prefix("ld ") {
            let place = parse_place_expr(rest.trim())?;
            return Ok(Some(Stmt::Load { dst, place }));
        }
        if let Some((a, b)) = rhs.split_once(" + ") {
            return Ok(Some(Stmt::BinOp {
                dst,
                lhs: parse_value_expr(a.trim())?,
                op: BinOp::Add,
                rhs: parse_value_expr(b.trim())?,
            }));
        }
        if let Some((a, b)) = rhs.split_once(" - ") {
            return Ok(Some(Stmt::BinOp {
                dst,
                lhs: parse_value_expr(a.trim())?,
                op: BinOp::Sub,
                rhs: parse_value_expr(b.trim())?,
            }));
        }
        return Ok(Some(Stmt::Assign {
            dst,
            src: parse_value_expr(rhs)?,
        }));
    }

    let place = parse_place_expr(lhs)?;
    let src = rhs
        .strip_prefix("st ")
        .ok_or_else(|| format!("store stmt requires 'st': {line}"))?;
    Ok(Some(Stmt::Store {
        place,
        src: parse_value_expr(src.trim())?,
    }))
}

fn parse_jump_if(line: &str) -> Result<Option<JumpIf>, String> {
    if !line.starts_with("if ") || !line.ends_with(';') {
        return Ok(None);
    }
    let body = &line[..line.len() - 1];
    let rest = body
        .strip_prefix("if ")
        .ok_or_else(|| format!("Invalid if syntax: {line}"))?;
    let Some((cond_s, target_s)) = rest.split_once(" then ") else {
        return Err(format!("Invalid if syntax: {line}"));
    };
    Ok(Some(JumpIf {
        cond: parse_cond(cond_s.trim())?,
        target: parse_value_expr(target_s.trim())?,
    }))
}

fn parse_cont(line: &str, ifs: &[JumpIf]) -> Result<Option<Cont>, String> {
    if let Some(rest) = line.strip_prefix("goto ") {
        let body = trim_stmt_end(rest)?;
        return Ok(Some(Cont::Go {
            ifs: ifs.to_vec(),
            target: parse_value_expr(body)?,
        }));
    }
    if let Some(rest) = line.strip_prefix("enter ") {
        let body = trim_stmt_end(rest)?;
        return Ok(Some(Cont::Enter {
            ifs: ifs.to_vec(),
            target: parse_value_expr(body)?,
        }));
    }
    if line == "halt;" {
        return Ok(Some(Cont::Halt { ifs: ifs.to_vec() }));
    }
    Ok(None)
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
        return Ok(Cond::Gt(
            parse_value_expr(a.trim())?,
            parse_value_expr(b.trim())?,
        ));
    }
    Err(format!("Invalid condition: {s}"))
}

fn parse_place_expr(s: &str) -> Result<PlaceExpr, String> {
    let s = s.trim();
    if let Some(label) = s.strip_prefix('@') {
        if label.is_empty() {
            return Err("Empty data-label".to_string());
        }
        return Ok(PlaceExpr::Label(label.to_string()));
    }
    if let Some(rest) = s.strip_prefix("deref ") {
        return Ok(PlaceExpr::Deref(parse_vreg(rest.trim())?));
    }
    if let Some(rest) = s.strip_prefix("sacc ") {
        return Ok(PlaceExpr::SAcc(Box::new(parse_value_expr(rest.trim())?)));
    }
    if let Some(rest) = s.strip_prefix("hacc(") {
        let close = rest
            .find(')')
            .ok_or_else(|| format!("hacc missing ')': {s}"))?;
        let handle = parse_value_expr(rest[..close].trim())?;
        let suffix = rest[close + 1..].trim();
        let idx_inner = suffix
            .strip_prefix('[')
            .and_then(|x| x.strip_suffix(']'))
            .ok_or_else(|| format!("hacc index form should be [expr]: {s}"))?;
        let index = parse_value_expr(idx_inner.trim())?;
        return Ok(PlaceExpr::HAcc {
            handle: Box::new(handle),
            index: Box::new(index),
        });
    }
    Err(format!("Invalid place expression: {s}"))
}

fn parse_value_expr(s: &str) -> Result<ValueExpr, String> {
    let s = s.trim();
    if let Some(rest) = s.strip_prefix("ref ") {
        return Ok(ValueExpr::Ref(parse_place_expr(rest.trim())?));
    }
    if s.starts_with('%') {
        return Ok(ValueExpr::VReg(parse_vreg(s)?));
    }
    if let Some(label) = s.strip_prefix(':') {
        if label.is_empty() {
            return Err("Empty code-label".to_string());
        }
        return Ok(ValueExpr::CodeLabel(label.to_string()));
    }
    if let Some(n) = s.strip_prefix('#') {
        return Ok(ValueExpr::Imm(Number::parse(n)?));
    }
    if s.chars().all(|c| c.is_ascii_digit()) {
        return Ok(ValueExpr::Imm(Number::parse(s)?));
    }
    Err(format!("Invalid value expression: {s}"))
}

fn parse_vreg(s: &str) -> Result<Vreg, String> {
    let body = s
        .trim()
        .strip_prefix('%')
        .ok_or_else(|| format!("Invalid vreg: {s}"))?;
    if body.is_empty() {
        return Err(format!("Invalid vreg: {s}"));
    }
    Ok(body.to_string())
}

fn trim_stmt_end(s: &str) -> Result<&str, String> {
    let s = s.trim();
    s.strip_suffix(';')
        .map(str::trim)
        .ok_or_else(|| format!("Statement should end with ';': {s}"))
}

fn value_to_text(v: &ValueExpr) -> String {
    match v {
        ValueExpr::VReg(i) => format!("%{i}"),
        ValueExpr::Imm(n) => format!("#{}", n.to_decimal_string()),
        ValueExpr::CodeLabel(l) => format!(":{l}"),
        ValueExpr::Ref(p) => format!("ref {}", place_to_text(p)),
    }
}

fn place_to_text(p: &PlaceExpr) -> String {
    match p {
        PlaceExpr::Label(label) => format!("@{label}"),
        PlaceExpr::Deref(vreg) => format!("deref %{vreg}"),
        PlaceExpr::SAcc(v) => format!("sacc {}", value_to_text(v)),
        PlaceExpr::HAcc { handle, index } => {
            format!("hacc({})[{}]", value_to_text(handle), value_to_text(index))
        }
    }
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
        Stmt::Nop => "nop;".to_string(),
        Stmt::Load { dst, place } => format!("%{dst} := ld {};", place_to_text(place)),
        Stmt::Assign { dst, src } => format!("%{dst} := {};", value_to_text(src)),
        Stmt::BinOp { dst, lhs, op, rhs } => {
            let op = match op {
                BinOp::Add => "+",
                BinOp::Sub => "-",
            };
            format!(
                "%{dst} := {} {} {};",
                value_to_text(lhs),
                op,
                value_to_text(rhs)
            )
        }
        Stmt::Store { place, src } => {
            format!("{} := st {};", place_to_text(place), value_to_text(src))
        }
        Stmt::Pop { dst } => format!("pop %{dst};"),
        Stmt::Push { src } => format!("push {};", value_to_text(src)),
        Stmt::LGet { dst } => format!("lget %{dst};"),
        Stmt::HAlloc { size, dst } => format!("halloc({}) %{dst};", value_to_text(size)),
        Stmt::HFree { handle } => format!("hfree {};", value_to_text(handle)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn roundtrip_code(text: &str) {
        let parsed = FlowIrCode::parse(text).expect("parse should succeed");
        let printed = parsed.print();
        let reparsed = FlowIrCode::parse(&printed).expect("reparse should succeed");
        assert_eq!(parsed, reparsed, "roundtrip failed:\n{printed}");
    }

    #[test]
    fn flow_ir_code_roundtrip() {
        roundtrip_code(
            r#"
@slot 0
:main {
  :entry {
    nop;
    %p := ref @slot;
    deref %p := st :next;
    %jmp := ld @slot;
    if %cond = #0 then :next;
    goto %jmp;
  }
  :next {
    halloc(#3) %h;
    hacc(%h)[#1] := st #7;
    %x := ld hacc(%h)[#1];
    push %x;
    halt;
  }
}
:r2 {
  :entry {
    lget %len;
    pop %tmp;
    push %len;
    hfree #1;
    halt;
  }
}
"#,
        );
    }

    #[test]
    fn flow_value_roundtrip() {
        let values = vec![
            FlowValue::Num(Number::from(0usize)),
            FlowValue::Num(Number::from(42usize)),
            FlowValue::CodeLabel("main".to_string()),
            FlowValue::Key(PlaceKey::Static("x".to_string())),
            FlowValue::Key(PlaceKey::Stack(3)),
            FlowValue::Key(PlaceKey::Heap {
                handle: 7,
                index: 9,
            }),
        ];
        for v in values {
            let printed = v.print();
            let reparsed = FlowValue::parse(&printed).expect("FlowValue parse should succeed");
            assert_eq!(v, reparsed, "FlowValue roundtrip failed: {printed}");
        }
    }
}
