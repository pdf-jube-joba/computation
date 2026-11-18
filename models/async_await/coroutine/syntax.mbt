///|
priv enum Atom {
  Var(String)
  Imm(@base.Nat)
}

///|
priv enum Binop {
  Add
  Sub
  Mul
}

///|
priv enum Aexp {
  Atom(Atom)
  Bin(Aexp, Binop, Aexp)
}

///|
priv enum Relop {
  Eq
  Ne
  Lt
}

///|
priv enum Bexp {
  Not(Bexp)
  Rel(Aexp, Relop, Aexp)
  And(Bexp, Bexp)
  Or(Bexp, Bexp)
  Done(String)
}

///|
priv enum Stmt {
  Assign(String, Aexp)
  IfGoto(Bexp, Int32)
  Goto(Int32)
  Call(String)
  Run(String, String)
  Yield
}

///|
priv struct FunctionDef {
  name : String
  body : Array[Stmt]
}

///|
fn a_atom(p : @p.Parser) -> Aexp raise @base.Fault {
  match p.peek() {
    @p.Number(_) => Atom(Imm(p.natural()))
    @p.Ident(_) => Atom(Var(p.ident()))
    @p.Symbol('(') => {
      let g = p.group('(')
      let a = aexp(g)
      g.finish()
      a
    }
    _ => p.error("Unexpected token in arithmetic expression")
  }
}

///|
fn aexp(p : @p.Parser, min_prec~ : Int32 = 0) -> Aexp raise @base.Fault {
  let mut a = a_atom(p)
  while true {
    let op = match p.peek() {
      @p.Symbol('+') => Some((Add, 1))
      @p.Symbol('-') => Some((Sub, 1))
      @p.Symbol('*') => Some((Mul, 3))
      _ => None
    }
    match op {
      Some((o, prec)) if prec >= min_prec => {
        ignore(p.bump())
        a = Bin(a, o, aexp(p, min_prec=prec + 1))
      }
      _ => break
    }
  }
  a
}

///|
fn b_atom(p : @p.Parser) -> Bexp raise @base.Fault {
  if p.eat('!') {
    return Not(b_atom(p))
  }
  if p.eat_keyword("done") {
    p.expect('$')
    return Done(p.ident())
  }
  if p.peek() == @p.Symbol('(') {
    let pos = p.position()
    try {
      let g = p.group('(')
      let b = bexp(g)
      g.finish()
      return b
    } catch {
      _ => p.rewind(pos)
    }
  }
  let a = aexp(p)
  let o = if p.eat_text("==") {
    Eq
  } else if p.eat_text("!=") {
    Ne
  } else {
    p.expect('<')
    Lt
  }
  Rel(a, o, aexp(p))
}

///|
fn b_and(p : @p.Parser) -> Bexp raise @base.Fault {
  let mut b = b_atom(p)
  while p.eat_text("&&") {
    b = And(b, b_atom(p))
  }
  b
}

///|
fn bexp(p : @p.Parser) -> Bexp raise @base.Fault {
  let mut b = b_and(p)
  while p.eat_text("||") {
    b = Or(b, b_and(p))
  }
  b
}

///|
fn offset(p : @p.Parser) -> Int32 raise @base.Fault {
  let neg = p.eat('-')
  if !neg {
    ignore(p.eat('+'))
  }
  let n = p.integer()
  if neg {
    -n
  } else {
    n
  }
}

///|
fn read_stmt(p : @p.Parser) -> Stmt raise @base.Fault {
  if p.eat_keyword("if") {
    let b = bexp(p)
    p.expect_keyword("goto")
    IfGoto(b, offset(p))
  } else if p.eat_keyword("goto") {
    Goto(offset(p))
  } else if p.eat_keyword("call") {
    Call(p.ident())
  } else if p.eat_keyword("run") {
    let n = p.ident()
    p.expect_text("->")
    p.expect('$')
    Run(n, p.ident())
  } else if p.eat_keyword("yield") {
    Yield
  } else {
    let n = p.ident()
    p.expect_text("<-")
    Assign(n, aexp(p))
  }
}

///|
fn parse(text : String) -> Array[FunctionDef] raise @base.Fault {
  let p = @p.create(text)
  let code = []
  while !p.eof() {
    p.expect_keyword("fn")
    let name = p.ident()
    let g = p.group('{')
    let body = []
    while !g.eof() {
      if g.eat(';') {
        continue
      }
      body.push(read_stmt(g))
      if !g.eof() {
        g.expect(';')
      }
    }
    code.push({ name, body, })
  }
  @base.require(!code.is_empty(), "At least one function is required")
  code
}

///|
fn validate(code : Array[FunctionDef]) -> Unit raise @base.Fault {
  @data.unique(code.map(f => f.name), "duplicate function: ")
  @base.require(code.any(f => f.name == "main"), "missing main function")
}

///|
fn atext(a : Aexp) -> String {
  match a {
    Atom(Var(n)) => n
    Atom(Imm(n)) => n.to_string()
    Bin(a, o, b) =>
      "(" +
      atext(a) +
      " " +
      (match o {
        Add => "+"
        Sub => "-"
        Mul => "*"
      }) +
      " " +
      atext(b) +
      ")"
  }
}

///|
fn btext(b : Bexp) -> String {
  match b {
    Not(b) => "!" + btext(b)
    Rel(a, o, b) =>
      atext(a) +
      " " +
      (match o {
        Eq => "=="
        Ne => "!="
        Lt => "<"
      }) +
      " " +
      atext(b)
    And(a, b) => "(" + btext(a) + " && " + btext(b) + ")"
    Or(a, b) => "(" + btext(a) + " || " + btext(b) + ")"
    Done(n) => "\\done $" + n
  }
}

///|
fn stext(s : Stmt) -> String {
  fn fmt(n : Int32) -> String {
    if n >= 0 {
      "+" + n.to_string()
    } else {
      n.to_string()
    }
  }
  match s {
    Assign(n, a) => n + " <- " + atext(a)
    IfGoto(b, n) => "\\if " + btext(b) + " \\goto " + fmt(n)
    Goto(n) => "\\goto " + fmt(n)
    Call(n) => "\\call " + n
    Run(n, id) => "\\run " + n + " -> $" + id
    Yield => "\\yield"
  }
}
