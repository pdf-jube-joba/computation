///|
pub(all) enum Aexp {
  Var(String)
  Num(@base.Nat)
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
priv struct State {
  code : Stmt
  stmt : Stmt
  env : Array[(String, @base.Nat)]
}

///|
fn atom(p : @p.Parser) -> Aexp raise @base.Fault {
  if p.peek() == @p.Symbol('(') {
    let g = p.group('(')
    let a = aexp(g)
    g.finish()
    a
  } else {
    match p.peek() {
      @p.Number(_) => Num(p.natural())
      _ => Var(p.ident())
    }
  }
}

///|
fn aexp(p : @p.Parser) -> Aexp raise @base.Fault {
  if p.eat_keyword("if") {
    let c = bexp(p)
    p.expect_keyword("then")
    let a = aexp(p)
    p.expect_keyword("else")
    Conditional(c, a, aexp(p))
  } else {
    let mut a = atom(p)
    while p.peek() == @p.Symbol('+') || p.peek() == @p.Symbol('-') {
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
fn boperand(p : @p.Parser) -> Bexp raise @base.Fault {
  if p.eat('!') {
    return Not(boperand(p))
  }
  if p.peek() == @p.Symbol('(') {
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
    @p.Symbol('<') => "<"
    @p.Symbol('=') => "="
    @p.Symbol('>') => ">"
    _ => p.error("Expected comparison")
  }
  Rel(a, op, aexp(p))
}

///|
fn bexp(p : @p.Parser) -> Bexp raise @base.Fault {
  let mut b = boperand(p)
  while p.eat_text("||") {
    b = Or(b, boperand(p))
  }
  b
}

///|
fn single(p : @p.Parser) -> Stmt raise @base.Fault {
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
  } else if p.peek() == @p.Symbol('(') {
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
fn statement(p : @p.Parser) -> Stmt raise @base.Fault {
  let mut s = single(p)
  while p.eat(';') {
    s = Seq(s, single(p))
  }
  s
}

///|
fn lookup(env : Array[(String, @base.Nat)], name : String) -> @base.Nat {
  for pair in env {
    if pair.0 == name {
      return pair.1
    }
  }
  @base.zero()
}

///|
fn set(
  env : Array[(String, @base.Nat)],
  name : String,
  value : @base.Nat,
) -> Array[(String, @base.Nat)] {
  let result = env.map(pair => if pair.0 == name { (name, value) } else { pair })
  if !env.any(pair => pair.0 == name) {
    result.push((name, value))
  }
  result
}

///|
fn eval_a(a : Aexp, env : Array[(String, @base.Nat)]) -> @base.Nat {
  match a {
    Var(v) => lookup(env, v)
    Num(n) => n
    Bin(a, add, b) =>
      if add {
        eval_a(a, env) + eval_a(b, env)
      } else {
        @base.sub(eval_a(a, env), eval_a(b, env))
      }
    Conditional(c, a, b) => eval_a(if eval_b(c, env) { a } else { b }, env)
  }
}

///|
fn eval_b(b : Bexp, env : Array[(String, @base.Nat)]) -> Bool {
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
  env : Array[(String, @base.Nat)],
) -> (Stmt, Array[(String, @base.Nat)]) {
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
fn make(code : String, input : String) -> State raise @base.Fault {
  let p = @p.create(code)
  let code = statement(p)
  p.finish()
  let mut env = []
  for line in input.split("\n") {
    if !line.trim().is_empty() {
      let parts = line.split("=").collect()
      @base.require(parts.length() == 2, "Invalid env line")
      env = set(
        env,
        @base.identifier(parts[0].trim().to_owned()),
        @base.nat(parts[1].to_owned()),
      )
    }
  }
  { code, stmt: code, env, }
}

///|
fn step(
  s : State,
  input : String,
) -> @runtime.Transition[State] raise @base.Fault {
  @base.unit_input(input)
  match s.stmt {
    Nop =>
      @runtime.Halt(
        @base.join(
          s.env.map(pair => pair.0 + " = " + pair.1.to_string() + "\n"),
          "",
        ),
      )
    _ => {
      let (stmt, env) = small_step(s.stmt, s.env)
      @runtime.Continue({ ..s, stmt, env, }, "")
    }
  }
}

///|
fn aj(a : Aexp) -> Json {
  match a {
    Var(v) => @base.tag("Var", [@base.jstr(v)])
    Num(n) => @base.tag("Num", [@base.jstr(n.to_string())])
    Bin(a, add, b) =>
      @base.tag("BinOp", [
        aj(a),
        @base.tag(if add { "Add" } else { "Sub" }, []),
        aj(b),
      ])
    Conditional(c, a, b) => @base.tag("IfThenElse", [bj(c), aj(a), aj(b)])
  }
}

///|
fn bj(b : Bexp) -> Json {
  match b {
    Rel(a, op, b) =>
      @base.tag("Rel", [
        aj(a),
        @base.tag(
          match op {
            "<" => "Lt"
            ">" => "Gt"
            _ => "Eq"
          },
          [],
        ),
        aj(b),
      ])
    Or(a, b) => @base.tag("Or", [bj(a), bj(b)])
    Not(b) => @base.tag("Not", [bj(b)])
  }
}

///|
fn sj(s : Stmt) -> Json {
  match s {
    Nop => @base.tag("Nop", [])
    Assign(v, a) => @base.tag("Assign", [@base.jstr(v), aj(a)])
    Seq(a, b) => @base.tag("Seq", [sj(a), sj(b)])
    If(c, b) => @base.tag("If", [bj(c), sj(b)])
    While(c, b) => @base.tag("While", [bj(c), sj(b)])
  }
}

///|
fn ad(j : Json) -> Aexp raise @base.Fault {
  fn arg(i : Int32) -> Json raise @base.Fault {
    @base.tag_arg(j, i)
  }
  match @base.tag_name(j) {
    "Var" => Var(@base.identifier(@base.string(arg(0))))
    "Num" => Num(@base.json_nat(arg(0)))
    "BinOp" => {
      let op = @base.tag_name(arg(1))
      @base.require(op == "Add" || op == "Sub", "Invalid arithmetic operator")
      Bin(ad(arg(0)), op == "Add", ad(arg(2)))
    }
    "IfThenElse" => Conditional(bd(arg(0)), ad(arg(1)), ad(arg(2)))
    _ => @base.error("Invalid expression")
  }
}

///|
fn bd(j : Json) -> Bexp raise @base.Fault {
  fn arg(i : Int32) -> Json raise @base.Fault {
    @base.tag_arg(j, i)
  }
  match @base.tag_name(j) {
    "Rel" => {
      let op = match @base.tag_name(arg(1)) {
        "Lt" => "<"
        "Gt" => ">"
        "Eq" => "="
        _ => @base.error("Invalid relation")
      }
      Rel(ad(arg(0)), op, ad(arg(2)))
    }
    "Or" => Or(bd(arg(0)), bd(arg(1)))
    "Not" => Not(bd(arg(0)))
    _ => @base.error("Invalid condition")
  }
}

///|
fn sd(j : Json) -> Stmt raise @base.Fault {
  fn arg(i : Int32) -> Json raise @base.Fault {
    @base.tag_arg(j, i)
  }
  match @base.tag_name(j) {
    "Nop" => Nop
    "Assign" => Assign(@base.identifier(@base.string(arg(0))), ad(arg(1)))
    "Seq" => Seq(sd(arg(0)), sd(arg(1)))
    "If" => If(bd(arg(0)), sd(arg(1)))
    "While" => While(bd(arg(0)), sd(arg(1)))
    _ => @base.error("Invalid statement")
  }
}

///|
fn encode(s : State) -> Json {
  @base.object([
    ("code", sj(s.code)),
    ("stmt", sj(s.stmt)),
    (
      "env",
      @base.jarray(
        s.env.map(pair => {
          @base.jarray([@base.jstr(pair.0), @base.jstr(pair.1.to_string())])
        }),
      ),
    ),
  ])
}

///|
fn decode(j : Json) -> State raise @base.Fault {
  {
    code: sd(@base.field(j, "code")),
    stmt: sd(@base.field(j, "stmt")),
    env: @base.array(@base.field(j, "env")).map(pair => {
      let a = @base.array(pair)
      @base.require(a.length() == 2, "Invalid env entry")
      (@base.identifier(@base.string(a[0])), @base.json_nat(a[1]))
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
pub fn model() -> @runtime.Model {
  @runtime.model(make, step, encode, decode, s => {
    @base.jarray([
      @base.titled("stmt", st(s.stmt)),
      @base.table(
        "env",
        ["var", "value"],
        s.env.map(pair => [pair.0, pair.1.to_string()]),
      ),
    ])
  })
}

///|
test "arbitrary precision, boolean grouping, saturation and conditional expressions" {
  let m = model()
  m.make(
    "x := 99999999999999999999; \\while !(x = 0) [x := x - 99999999999999999999]; y := \\if (x + 1) > 0 || x > 2 \\then 7 \\else 9",
    "",
  )
  let mut result = @base.parse_json(m.step(""))
  for _ in 0..<30 {
    if @base.string(@base.field(result, "kind")) == "halt" {
      break
    }
    result = @base.parse_json(m.step(""))
  }
  assert_eq(@base.string(@base.field(result, "foutput")), "x = 0\ny = 7\n")
}

///|
pub fn parse_code(text : String) -> Stmt raise @base.Fault {
  let p = @p.create(text)
  let s = statement(p)
  p.finish()
  s
}
