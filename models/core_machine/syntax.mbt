///|
pub(all) enum ValueExpr {
  Reg(String)
  Imm(@base.Nat)
  Label(String)
  Memory(Int32)
}

///|
pub(all) enum Place {
  Deref(ValueExpr)
}

///|
pub(all) enum Binop {
  Add
  Sub
}

///|
pub(all) enum Rhs {
  Value(ValueExpr)
  Binop(ValueExpr, Binop, ValueExpr)
  Load(Place)
  Slice(ValueExpr, ValueExpr, ValueExpr)
  Addr(ValueExpr, ValueExpr)
  Len(ValueExpr)
}

///|
pub(all) enum Stmt {
  Assign(String, Rhs)
  Store(Place, ValueExpr)
  Print(ValueExpr)
  Input(Place)
}

///|
#derive(Eq)
pub(all) enum Rel {
  Eq
  Lt
  Gt
}

///|
pub type Cond = (ValueExpr, Rel, ValueExpr)

///|
pub(all) struct JumpIf {
  cond : Cond
  target : ValueExpr
}

///|
pub(all) enum Ending {
  Goto(ValueExpr)
  Halt
}

///|
pub(all) struct Block {
  label : String
  stmts : Array[Stmt]
  ifs : Array[JumpIf]
  ending : Ending
}

///|
pub(all) struct Code {
  memories : Int32
  blocks : Array[Block]
}

///|
fn reg(p : @p.Parser) -> String raise @base.Fault {
  p.expect('%')
  p.ident()
}

///|
fn read_value(p : @p.Parser) -> ValueExpr raise @base.Fault {
  if p.peek() == @p.Symbol('%') {
    Reg(reg(p))
  } else if p.eat(':') {
    Label(p.ident())
  } else {
    match p.peek() {
      @p.Ident(n) if n.has_prefix("MEM_") => {
        ignore(p.bump())
        Memory(
          @base.index(@base.nat(n.unsafe_substring(start=4, end=n.length()))),
        )
      }
      _ => {
        ignore(p.eat('#'))
        Imm(p.natural())
      }
    }
  }
}

///|
fn read_place(p : @p.Parser) -> Place raise @base.Fault {
  p.expect_keyword("deref")
  Deref(read_value(p))
}

///|
fn read_target(p : @p.Parser) -> ValueExpr raise @base.Fault {
  match read_value(p) {
    Reg(_) | Label(_) as v => v
    _ => p.error("Jump target must be a register or code label")
  }
}

///|
fn read_rhs(p : @p.Parser) -> Rhs raise @base.Fault {
  if p.eat_keyword("ld") {
    Load(read_place(p))
  } else if p.eat_keyword("slice") {
    let s = read_value(p)
    let b = read_value(p)
    Slice(s, b, read_value(p))
  } else if p.eat_keyword("addr") {
    let s = read_value(p)
    Addr(s, read_value(p))
  } else if p.eat_keyword("len") {
    Len(read_value(p))
  } else {
    let a = read_value(p)
    if p.eat('+') {
      Binop(a, Add, read_value(p))
    } else if p.eat('-') {
      Binop(a, Sub, read_value(p))
    } else {
      Value(a)
    }
  }
}

///|
fn read_stmt(p : @p.Parser) -> Stmt raise @base.Fault {
  let s = if p.eat_keyword("print") {
    Print(read_value(p))
  } else if p.eat_keyword("input") {
    Input(read_place(p))
  } else if p.peek() == @p.Symbol('%') {
    let r = reg(p)
    p.expect_text(":=")
    Assign(r, read_rhs(p))
  } else {
    let dst = read_place(p)
    p.expect_text(":= \\st")
    Store(dst, read_value(p))
  }
  p.expect(';')
  s
}

///|
fn read_cond(p : @p.Parser) -> Cond raise @base.Fault {
  let a = read_value(p)
  let o = if p.eat_text("==") {
    Eq
  } else if p.eat('<') {
    Lt
  } else {
    p.expect('>')
    Gt
  }
  (a, o, read_value(p))
}

///|
fn parse(text : String) -> Code raise @base.Fault {
  let p = @p.create(text)
  p.expect_keyword("memories")
  let memories = p.integer()
  p.expect(';')
  @base.require(memories >= 1, "At least one memory is required")
  let blocks = []
  while !p.eof() {
    p.expect(':')
    let label = p.ident()
    let g = p.group('{')
    let stmts = []
    let ifs = []
    let ending : Ref[Ending?] = { val: None, }
    while !g.eof() {
      if g.eat_keyword("goto") {
        ending.val = Some(Goto(read_target(g)))
        g.expect(';')
        g.finish()
        break
      } else if g.eat_keyword("halt") {
        ending.val = Some(Halt)
        g.expect(';')
        g.finish()
        break
      } else if g.eat_keyword("if") {
        let cond = read_cond(g)
        g.expect_keyword("then")
        let target = read_target(g)
        g.expect(';')
        ifs.push({ cond, target, })
      } else {
        @base.require(ifs.is_empty(), "Statement after conditional jump")
        stmts.push(read_stmt(g))
      }
    }
    let ending = match ending.val {
      Some(e) => e
      None => g.error("Missing terminator")
    }
    blocks.push({ label, stmts, ifs, ending, })
  }
  { memories, blocks, }
}
