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
enum Stmt {
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
  env : Array[(String, Array[String])]
}

///|
fn atom(p : @utils.Parser) -> Aexp raise @utils.Fault {
  if p.eat_keyword("head") {
    Head(atom(p))
  } else if p.eat_keyword("tail") {
    Tail(atom(p))
  } else if p.peek() == @utils.Symbol('(') {
    let g = p.group('(')
    let a = aexp(g)
    g.finish()
    a
  } else if p.peek() == @utils.Symbol('[') {
    Const(@utils.read_chars(p, grouped_characters=false))
  } else {
    Var(p.ident())
  }
}

///|
fn aexp(p : @utils.Parser) -> Aexp raise @utils.Fault {
  let mut a = atom(p)
  while p.eat_text("++") {
    a = Concat(a, atom(p))
  }
  a
}

///|
fn bexp(p : @utils.Parser) -> Bexp raise @utils.Fault {
  if p.eat_keyword("is_empty") {
    IsEmpty(aexp(p))
  } else {
    let a = aexp(p)
    p.expect_text("==")
    Equal(a, aexp(p))
  }
}

///|
fn single(p : @utils.Parser) -> Stmt raise @utils.Fault {
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
  } else if p.peek() == @utils.Symbol('(') {
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
fn statement(p : @utils.Parser) -> Stmt raise @utils.Fault {
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
    Var(n) => @utils.env_get(env, n).unwrap_or([])
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
    Assign(n, a) => (Nop, @utils.env_set(env, n, eval(a, env)))
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
      let (stmt, env) = small(s.stmt, s.env)
      @definition.Continue({ ..s, stmt, env, }, ())
    }
  }
}

///|
fn aj(a : Aexp) -> Json {
  match a {
    Const(cs) => @utils.tag("Const", [@utils.jarray(cs.map(@utils.jstr))])
    Var(n) => @utils.tag("Var", [@utils.jstr(n)])
    Concat(a, b) => @utils.tag("Concat", [aj(a), aj(b)])
    Head(a) => @utils.tag("Head", [aj(a)])
    Tail(a) => @utils.tag("Tail", [aj(a)])
  }
}

///|
fn ad(j : Json) -> Aexp raise @utils.Fault {
  fn a(i : Int32) -> Json raise @utils.Fault {
    @utils.tag_arg(j, i)
  }
  match @utils.tag_name(j) {
    "Const" => Const(@utils.array(a(0)).map(@utils.decode_scalar))
    "Var" => Var(@utils.identifier(@utils.string(a(0))))
    "Concat" => Concat(ad(a(0)), ad(a(1)))
    "Head" => Head(ad(a(0)))
    "Tail" => Tail(ad(a(0)))
    _ => @utils.error("Invalid array expression")
  }
}

///|
fn bj(b : Bexp) -> Json {
  match b {
    IsEmpty(a) => @utils.tag("IsEmpty", [aj(a)])
    Equal(a, b) => @utils.tag("Eq", [aj(a), aj(b)])
  }
}

///|
fn bd(j : Json) -> Bexp raise @utils.Fault {
  match @utils.tag_name(j) {
    "IsEmpty" => IsEmpty(ad(@utils.tag_arg(j, 0)))
    "Eq" => Equal(ad(@utils.tag_arg(j, 0)), ad(@utils.tag_arg(j, 1)))
    _ => @utils.error("Invalid array condition")
  }
}

///|
fn sj(s : Stmt) -> Json {
  match s {
    Nop => @utils.tag("Nop", [])
    Assign(n, a) => @utils.tag("Assign", [@utils.jstr(n), aj(a)])
    Seq(a, b) => @utils.tag("Seq", [sj(a), sj(b)])
    If(c, b) => @utils.tag("If", [bj(c), sj(b)])
    While(c, b) => @utils.tag("While", [bj(c), sj(b)])
  }
}

///|
fn sd(j : Json) -> Stmt raise @utils.Fault {
  fn a(i : Int32) -> Json raise @utils.Fault {
    @utils.tag_arg(j, i)
  }
  match @utils.tag_name(j) {
    "Nop" => Nop
    "Assign" => Assign(@utils.identifier(@utils.string(a(0))), ad(a(1)))
    "Seq" => Seq(sd(a(0)), sd(a(1)))
    "If" => If(bd(a(0)), sd(a(1)))
    "While" => While(bd(a(0)), sd(a(1)))
    _ => @utils.error("Invalid array statement")
  }
}

///|
fn encode(s : State) -> Json {
  @utils.object([
    ("code", sj(s.code)),
    ("stmt", sj(s.stmt)),
    ("env", @utils.env_json(s.env, cs => @utils.jarray(cs.map(@utils.jstr)))),
  ])
}

///|
fn decode(j : Json) -> State raise @utils.Fault {
  {
    code: sd(@utils.field(j, "code")),
    stmt: sd(@utils.field(j, "stmt")),
    env: @utils.decode_env(@utils.field(j, "env"), j => {
      @utils.array(j).map(@utils.decode_scalar)
    }),
  }
}

///|
fn at(a : Aexp) -> String {
  match a {
    Const(cs) => @utils.chars_text(cs)
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
