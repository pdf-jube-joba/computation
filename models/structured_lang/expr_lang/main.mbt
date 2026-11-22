///|
pub(all) enum Aexp {
  Var(String)
  Num(@utils.Nat)
  Bin(Aexp, Bool, Aexp)
  Conditional(Bexp, Aexp, Aexp)
}

///|
pub(all) enum Bexp {
  Rel(Aexp, String, Aexp)
  Or(Bexp, Bexp)
  Not(Bexp)
}

///|
pub(all) enum Stmt {
  Nop
  Assign(String, Aexp)
  Seq(Stmt, Stmt)
  If(Bexp, Stmt)
  While(Bexp, Stmt)
}

///|
struct State {
  code : Stmt
  stmt : Stmt
  env : Array[(String, @utils.Nat)]
}

///|
fn atom(p : @utils.Parser) -> Aexp raise @utils.Fault {
  if p.peek() == @utils.Symbol('(') {
    let g = p.group('(')
    let a = aexp(g)
    g.finish()
    a
  } else {
    match p.peek() {
      @utils.Number(_) => Num(p.natural())
      _ => Var(p.ident())
    }
  }
}

///|
fn aexp(p : @utils.Parser) -> Aexp raise @utils.Fault {
  if p.eat_keyword("if") {
    let c = bexp(p)
    p.expect_keyword("then")
    let a = aexp(p)
    p.expect_keyword("else")
    Conditional(c, a, aexp(p))
  } else {
    let mut a = atom(p)
    while p.peek() == @utils.Symbol('+') || p.peek() == @utils.Symbol('-') {
      let add = p.eat('+')
      if !add {
        p.expect('-')
      }
      a = Bin(a, add, atom(p))
    }
    a
  }
}

///|
fn boperand(p : @utils.Parser) -> Bexp raise @utils.Fault {
  if p.eat('!') {
    return Not(boperand(p))
  }
  if p.peek() == @utils.Symbol('(') {
    let saved = p.position()
    try {
      let g = p.group('(')
      let c = bexp(g)
      g.finish()
      return c
    } catch {
      _ => p.rewind(saved)
    }
  }
  let a = aexp(p)
  let op = match p.bump() {
    @utils.Symbol('<') => "<"
    @utils.Symbol('=') => "="
    @utils.Symbol('>') => ">"
    _ => p.error("Expected comparison")
  }
  Rel(a, op, aexp(p))
}

///|
fn bexp(p : @utils.Parser) -> Bexp raise @utils.Fault {
  let mut b = boperand(p)
  while p.eat_text("||") {
    b = Or(b, boperand(p))
  }
  b
}

///|
fn single(p : @utils.Parser) -> Stmt raise @utils.Fault {
  if p.eat_keyword("Nop") {
    Nop
  } else if p.eat_keyword("if") {
    let c = bexp(p)
    If(c, single(p))
  } else if p.eat_keyword("while") {
    let c = bexp(p)
    let g = p.group('[')
    let b = statement(g)
    g.finish()
    While(c, b)
  } else if p.peek() == @utils.Symbol('(') {
    let g = p.group('(')
    let b = statement(g)
    g.finish()
    b
  } else {
    let name = p.ident()
    p.expect_text(":=")
    Assign(name, aexp(p))
  }
}

///|
fn statement(p : @utils.Parser) -> Stmt raise @utils.Fault {
  let mut s = single(p)
  while p.eat(';') {
    s = Seq(s, single(p))
  }
  s
}

///|
fn lookup(env : Array[(String, @utils.Nat)], name : String) -> @utils.Nat {
  for pair in env {
    if pair.0 == name {
      return pair.1
    }
  }
  @utils.zero()
}

///|
fn set(
  env : Array[(String, @utils.Nat)],
  name : String,
  value : @utils.Nat,
) -> Array[(String, @utils.Nat)] {
  let result = env.map(pair => if pair.0 == name { (name, value) } else { pair })
  if !env.any(pair => pair.0 == name) {
    result.push((name, value))
  }
  result
}

///|
fn eval_a(a : Aexp, env : Array[(String, @utils.Nat)]) -> @utils.Nat {
  match a {
    Var(v) => lookup(env, v)
    Num(n) => n
    Bin(a, add, b) =>
      if add {
        eval_a(a, env) + eval_a(b, env)
      } else {
        @utils.sub(eval_a(a, env), eval_a(b, env))
      }
    Conditional(c, a, b) => eval_a(if eval_b(c, env) { a } else { b }, env)
  }
}

///|
fn eval_b(b : Bexp, env : Array[(String, @utils.Nat)]) -> Bool {
  match b {
    Rel(a, op, b) => {
      let a = eval_a(a, env)
      let b = eval_a(b, env)
      match op {
        "<" => a < b
        ">" => a > b
        _ => a == b
      }
    }
    Or(a, b) => eval_b(a, env) || eval_b(b, env)
    Not(b) => !eval_b(b, env)
  }
}

///|
fn small_step(
  s : Stmt,
  env : Array[(String, @utils.Nat)],
) -> (Stmt, Array[(String, @utils.Nat)]) {
  match s {
    Nop => (Nop, env)
    Assign(v, a) => (Nop, set(env, v, eval_a(a, env)))
    Seq(Nop, b) => (b, env)
    Seq(a, b) => {
      let (a, env) = small_step(a, env)
      (Seq(a, b), env)
    }
    If(c, b) => (if eval_b(c, env) { b } else { Nop }, env)
    While(c, b) => (If(c, Seq(b, s)), env)
  }
}

///|
fn make(code : Stmt, input : Environment) -> State {
  { code, stmt: code, env: input.values, }
}

///|
fn step(
  s : State,
  _input : Rinput,
) -> @definition.Transition[State, Routput, Foutput] {
  match s.stmt {
    Nop => @definition.Halt(Environment::{ values: s.env, })
    _ => {
      let (stmt, env) = small_step(s.stmt, s.env)
      @definition.Continue({ ..s, stmt, env, }, ())
    }
  }
}

///|
fn aj(a : Aexp) -> Json {
  match a {
    Var(v) => @utils.tag("Var", [@utils.jstr(v)])
    Num(n) => @utils.tag("Num", [@utils.jstr(n.to_string())])
    Bin(a, add, b) =>
      @utils.tag("BinOp", [
        aj(a),
        @utils.tag(if add { "Add" } else { "Sub" }, []),
        aj(b),
      ])
    Conditional(c, a, b) => @utils.tag("IfThenElse", [bj(c), aj(a), aj(b)])
  }
}

///|
fn bj(b : Bexp) -> Json {
  match b {
    Rel(a, op, b) =>
      @utils.tag("Rel", [
        aj(a),
        @utils.tag(
          match op {
            "<" => "Lt"
            ">" => "Gt"
            _ => "Eq"
          },
          [],
        ),
        aj(b),
      ])
    Or(a, b) => @utils.tag("Or", [bj(a), bj(b)])
    Not(b) => @utils.tag("Not", [bj(b)])
  }
}

///|
fn sj(s : Stmt) -> Json {
  match s {
    Nop => @utils.tag("Nop", [])
    Assign(v, a) => @utils.tag("Assign", [@utils.jstr(v), aj(a)])
    Seq(a, b) => @utils.tag("Seq", [sj(a), sj(b)])
    If(c, b) => @utils.tag("If", [bj(c), sj(b)])
    While(c, b) => @utils.tag("While", [bj(c), sj(b)])
  }
}

///|
fn ad(j : Json) -> Aexp raise @utils.Fault {
  fn arg(i : Int32) -> Json raise @utils.Fault {
    @utils.tag_arg(j, i)
  }
  match @utils.tag_name(j) {
    "Var" => Var(@utils.identifier(@utils.string(arg(0))))
    "Num" => Num(@utils.json_nat(arg(0)))
    "BinOp" => {
      let op = @utils.tag_name(arg(1))
      @utils.require(op == "Add" || op == "Sub", "Invalid arithmetic operator")
      Bin(ad(arg(0)), op == "Add", ad(arg(2)))
    }
    "IfThenElse" => Conditional(bd(arg(0)), ad(arg(1)), ad(arg(2)))
    _ => @utils.error("Invalid expression")
  }
}

///|
fn bd(j : Json) -> Bexp raise @utils.Fault {
  fn arg(i : Int32) -> Json raise @utils.Fault {
    @utils.tag_arg(j, i)
  }
  match @utils.tag_name(j) {
    "Rel" => {
      let op = match @utils.tag_name(arg(1)) {
        "Lt" => "<"
        "Gt" => ">"
        "Eq" => "="
        _ => @utils.error("Invalid relation")
      }
      Rel(ad(arg(0)), op, ad(arg(2)))
    }
    "Or" => Or(bd(arg(0)), bd(arg(1)))
    "Not" => Not(bd(arg(0)))
    _ => @utils.error("Invalid condition")
  }
}

///|
fn sd(j : Json) -> Stmt raise @utils.Fault {
  fn arg(i : Int32) -> Json raise @utils.Fault {
    @utils.tag_arg(j, i)
  }
  match @utils.tag_name(j) {
    "Nop" => Nop
    "Assign" => Assign(@utils.identifier(@utils.string(arg(0))), ad(arg(1)))
    "Seq" => Seq(sd(arg(0)), sd(arg(1)))
    "If" => If(bd(arg(0)), sd(arg(1)))
    "While" => While(bd(arg(0)), sd(arg(1)))
    _ => @utils.error("Invalid statement")
  }
}

///|
fn encode(s : State) -> Json {
  @utils.object([
    ("code", sj(s.code)),
    ("stmt", sj(s.stmt)),
    (
      "env",
      @utils.jarray(
        s.env.map(pair => {
          @utils.jarray([@utils.jstr(pair.0), @utils.jstr(pair.1.to_string())])
        }),
      ),
    ),
  ])
}

///|
fn decode(j : Json) -> State raise @utils.Fault {
  {
    code: sd(@utils.field(j, "code")),
    stmt: sd(@utils.field(j, "stmt")),
    env: @utils.array(@utils.field(j, "env")).map(pair => {
      let a = @utils.array(pair)
      @utils.require(a.length() == 2, "Invalid env entry")
      (@utils.identifier(@utils.string(a[0])), @utils.json_nat(a[1]))
    }),
  }
}

///|
fn at(a : Aexp) -> String {
  match a {
    Var(v) => v
    Num(n) => n.to_string()
    Bin(a, add, b) =>
      "(" + at(a) + (if add { " + " } else { " - " }) + at(b) + ")"
    Conditional(c, a, b) =>
      "\\if " + bt(c) + " \\then " + at(a) + " \\else " + at(b)
  }
}

///|
fn bt(b : Bexp) -> String {
  match b {
    Rel(a, op, b) => "(" + at(a) + " " + op + " " + at(b) + ")"
    Or(a, b) => "(" + bt(a) + " || " + bt(b) + ")"
    Not(b) => "!" + bt(b)
  }
}

///|
fn st(s : Stmt) -> String {
  match s {
    Nop => "\\Nop"
    Assign(v, a) => v + " := " + at(a)
    Seq(a, b) => st(a) + " ; " + st(b)
    If(c, b) => "\\if " + bt(c) + " " + st(b)
    While(c, b) => "\\while " + bt(c) + " [ " + st(b) + " ]"
  }
}

///|
pub fn model() -> @definition.Model[
  Code,
  Ainput,
  Rinput,
  Routput,
  Foutput,
  State,
] {
  @definition.Model(make=(code, input) => make(code, input), step=(state, input) => {
    step(state, input)
  })
}

///|
pub fn parse_code(text : String) -> Stmt raise @utils.Fault {
  let p = @utils.create(text)
  let s = statement(p)
  p.finish()
  s
}
