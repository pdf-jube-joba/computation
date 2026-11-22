///|
priv enum Addr {
  VReg(Int32)
  Label(String)
  Imm(@utils.Nat)
}

///|
priv enum Value {
  VRegValue(Int32)
  ImmValue(@utils.Nat)
  LabelValue(String)
  Deref(Addr)
}

///|
priv enum Binop {
  Add
  Sub
}

///|
priv enum Cond {
  Lt(Value, Value)
  Eq(Value, Value)
  Gt(Value, Value)
}

///|
priv enum Stmt {
  Assign(Int32, Value)
  BinOp(Int32, Value, Binop, Value)
  Store(Addr, Value)
}

///|
priv struct JumpIf {
  cond : Cond
  target : Addr
}

///|
priv struct Cont {
  ifs : Array[JumpIf]
  jump : Addr
}

///|
priv struct Block {
  label : String
  stmts : Array[Stmt]
  cont : Cont
}

///|
priv struct StaticDef {
  label : String
  value : @utils.Nat
}

///|
struct Code {
  statics : Array[StaticDef]
  blocks : Array[Block]
}

///|
fn read_vreg(p : @utils.Parser) -> Int32 raise @utils.Fault {
  p.expect('%')
  let n = p.ident()
  @utils.require(n.has_prefix("v"), "Invalid vreg: %" + n)
  @utils.index(@utils.nat(n.unsafe_substring(start=1, end=n.length())))
}

///|
fn read_addr(p : @utils.Parser) -> Addr raise @utils.Fault {
  if p.peek() == @utils.Symbol('%') {
    VReg(read_vreg(p))
  } else if p.eat('@') {
    Label(p.ident())
  } else {
    if p.eat('#') {
      ignore(p.eat('+'))
    }
    Imm(p.natural())
  }
}

///|
fn read_place(p : @utils.Parser) -> Addr raise @utils.Fault {
  let g = p.group('[')
  let a = read_addr(g)
  g.finish()
  a
}

///|
fn read_value(p : @utils.Parser) -> Value raise @utils.Fault {
  if p.eat('*') {
    Deref(read_place(p))
  } else {
    match read_addr(p) {
      VReg(i) => VRegValue(i)
      Label(n) => LabelValue(n)
      Imm(n) => ImmValue(n)
    }
  }
}

///|
fn read_stmt(p : @utils.Parser) -> Stmt raise @utils.Fault {
  let s = if p.peek() == @utils.Symbol('%') {
    let d = read_vreg(p)
    p.expect_text(":=")
    let a = read_value(p)
    if p.eat('+') {
      BinOp(d, a, Add, read_value(p))
    } else if p.eat('-') {
      BinOp(d, a, Sub, read_value(p))
    } else {
      Assign(d, a)
    }
  } else {
    let d = read_place(p)
    p.expect_text(":=")
    Store(d, read_value(p))
  }
  p.expect(';')
  s
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
fn parse(text : String) -> Code raise @utils.Fault {
  let p = @utils.create(text)
  let statics = []
  let blocks = []
  while !p.eof() {
    p.expect('@')
    let label = p.ident()
    if p.peek() == @utils.Symbol('{') {
      let g = p.group('{')
      let stmts = []
      let ifs = []
      let jump : Ref[Addr?] = { val: None, }
      while !g.eof() {
        if g.eat_keyword("if") {
          let cond = read_cond(g)
          g.expect_keyword("then")
          let target = read_addr(g)
          g.expect(';')
          ifs.push({ cond, target, })
        } else if g.eat_keyword("goto") {
          @utils.require(
            jump.val is None,
            "Multiple final \\goto in block @" + label,
          )
          jump.val = Some(read_addr(g))
          g.expect(';')
        } else {
          @utils.require(
            jump.val is None,
            "Statement after \\goto in block @" + label,
          )
          stmts.push(read_stmt(g))
        }
      }
      let jump = match jump.val {
        Some(a) => a
        None => g.error("Block @" + label + " missing final \\goto")
      }
      blocks.push({ label, stmts, cont: { ifs, jump, }, })
    } else {
      ignore(p.eat('+'))
      statics.push({ label, value: p.natural(), })
    }
  }
  { statics, blocks, }
}

///|
fn addr_text(a : Addr) -> String {
  match a {
    VReg(i) => "%v" + i.to_string()
    Imm(n) => "#" + n.to_string()
    Label(n) => "@" + n
  }
}

///|
fn value_text(v : Value) -> String {
  match v {
    VRegValue(i) => "%v" + i.to_string()
    ImmValue(n) => "#" + n.to_string()
    LabelValue(n) => "@" + n
    Deref(p) => "*[" + addr_text(p) + "]"
  }
}

///|
fn stmt_text(s : Stmt) -> String {
  match s {
    Assign(dst, src) => "%v\{dst} := " + value_text(src) + ";"
    BinOp(dst, a, o, b) =>
      "%v\{dst} := " +
      value_text(a) +
      (match o {
        Add => " + "
        Sub => " - "
      }) +
      value_text(b) +
      ";"
    Store(p, src) => "[" + addr_text(p) + "] := " + value_text(src) + ";"
  }
}
