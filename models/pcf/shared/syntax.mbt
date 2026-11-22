///|
#derive(Eq)
pub(all) enum Binop {
  Add
  Sub
  And
}

///|
#derive(Eq)
pub(all) enum Unop {
  Inc
  Dec
  Not
}

///|
#derive(Eq)
pub(all) enum Expr {
  Nat(@utils.Nat)
  Bool(Bool)
  Unit
  Print(@utils.Nat)
  Var(String)
  BinOp(Expr, Binop, Expr)
  UnOp(Unop, Expr)
  Fun(String, Expr)
  Rec(String, String, Expr)
  App(Expr, Expr)
  If(Expr, Expr, Expr)
  Block(Stmt, Expr)
}

///|
#derive(Eq)
pub(all) enum Stmt {
  Skip
  PrintStmt(@utils.Nat)
  Seq(Stmt, Stmt)
  Let(String, Expr)
  IfStmt(Expr, Stmt)
  While(Expr, Stmt)
}

///|
pub(all) enum Value {
  VNat(@utils.Nat)
  VBool(Bool)
  VUnit
  VFun(String, Expr, Array[(String, Value)])
  VRec(String, String, Expr, Array[(String, Value)])
}

///|
pub type Env = Array[(String, Value)]

///|
pub fn binop_text(o : Binop) -> String {
  match o {
    Add => "+"
    Sub => "-"
    And => "&&"
  }
}

///|
pub fn unop_text(o : Unop) -> String {
  match o {
    Inc => "\\inc"
    Dec => "\\dec"
    Not => "\\not"
  }
}

///|
pub fn print(e : Expr) -> String {
  match e {
    Nat(n) => n.to_string()
    Bool(b) => if b { "\\true" } else { "\\false" }
    Unit => "\\unit"
    Print(n) => "\\print " + n.to_string()
    Var(n) => n
    BinOp(a, o, b) =>
      "(" + print(a) + " " + binop_text(o) + " " + print(b) + ")"
    UnOp(o, e) => "(" + unop_text(o) + " " + print(e) + ")"
    Fun(x, b) => "(\\fun " + x + " => " + print(b) + ")"
    Rec(f, x, b) => "(\\rec " + f + " " + x + " => " + print(b) + ")"
    App(f, x) => print(f) + "(" + print(x) + ")"
    If(c, t, e) =>
      "(\\if " +
      print(c) +
      " \\then " +
      print(t) +
      " \\else " +
      print(e) +
      " \\fi)"
    Block(s, e) => "{ " + print_stmt(s) + "; " + print(e) + " }"
  }
}

///|
pub fn print_stmt(s : Stmt) -> String {
  match s {
    Skip => "\\skip"
    PrintStmt(n) => "\\print " + n.to_string()
    Seq(a, b) => print_stmt(a) + "; " + print_stmt(b)
    Let(x, e) => "\\let " + x + " := " + print(e)
    IfStmt(c, s) => "\\if " + print(c) + " \\then " + print_stmt(s) + " \\end"
    While(c, s) => "\\while " + print(c) + " \\do " + print_stmt(s) + " \\end"
  }
}

///|
pub fn read(
  p : @utils.Parser,
  statements~ : Bool = false,
) -> Expr raise @utils.Fault {
  if p.eat_keyword("if") {
    return read_if(p, statements)
  }
  let mut e = application(p, statements)
  while p.peek() == @utils.Symbol('+') ||
        p.peek() == @utils.Symbol('-') ||
        p.peek() == @utils.Symbol('&') {
    let o = if p.eat('+') {
      Add
    } else if p.eat('-') {
      Sub
    } else {
      p.expect_text("&&")
      And
    }
    e = BinOp(e, o, application(p, statements))
  }
  e
}

///|
fn read_if(p : @utils.Parser, statements : Bool) -> Expr raise @utils.Fault {
  let c = read(p, statements~)
  p.expect_keyword("then")
  let t = read(p, statements~)
  p.expect_keyword("else")
  let e = read(p, statements~)
  p.expect_keyword("fi")
  If(c, t, e)
}

///|
fn application(p : @utils.Parser, statements : Bool) -> Expr raise @utils.Fault {
  let mut e = prefix(p, statements)
  while p.peek() == @utils.Symbol('(') {
    let g = p.group('(')
    let a = read(g, statements~)
    g.finish()
    e = App(e, a)
  }
  e
}

///|
fn prefix(p : @utils.Parser, statements : Bool) -> Expr raise @utils.Fault {
  if p.eat_keyword("inc") {
    UnOp(Inc, prefix(p, statements))
  } else if p.eat_keyword("dec") {
    UnOp(Dec, prefix(p, statements))
  } else if p.eat_keyword("not") {
    UnOp(Not, prefix(p, statements))
  } else {
    match p.peek() {
      @utils.Number(_) => Nat(p.natural())
      @utils.Keyword("true") => {
        ignore(p.bump())
        Bool(true)
      }
      @utils.Keyword("false") => {
        ignore(p.bump())
        Bool(false)
      }
      @utils.Keyword("unit") => {
        ignore(p.bump())
        Unit
      }
      @utils.Keyword("print") if !statements => {
        ignore(p.bump())
        Print(p.natural())
      }
      @utils.Keyword("fun") => {
        ignore(p.bump())
        let x = p.ident()
        p.expect_text("=>")
        Fun(x, read(p, statements~))
      }
      @utils.Keyword("rec") => {
        ignore(p.bump())
        let f = p.ident()
        let x = p.ident()
        p.expect_text("=>")
        Rec(f, x, read(p, statements~))
      }
      @utils.Keyword("if") => {
        ignore(p.bump())
        read_if(p, statements)
      }
      @utils.Symbol('(') => {
        let g = p.group('(')
        let e = read(g, statements~)
        g.finish()
        e
      }
      @utils.Symbol('{') if statements => {
        let g = p.group('{')
        let (s, e) = block_contents(g)
        g.finish()
        Block(s, e)
      }
      _ => Var(p.ident())
    }
  }
}

///|
fn simple_stmt(p : @utils.Parser) -> Stmt raise @utils.Fault {
  if p.eat_keyword("skip") {
    Skip
  } else if p.eat_keyword("print") {
    PrintStmt(p.natural())
  } else if p.eat_keyword("let") {
    let x = p.ident()
    p.expect_text(":=")
    Let(x, read(p, statements=true))
  } else if p.eat_keyword("if") {
    let c = read(p, statements=true)
    p.expect_keyword("then")
    let s = statement(p)
    p.expect_keyword("end")
    IfStmt(c, s)
  } else if p.eat_keyword("while") {
    let c = read(p, statements=true)
    p.expect_keyword("do")
    let s = statement(p)
    p.expect_keyword("end")
    While(c, s)
  } else {
    p.error("statement expected")
  }
}

///|
fn statement(p : @utils.Parser) -> Stmt raise @utils.Fault {
  let mut s = simple_stmt(p)
  while p.eat(';') {
    s = Seq(s, simple_stmt(p))
  }
  s
}

///|
fn block_contents(p : @utils.Parser) -> (Stmt, Expr) raise @utils.Fault {
  let s = simple_stmt(p)
  p.expect(';')
  let pos = p.position()
  try {
    let (t, e) = block_contents(p)
    (Seq(s, t), e)
  } catch {
    _ => {
      p.rewind(pos)
      (s, read(p, statements=true))
    }
  }
}

///|
pub fn validate_direct(e : Expr) -> Unit raise @utils.Fault {
  match e {
    Block(_, _) =>
      @utils.error("blocks belong to the expression/statement machine")
    BinOp(a, _, b) | App(a, b) => {
      validate_direct(a)
      validate_direct(b)
    }
    UnOp(_, e) | Fun(_, e) | Rec(_, _, e) => validate_direct(e)
    If(c, t, e) => {
      validate_direct(c)
      validate_direct(t)
      validate_direct(e)
    }
    _ => ()
  }
}

///|
fn view(e : Expr) -> @utils.View[Expr] {
  match e {
    Var(x) => @utils.Variable(x)
    Fun(x, b) => @utils.Node([x], [([0], b)], (ns, cs) => Fun(ns[0], cs[0]))
    Rec(f, x, b) =>
      @utils.Node([f, x], [([0, 1], b)], (ns, cs) => Rec(ns[0], ns[1], cs[0]))
    BinOp(a, o, b) =>
      @utils.Node([], [([], a), ([], b)], (_, cs) => BinOp(cs[0], o, cs[1]))
    UnOp(o, e) => @utils.Node([], [([], e)], (_, cs) => UnOp(o, cs[0]))
    App(a, b) =>
      @utils.Node([], [([], a), ([], b)], (_, cs) => App(cs[0], cs[1]))
    If(c, t, e) =>
      @utils.Node([], [([], c), ([], t), ([], e)], (_, cs) => {
        If(cs[0], cs[1], cs[2])
      })
    _ => @utils.Node([], [], (_, _) => e)
  }
}

///|
pub fn normalize(e : Expr) -> Expr {
  @utils.normalize({ view, variable: x => Var(x), }, e)
}

///|
pub fn substitute(e : Expr, x : String, value : Expr) -> Expr {
  @utils.substitute({ view, variable: x => Var(x), }, e, x, value)
}

///|
pub fn parse(
  text : String,
  statements~ : Bool = false,
) -> Expr raise @utils.Fault {
  let p = @utils.create(text)
  let e = read(p, statements~)
  p.finish()
  if statements {
    e
  } else {
    validate_direct(e)
    normalize(e)
  }
}

///|
pub fn inputs(text : String) -> Array[@utils.Nat] raise @utils.Fault {
  let p = @utils.create(text)
  let g = p.group('(')
  p.finish()
  let xs = []
  while !g.eof() {
    xs.push(g.natural())
    if g.eof() {
      break
    }
    g.expect(',')
  }
  xs
}

///|
pub fn apply_inputs(e : Expr, inputs : Array[@utils.Nat]) -> Expr {
  let mut e = e
  for n in inputs {
    e = App(e, Nat(n))
  }
  e
}

///|
pub fn is_value(e : Expr) -> Bool {
  match e {
    Nat(_) | Bool(_) | Unit | Fun(_, _) | Rec(_, _, _) => true
    _ => false
  }
}

///|
pub fn binop(o : Binop, a : Expr, b : Expr) -> Expr raise @utils.Fault {
  match (o, a, b) {
    (Add, Nat(a), Nat(b)) => Nat(a + b)
    (Sub, Nat(a), Nat(b)) => {
      @utils.require(a >= b, "operator '-' expects lhs >= rhs over naturals")
      Nat(a - b)
    }
    (And, Bool(a), Bool(b)) => Bool(a && b)
    (Add, _, _) => @utils.error("operator '+' expects naturals")
    (Sub, _, _) => @utils.error("operator '-' expects naturals")
    (And, _, _) => @utils.error("operator '&&' expects booleans")
  }
}

///|
pub fn unop(o : Unop, e : Expr) -> Expr raise @utils.Fault {
  match (o, e) {
    (Inc, Nat(n)) => Nat(n + @utils.one())
    (Dec, Nat(n)) => {
      @utils.require(!n.is_zero(), "operator 'dec' expects a positive natural")
      Nat(@utils.pred(n))
    }
    (Not, Bool(b)) => Bool(!b)
    (Inc, _) => @utils.error("operator 'inc' expects a natural")
    (Dec, _) => @utils.error("operator 'dec' expects a natural")
    (Not, _) => @utils.error("operator 'not' expects a boolean")
  }
}

///|
pub fn value_expr(v : Value) -> Expr {
  match v {
    VNat(n) => Nat(n)
    VBool(b) => Bool(b)
    VUnit => Unit
    VFun(x, b, _) => Fun(x, b)
    VRec(f, x, b, _) => Rec(f, x, b)
  }
}

///|
pub fn value_text(v : Value) -> String {
  print(value_expr(v))
}

///|
pub fn scalar_value(e : Expr) -> Value raise @utils.Fault {
  match e {
    Nat(n) => VNat(n)
    Bool(b) => VBool(b)
    Unit => VUnit
    _ => @utils.error("expected scalar value")
  }
}

///|
pub fn value_binop(o : Binop, a : Value, b : Value) -> Value raise @utils.Fault {
  scalar_value(binop(o, value_expr(a), value_expr(b)))
}

///|
pub fn value_unop(o : Unop, v : Value) -> Value raise @utils.Fault {
  scalar_value(unop(o, value_expr(v)))
}

///|
pub fn as_nat(v : Value) -> @utils.Nat raise @utils.Fault {
  match v {
    VNat(n) => n
    _ => @utils.error("final value is not a natural number")
  }
}

///|
pub fn boolean(context : String, v : Value) -> Bool raise @utils.Fault {
  match v {
    VBool(b) => b
    _ =>
      @utils.error(
        context + " expects a boolean condition, found " + value_text(v),
      )
  }
}

///|
pub fn lookup(env : Env, x : String) -> Value raise @utils.Fault {
  match @utils.env_get(env, x) {
    Some(v) => v
    None => @utils.error("unbound variable: " + x)
  }
}

///|
pub fn bind(env : Env, x : String, v : Value) -> Env {
  let e = [(x, v)]
  e.append(env)
  e
}

///|
pub fn apply(f : Value, arg : Value) -> (Expr, Env) raise @utils.Fault {
  match f {
    VFun(x, b, env) => (b, bind(env, x, arg))
    VRec(f, x, b, env) as r => (b, bind(bind(env, f, r), x, arg))
    _ => @utils.error("application expects a closure, found " + value_text(f))
  }
}

///|
pub fn render_env(env : Env) -> @definition.Block {
  @definition.text_table(
    "env",
    ["name", "value"],
    env.rev().map(p => [p.0, value_text(p.1)]),
  )
}
