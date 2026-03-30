use utils::number::Number;
use utils::{TextCodec, Token as LexToken, lex};

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

fn parse_program(text: &str) -> Result<Program, String> {
    let tokens = lex(text).map_err(|e| e.to_string())?;
    let mut parser = Parser::new(tokens);
    let mut statics = Vec::new();
    let mut regions = Vec::new();
    while !parser.is_eof() {
        if parser.peek_symbol('@') {
            let (label, value) = parser.parse_static_def()?;
            statics.push(StaticDef { label, value });
            continue;
        }
        if parser.peek_symbol(':') {
            regions.push(parser.parse_region()?);
            continue;
        }
        return Err(parser.error_here("expected static definition or region header"));
    }
    Ok(Program { statics, regions })
}

struct Parser {
    tokens: Vec<LexToken>,
    pos: usize,
}

impl Parser {
    fn new(tokens: Vec<LexToken>) -> Self {
        let tokens = tokens
            .into_iter()
            .filter(|token| !matches!(token, LexToken::Whitespace(_) | LexToken::Comment(_)))
            .collect();
        Self { tokens, pos: 0 }
    }

    fn is_eof(&self) -> bool {
        self.pos >= self.tokens.len()
    }

    fn peek(&self) -> Option<&LexToken> {
        self.tokens.get(self.pos)
    }

    fn next(&mut self) -> Option<LexToken> {
        let token = self.tokens.get(self.pos).cloned();
        if token.is_some() {
            self.pos += 1;
        }
        token
    }

    fn error_here(&self, message: impl Into<String>) -> String {
        match self.peek() {
            Some(token) => format!("{} near {:?}", message.into(), token),
            None => format!("{} at end of input", message.into()),
        }
    }

    fn peek_symbol(&self, ch: char) -> bool {
        matches!(self.peek(), Some(LexToken::Symbol(found)) if *found == ch)
    }

    fn eat_symbol(&mut self, ch: char) -> bool {
        if self.peek_symbol(ch) {
            self.pos += 1;
            true
        } else {
            false
        }
    }

    fn expect_symbol(&mut self, ch: char) -> Result<(), String> {
        if self.eat_symbol(ch) {
            Ok(())
        } else {
            Err(self.error_here(format!("expected symbol '{}'", ch)))
        }
    }

    fn peek_ident(&self, word: &str) -> bool {
        matches!(self.peek(), Some(LexToken::Ident(found)) if found == word)
    }

    fn eat_ident(&mut self, word: &str) -> bool {
        if self.peek_ident(word) {
            self.pos += 1;
            true
        } else {
            false
        }
    }

    fn expect_ident(&mut self, word: &str) -> Result<(), String> {
        if self.eat_ident(word) {
            Ok(())
        } else {
            Err(self.error_here(format!("expected keyword '{word}'")))
        }
    }

    fn parse_name(&mut self, kind: &str) -> Result<String, String> {
        match self.next() {
            Some(LexToken::Ident(name)) | Some(LexToken::Number(name)) => Ok(name),
            _ => Err(self.error_here(format!("expected {kind}"))),
        }
    }

    fn parse_static_def(&mut self) -> Result<(String, Number), String> {
        self.expect_symbol('@')?;
        let label = self.parse_name("static label")?;
        let value = self.parse_number_literal()?;
        Ok((label, value))
    }

    fn parse_region(&mut self) -> Result<Region, String> {
        self.expect_symbol(':')?;
        let label = self.parse_name("region label")?;
        self.expect_symbol('{')?;

        let mut blocks = Vec::new();
        while !self.peek_symbol('}') {
            blocks.push(self.parse_block()?);
        }
        self.expect_symbol('}')?;

        if blocks.is_empty() {
            return Err(format!("Region :{} has no blocks", label));
        }
        Ok(Region { label, blocks })
    }

    fn parse_block(&mut self) -> Result<Block, String> {
        self.expect_symbol(':')?;
        let label = self.parse_name("block label")?;
        self.expect_symbol('{')?;

        let mut stmts = Vec::new();
        let mut ifs = Vec::new();
        let mut cont = None;

        while !self.peek_symbol('}') {
            if self.peek_ident("if") {
                ifs.push(self.parse_jump_if()?);
                continue;
            }

            if cont.is_none() && self.peek_cont_start() {
                cont = Some(self.parse_cont(&ifs)?);
                continue;
            }

            if cont.is_some() {
                return Err(format!("Line exists after terminator in block :{}", label));
            }

            stmts.push(self.parse_stmt()?);
        }

        self.expect_symbol('}')?;
        let cont = cont.ok_or_else(|| format!("Missing terminator in block :{}", label))?;
        Ok(Block { label, stmts, cont })
    }

    fn peek_cont_start(&self) -> bool {
        self.peek_ident("goto") || self.peek_ident("enter") || self.peek_ident("halt")
    }

    fn parse_stmt(&mut self) -> Result<Stmt, String> {
        if self.eat_ident("nop") {
            self.expect_symbol(';')?;
            return Ok(Stmt::Nop);
        }
        if self.eat_ident("pop") {
            let dst = self.parse_vreg()?;
            self.expect_symbol(';')?;
            return Ok(Stmt::Pop { dst });
        }
        if self.eat_ident("push") {
            let src = self.parse_value_expr()?;
            self.expect_symbol(';')?;
            return Ok(Stmt::Push { src });
        }
        if self.eat_ident("lget") {
            let dst = self.parse_vreg()?;
            self.expect_symbol(';')?;
            return Ok(Stmt::LGet { dst });
        }
        if self.eat_ident("halloc") {
            self.expect_symbol('(')?;
            let size = self.parse_value_expr()?;
            self.expect_symbol(')')?;
            let dst = self.parse_vreg()?;
            self.expect_symbol(';')?;
            return Ok(Stmt::HAlloc { size, dst });
        }
        if self.eat_ident("hfree") {
            let handle = self.parse_value_expr()?;
            self.expect_symbol(';')?;
            return Ok(Stmt::HFree { handle });
        }
        if self.eat_ident("print") {
            let src = self.parse_vreg()?;
            self.expect_symbol(';')?;
            return Ok(Stmt::Print { src });
        }
        if self.eat_ident("input") {
            let place = self.parse_place_expr()?;
            self.expect_symbol(';')?;
            return Ok(Stmt::Input { place });
        }

        if self.peek_symbol('%') {
            let dst = self.parse_vreg()?;
            self.expect_symbol(':')?;
            self.expect_symbol('=')?;

            if self.eat_ident("ld") {
                let place = self.parse_place_expr()?;
                self.expect_symbol(';')?;
                return Ok(Stmt::Load { dst, place });
            }

            let lhs = self.parse_value_expr()?;
            if self.eat_symbol('+') {
                let rhs = self.parse_value_expr()?;
                self.expect_symbol(';')?;
                return Ok(Stmt::BinOp {
                    dst,
                    lhs,
                    op: BinOp::Add,
                    rhs,
                });
            }
            if self.eat_symbol('-') {
                let rhs = self.parse_value_expr()?;
                self.expect_symbol(';')?;
                return Ok(Stmt::BinOp {
                    dst,
                    lhs,
                    op: BinOp::Sub,
                    rhs,
                });
            }

            self.expect_symbol(';')?;
            return Ok(Stmt::Assign { dst, src: lhs });
        }

        let place = self.parse_place_expr()?;
        self.expect_symbol(':')?;
        self.expect_symbol('=')?;
        self.expect_ident("st")?;
        let src = self.parse_value_expr()?;
        self.expect_symbol(';')?;
        Ok(Stmt::Store { place, src })
    }

    fn parse_jump_if(&mut self) -> Result<JumpIf, String> {
        self.expect_ident("if")?;
        let cond = self.parse_cond()?;
        self.expect_ident("then")?;
        let target = self.parse_value_expr()?;
        self.expect_symbol(';')?;
        Ok(JumpIf { cond, target })
    }

    fn parse_cont(&mut self, ifs: &[JumpIf]) -> Result<Cont, String> {
        if self.eat_ident("goto") {
            let target = self.parse_value_expr()?;
            self.expect_symbol(';')?;
            return Ok(Cont::Go {
                ifs: ifs.to_vec(),
                target,
            });
        }
        if self.eat_ident("enter") {
            let target = self.parse_value_expr()?;
            self.expect_symbol(';')?;
            return Ok(Cont::Enter {
                ifs: ifs.to_vec(),
                target,
            });
        }
        if self.eat_ident("halt") {
            self.expect_symbol(';')?;
            return Ok(Cont::Halt { ifs: ifs.to_vec() });
        }
        Err(self.error_here("expected block terminator"))
    }

    fn parse_cond(&mut self) -> Result<Cond, String> {
        let lhs = self.parse_value_expr()?;
        if self.eat_symbol('<') {
            let rhs = self.parse_value_expr()?;
            return Ok(Cond::Lt(lhs, rhs));
        }
        if self.eat_symbol('=') {
            let rhs = self.parse_value_expr()?;
            return Ok(Cond::Eq(lhs, rhs));
        }
        if self.eat_symbol('>') {
            let rhs = self.parse_value_expr()?;
            return Ok(Cond::Gt(lhs, rhs));
        }
        Err(self.error_here("expected relation operator"))
    }

    fn parse_place_expr(&mut self) -> Result<PlaceExpr, String> {
        if self.eat_symbol('@') {
            let label = self.parse_name("data label")?;
            return Ok(PlaceExpr::Label(label));
        }
        if self.eat_ident("deref") {
            return Ok(PlaceExpr::Deref(self.parse_vreg()?));
        }
        if self.eat_ident("sacc") {
            return Ok(PlaceExpr::SAcc(Box::new(self.parse_value_expr()?)));
        }
        if self.eat_ident("hacc") {
            self.expect_symbol('(')?;
            let handle = self.parse_value_expr()?;
            self.expect_symbol(')')?;
            self.expect_symbol('[')?;
            let index = self.parse_value_expr()?;
            self.expect_symbol(']')?;
            return Ok(PlaceExpr::HAcc {
                handle: Box::new(handle),
                index: Box::new(index),
            });
        }
        Err(self.error_here("expected place expression"))
    }

    fn parse_value_expr(&mut self) -> Result<ValueExpr, String> {
        if self.eat_ident("ref") {
            return Ok(ValueExpr::Ref(self.parse_place_expr()?));
        }
        if self.peek_symbol('%') {
            return Ok(ValueExpr::VReg(self.parse_vreg()?));
        }
        if self.eat_symbol(':') {
            let label = self.parse_name("code label")?;
            return Ok(ValueExpr::CodeLabel(label));
        }
        if self.eat_symbol('#') {
            return Ok(ValueExpr::Imm(self.parse_number_literal()?));
        }
        if matches!(self.peek(), Some(LexToken::Number(_))) {
            return Ok(ValueExpr::Imm(self.parse_number_literal()?));
        }
        Err(self.error_here("expected value expression"))
    }

    fn parse_number_literal(&mut self) -> Result<Number, String> {
        match self.next() {
            Some(LexToken::Number(text)) => Number::parse(&text),
            _ => Err(self.error_here("expected number literal")),
        }
    }

    fn parse_vreg(&mut self) -> Result<Vreg, String> {
        self.expect_symbol('%')?;
        self.parse_name("virtual register")
    }
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
        Stmt::Print { src } => format!("print %{src};"),
        Stmt::Input { place } => format!("input {};", place_to_text(place)),
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
    print %x;
    input @slot;
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
    fn flow_ir_parses_comments_and_dense_spacing() {
        roundtrip_code(
            r#"
@slot 0 // static
/* region comment */
:main{
  :entry{
    %p:=ref @slot;
    if %cond=#0 then :next;
    goto %p;
  }
  :next{
    halloc(#3)%h;
    hacc(%h)[#1]:=st #7;
    %x:=ld hacc(%h)[#1];
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

    #[test]
    fn flow_value_rejects_old_static_key_syntax() {
        let err = FlowValue::parse("@x").unwrap_err();
        assert_eq!(err, "invalid digit found in string");
    }
}
