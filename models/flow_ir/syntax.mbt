///|
#derive(Eq)
pub(all) enum Binop {
  Add
  Sub
}

///|
#derive(Eq)
pub(all) enum ValueExpr {
  VReg(String)
  Imm(@utils.Nat)
  CodeLabel(String)
  Ref(PlaceExpr)
}

///|
#derive(Eq)
pub(all) enum PlaceExpr {
  Label(String)
  Deref(String)
  SAcc(ValueExpr)
  HAcc(ValueExpr, ValueExpr)
}

///|
#derive(Eq)
pub(all) enum Cond {
  Lt(ValueExpr, ValueExpr)
  Eq(ValueExpr, ValueExpr)
  Gt(ValueExpr, ValueExpr)
}

///|
#derive(Eq)
pub(all) enum Stmt {
  Nop
  Load(String, PlaceExpr)
  Assign(String, ValueExpr)
  BinOp(String, ValueExpr, Binop, ValueExpr)
  Store(PlaceExpr, ValueExpr)
  Pop(String)
  Push(ValueExpr)
  LGet(String)
  HAlloc(ValueExpr, String)
  HFree(ValueExpr)
  Print(String)
  Input(PlaceExpr)
}

///|
#derive(Eq)
pub(all) struct JumpIf {
  cond : Cond
  target : ValueExpr
}

///|
#derive(Eq)
pub(all) enum Cont {
  Go(Array[JumpIf], ValueExpr)
  Enter(Array[JumpIf], ValueExpr)
  Halt(Array[JumpIf])
}

///|
#derive(Eq)
pub(all) struct Block {
  label : String
  stmts : Array[Stmt]
  cont : Cont
}

///|
#derive(Eq)
pub(all) struct Region {
  label : String
  blocks : Array[Block]
}

///|
#derive(Eq)
pub(all) struct StaticDef {
  label : String
  value : @utils.Nat
}

///|
#derive(Eq)
pub(all) struct Code {
  statics : Array[StaticDef]
  regions : Array[Region]
}

///|
pub fn ifs(c : Cont) -> Array[JumpIf] {
  match c {
    Go(xs, _) | Enter(xs, _) | Halt(xs) => xs
  }
}

///|
fn vreg(p : @utils.Parser) -> String raise @utils.Fault {
  p.expect('%')
  p.ident()
}

///|
pub fn read_value(p : @utils.Parser) -> ValueExpr raise @utils.Fault {
  if p.eat_keyword("ref") {
    Ref(read_place(p))
  } else if p.peek() == @utils.Symbol('%') {
    VReg(vreg(p))
  } else if p.eat(':') {
    CodeLabel(p.ident())
  } else {
    ignore(p.eat('#'))
    Imm(p.natural())
  }
}

///|
pub fn read_place(p : @utils.Parser) -> PlaceExpr raise @utils.Fault {
  if p.eat('@') {
    Label(p.ident())
  } else if p.eat_keyword("deref") {
    Deref(vreg(p))
  } else if p.eat_keyword("sacc") {
    SAcc(read_value(p))
  } else if p.eat_keyword("hacc") {
    let g = p.group('(')
    let h = read_value(g)
    g.finish()
    let g = p.group('[')
    let i = read_value(g)
    g.finish()
    HAcc(h, i)
  } else {
    p.error("Expected place expression")
  }
}

///|
fn read_cond(p : @utils.Parser) -> Cond raise @utils.Fault {
  let a = read_value(p)
  match p.bump() {
    @utils.Symbol('<') => Lt(a, read_value(p))
    @utils.Symbol('=') => Eq(a, read_value(p))
    @utils.Symbol('>') => Gt(a, read_value(p))
    _ => p.error("Expected comparison")
  }
}

///|
fn read_stmt(p : @utils.Parser) -> Stmt raise @utils.Fault {
  let s = if p.eat_keyword("nop") {
    Nop
  } else if p.eat_keyword("pop") {
    Pop(vreg(p))
  } else if p.eat_keyword("push") {
    Push(read_value(p))
  } else if p.eat_keyword("lget") {
    LGet(vreg(p))
  } else if p.eat_keyword("hfree") {
    HFree(read_value(p))
  } else if p.eat_keyword("print") {
    Print(vreg(p))
  } else if p.eat_keyword("input") {
    Input(read_place(p))
  } else if p.eat_keyword("halloc") {
    let g = p.group('(')
    let v = read_value(g)
    g.finish()
    HAlloc(v, vreg(p))
  } else if p.peek() == @utils.Symbol('%') {
    let r = vreg(p)
    p.expect_text(":=")
    if p.eat_keyword("ld") {
      Load(r, read_place(p))
    } else {
      let a = read_value(p)
      if p.eat('+') {
        BinOp(r, a, Add, read_value(p))
      } else if p.eat('-') {
        BinOp(r, a, Sub, read_value(p))
      } else {
        Assign(r, a)
      }
    }
  } else {
    let place = read_place(p)
    p.expect_text(":= \\st")
    Store(place, read_value(p))
  }
  p.expect(';')
  s
}

///|
fn read_block(p : @utils.Parser) -> Block raise @utils.Fault {
  p.expect(':')
  let label = p.ident()
  let g = p.group('{')
  let stmts = []
  let jumps = []
  let cont : Ref[Cont?] = { val: None, }
  while !g.eof() {
    if g.eat_keyword("if") {
      let cond = read_cond(g)
      g.expect_keyword("then")
      let target = read_value(g)
      g.expect(';')
      jumps.push({ cond, target, })
    } else {
      @utils.require(cont.val is None, "Line exists after terminator")
      if g.eat_keyword("goto") {
        let v = read_value(g)
        g.expect(';')
        cont.val = Some(Go(jumps.copy(), v))
      } else if g.eat_keyword("enter") {
        let v = read_value(g)
        g.expect(';')
        cont.val = Some(Enter(jumps.copy(), v))
      } else if g.eat_keyword("halt") {
        g.expect(';')
        cont.val = Some(Halt(jumps.copy()))
      } else {
        stmts.push(read_stmt(g))
      }
    }
  }
  {
    label,
    stmts,
    cont: match cont.val {
      Some(c) => c
      None => g.error("Missing terminator")
    },
  }
}

///|
pub fn parse(text : String) -> Code raise @utils.Fault {
  let p = @utils.create(text)
  let statics = []
  let regions = []
  while !p.eof() {
    if p.eat('@') {
      let label = p.ident()
      statics.push({ label, value: p.natural(), })
    } else {
      p.expect(':')
      let label = p.ident()
      let g = p.group('{')
      let blocks = []
      while !g.eof() {
        blocks.push(read_block(g))
      }
      @utils.require(!blocks.is_empty(), "Region has no blocks")
      regions.push({ label, blocks, })
    }
  }
  { statics, regions, }
}

///|
pub fn value_text(v : ValueExpr) -> String {
  match v {
    VReg(r) => "%" + r
    Imm(n) => "#" + n.to_string()
    CodeLabel(l) => ":" + l
    Ref(p) => "\\ref " + place_text(p)
  }
}

///|
pub fn place_text(p : PlaceExpr) -> String {
  match p {
    Label(l) => "@" + l
    Deref(r) => "\\deref %" + r
    SAcc(v) => "\\sacc " + value_text(v)
    HAcc(h, i) => "\\hacc(" + value_text(h) + ")[" + value_text(i) + "]"
  }
}

///|
pub fn cond_text(c : Cond) -> String {
  let (a, op, b) = match c {
    Lt(a, b) => (a, "<", b)
    Eq(a, b) => (a, "=", b)
    Gt(a, b) => (a, ">", b)
  }
  value_text(a) + " " + op + " " + value_text(b)
}

///|
pub fn stmt_text(s : Stmt) -> String {
  match s {
    Nop => "\\nop;"
    Load(r, p) => "%" + r + " := \\ld " + place_text(p) + ";"
    Assign(r, v) => "%" + r + " := " + value_text(v) + ";"
    BinOp(r, a, o, b) =>
      "%" +
      r +
      " := " +
      value_text(a) +
      (match o {
        Add => " + "
        Sub => " - "
      }) +
      value_text(b) +
      ";"
    Store(p, v) => place_text(p) + " := \\st " + value_text(v) + ";"
    Pop(r) => "\\pop %" + r + ";"
    Push(v) => "\\push " + value_text(v) + ";"
    LGet(r) => "\\lget %" + r + ";"
    HAlloc(v, r) => "\\halloc(" + value_text(v) + ") %" + r + ";"
    HFree(v) => "\\hfree " + value_text(v) + ";"
    Print(r) => "\\print %" + r + ";"
    Input(p) => "\\input " + place_text(p) + ";"
  }
}

///|
pub fn print_code(c : Code) -> String {
  let mut text = @utils.join(
    c.statics.map(s => "@" + s.label + " " + s.value.to_string() + "\n"),
    "",
  )
  for r in c.regions {
    text += ":" + r.label + " {\n"
    for b in r.blocks {
      text += "  :" + b.label + " {\n"
      for s in b.stmts {
        text += "    " + stmt_text(s) + "\n"
      }
      for j in ifs(b.cont) {
        text += "    \\if " +
          cond_text(j.cond) +
          " \\then " +
          value_text(j.target) +
          ";\n"
      }
      text += "    " +
        (match b.cont {
          Go(_, v) => "\\goto " + value_text(v) + ";"
          Enter(_, v) => "\\enter " + value_text(v) + ";"
          Halt(_) => "\\halt;"
        }) +
        "\n  }\n"
    }
    text += "}\n"
  }
  text
}
