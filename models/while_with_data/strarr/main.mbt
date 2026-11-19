///|
priv enum Aexp {
  Const(Array[String])
  Var(String)
  Concat(Aexp, Aexp)
  Head(Aexp)
  Tail(Aexp)
}

///|
priv enum Bexp {
  IsEmpty(Aexp)
  Equal(Aexp, Aexp)
}

///|
priv enum Stmt {
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
  env : Array[(String, Array[String])]
}

///|
fn atom(p : @p.Parser) -> Aexp raise @base.Fault {
  if p.eat_keyword("head") {
    Head(atom(p))
  } else if p.eat_keyword("tail") {
    Tail(atom(p))
  } else if p.peek() == @p.Symbol('(') {
    let g = p.group('(')
    let a = aexp(g)
    g.finish()
    a
  } else if p.peek() == @p.Symbol('[') {
    Const(@data.read_chars(p, grouped_characters=false))
  } else {
    Var(p.ident())
  }
}

///|
fn aexp(p : @p.Parser) -> Aexp raise @base.Fault {
  let mut a = atom(p)
  while p.eat_text("++") {
    a = Concat(a, atom(p))
  }
  a
}

///|
fn bexp(p : @p.Parser) -> Bexp raise @base.Fault {
  if p.eat_keyword("is_empty") {
    IsEmpty(aexp(p))
  } else {
    let a = aexp(p)
    p.expect_text("==")
    Equal(a, aexp(p))
  }
}

///|
fn single(p : @p.Parser) -> Stmt raise @base.Fault {
  if p.eat_keyword("Nop") {
    Nop
  } else if p.eat_keyword("if") {
    let c = bexp(p)
    p.expect_keyword("then")
    let b = statement(p)
    p.expect_keyword("end")
    If(c, b)
  } else if p.eat_keyword("while") {
    let c = bexp(p)
    let g = p.group('{')
    let b = statement(g)
    g.finish()
    While(c, b)
  } else if p.peek() == @p.Symbol('(') {
    let g = p.group('(')
    let s = statement(g)
    g.finish()
    s
  } else {
    let n = p.ident()
    p.expect_text(":=")
    Assign(n, aexp(p))
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
fn eval(a : Aexp, env : Array[(String, Array[String])]) -> Array[String] {
  match a {
    Const(cs) => cs
    Var(n) => @data.env_get(env, n).unwrap_or([])
    Concat(a, b) => {
      let cs = eval(a, env).copy()
      cs.append(eval(b, env))
      cs
    }
    Head(a) => {
      let cs = eval(a, env)
      if cs.is_empty() {
        []
      } else {
        [cs[0]]
      }
    }
    Tail(a) => {
      let cs = eval(a, env)
      if cs.is_empty() {
        []
      } else {
        cs[1:].to_owned()
      }
    }
  }
}

///|
fn condition(b : Bexp, env : Array[(String, Array[String])]) -> Bool {
  match b {
    IsEmpty(a) => eval(a, env).is_empty()
    Equal(a, b) => eval(a, env) == eval(b, env)
  }
}

///|
fn small(
  s : Stmt,
  env : Array[(String, Array[String])],
) -> (Stmt, Array[(String, Array[String])]) {
  match s {
    Nop => (Nop, env)
    Assign(n, a) => (Nop, @data.env_set(env, n, eval(a, env)))
    Seq(Nop, b) => (b, env)
    Seq(a, b) => {
      let (a, env) = small(a, env)
      (Seq(a, b), env)
    }
    If(c, b) => (if condition(c, env) { b } else { Nop }, env)
    While(c, b) => (If(c, Seq(b, s)), env)
  }
}

///|
fn make(code : String, input : String) -> State raise @base.Fault {
  let p = @p.create(code, character_literals=true)
  let code = statement(p)
  p.finish()
  { code, stmt: code, env: @data.parse_env(input, @data.parse_chars), }
}

///|
fn step(
  s : State,
  input : String,
) -> @runtime.Transition[State] raise @base.Fault {
  @base.unit_input(input)
  match s.stmt {
    Nop => @runtime.Halt(@data.print_env(s.env, @data.chars_text))
    _ => {
      let (stmt, env) = small(s.stmt, s.env)
      @runtime.Continue({ ..s, stmt, env, }, "")
    }
  }
}

///|
fn aj(a : Aexp) -> Json {
  match a {
    Const(cs) => @base.tag("Const", [@base.jarray(cs.map(@base.jstr))])
    Var(n) => @base.tag("Var", [@base.jstr(n)])
    Concat(a, b) => @base.tag("Concat", [aj(a), aj(b)])
    Head(a) => @base.tag("Head", [aj(a)])
    Tail(a) => @base.tag("Tail", [aj(a)])
  }
}

///|
fn ad(j : Json) -> Aexp raise @base.Fault {
  fn a(i : Int32) -> Json raise @base.Fault {
    @base.tag_arg(j, i)
  }
  match @base.tag_name(j) {
    "Const" => Const(@base.array(a(0)).map(@data.decode_scalar))
    "Var" => Var(@base.identifier(@base.string(a(0))))
    "Concat" => Concat(ad(a(0)), ad(a(1)))
    "Head" => Head(ad(a(0)))
    "Tail" => Tail(ad(a(0)))
    _ => @base.error("Invalid array expression")
  }
}

///|
fn bj(b : Bexp) -> Json {
  match b {
    IsEmpty(a) => @base.tag("IsEmpty", [aj(a)])
    Equal(a, b) => @base.tag("Eq", [aj(a), aj(b)])
  }
}

///|
fn bd(j : Json) -> Bexp raise @base.Fault {
  match @base.tag_name(j) {
    "IsEmpty" => IsEmpty(ad(@base.tag_arg(j, 0)))
    "Eq" => Equal(ad(@base.tag_arg(j, 0)), ad(@base.tag_arg(j, 1)))
    _ => @base.error("Invalid array condition")
  }
}

///|
fn sj(s : Stmt) -> Json {
  match s {
    Nop => @base.tag("Nop", [])
    Assign(n, a) => @base.tag("Assign", [@base.jstr(n), aj(a)])
    Seq(a, b) => @base.tag("Seq", [sj(a), sj(b)])
    If(c, b) => @base.tag("If", [bj(c), sj(b)])
    While(c, b) => @base.tag("While", [bj(c), sj(b)])
  }
}

///|
fn sd(j : Json) -> Stmt raise @base.Fault {
  fn a(i : Int32) -> Json raise @base.Fault {
    @base.tag_arg(j, i)
  }
  match @base.tag_name(j) {
    "Nop" => Nop
    "Assign" => Assign(@base.identifier(@base.string(a(0))), ad(a(1)))
    "Seq" => Seq(sd(a(0)), sd(a(1)))
    "If" => If(bd(a(0)), sd(a(1)))
    "While" => While(bd(a(0)), sd(a(1)))
    _ => @base.error("Invalid array statement")
  }
}

///|
fn encode(s : State) -> Json {
  @base.object([
    ("code", sj(s.code)),
    ("stmt", sj(s.stmt)),
    ("env", @data.env_json(s.env, cs => @base.jarray(cs.map(@base.jstr)))),
  ])
}

///|
fn decode(j : Json) -> State raise @base.Fault {
  {
    code: sd(@base.field(j, "code")),
    stmt: sd(@base.field(j, "stmt")),
    env: @data.decode_env(@base.field(j, "env"), j => {
      @base.array(j).map(@data.decode_scalar)
    }),
  }
}

///|
fn at(a : Aexp) -> String {
  match a {
    Const(cs) => @data.chars_text(cs)
    Var(n) => n
    Concat(a, b) => "(" + at(a) + " ++ " + at(b) + ")"
    Head(a) => "\\head " + at(a)
    Tail(a) => "\\tail " + at(a)
  }
}

///|
fn bt(b : Bexp) -> String {
  match b {
    IsEmpty(a) => "\\is_empty " + at(a)
    Equal(a, b) => at(a) + " == " + at(b)
  }
}

///|
fn st(s : Stmt) -> String {
  match s {
    Nop => "\\Nop"
    Assign(n, a) => n + " := " + at(a)
    Seq(a, b) => st(a) + " ; " + st(b)
    If(c, b) => "\\if " + bt(c) + " \\then " + st(b) + " \\end"
    While(c, b) => "\\while " + bt(c) + " { " + st(b) + " }"
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
        s.env.map(pair => [pair.0, @data.chars_text(pair.1)]),
      ),
    ])
  })
}
