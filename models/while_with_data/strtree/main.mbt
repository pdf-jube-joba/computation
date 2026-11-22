///|
priv enum Aexp {
  Mtoa(Texp)
  AtomChar(String)
}

///|
priv enum Texp {
  Var(String)
  Atom(Aexp)
  Cons(Texp, Texp)
  Left(Texp)
  Right(Texp)
}

///|
priv enum Bexp {
  IsAtom(Texp)
  Equal(Texp, Texp)
}

///|
enum Stmt {
  Nop
  Assign(String, Texp)
  Seq(Stmt, Stmt)
  IfEq(Bexp, Stmt)
  While(Bexp, Stmt)
}

///|
struct State {
  code : Stmt
  stmt : Stmt
  env : Array[(String, @utils.CharTree)]
}

///|
fn aexp(p : @utils.Parser) -> Aexp raise @utils.Fault {
  if p.eat_keyword("mtoa") {
    Mtoa(texp(p))
  } else if p.peek() == @utils.Symbol('(') {
    let g = p.group('(')
    let a = aexp(g)
    g.finish()
    a
  } else {
    AtomChar(p.character())
  }
}

///|
fn texp(p : @utils.Parser) -> Texp raise @utils.Fault {
  if p.eat_keyword("atom") {
    Atom(aexp(p))
  } else if p.eat_keyword("cons") {
    let a = texp(p)
    Cons(a, texp(p))
  } else if p.eat_keyword("left") {
    Left(texp(p))
  } else if p.eat_keyword("right") {
    Right(texp(p))
  } else if p.peek() == @utils.Symbol('(') {
    let g = p.group('(')
    let a = texp(g)
    g.finish()
    a
  } else {
    Var(p.ident())
  }
}

///|
fn relation(p : @utils.Parser) -> Bexp raise @utils.Fault {
  let a = texp(p)
  p.expect_text("==")
  Equal(a, texp(p))
}

///|
fn bexp(p : @utils.Parser) -> Bexp raise @utils.Fault {
  if p.eat_keyword("is_atom") {
    return IsAtom(texp(p))
  }
  let saved = p.position()
  relation(p) catch {
    _ => {
      p.rewind(saved)
      let g = p.group('(')
      let c = relation(g)
      g.finish()
      c
    }
  }
}

///|
fn single(p : @utils.Parser) -> Stmt raise @utils.Fault {
  if p.eat_keyword("Nop") {
    Nop
  } else if p.eat_keyword("ifeq") {
    let c = bexp(p)
    p.expect_keyword("then")
    let b = statement(p)
    p.expect_keyword("end")
    IfEq(c, b)
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
    Assign(n, texp(p))
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
fn eval_a(
  a : Aexp,
  env : Array[(String, @utils.CharTree)],
) -> String raise @utils.Fault {
  match a {
    AtomChar(c) => c
    Mtoa(t) =>
      match eval_t(t, env) {
        @utils.AtomValue(c) => c
        _ => @utils.error("mtoa expects an atom")
      }
  }
}

///|
fn eval_t(
  t : Texp,
  env : Array[(String, @utils.CharTree)],
) -> @utils.CharTree raise @utils.Fault {
  match t {
    Var(n) =>
      match @utils.env_get(env, n) {
        Some(v) => v
        None => @utils.error("uninitialized variable: " + n)
      }
    Atom(a) => @utils.AtomValue(eval_a(a, env))
    Cons(a, b) => {
      let a = eval_t(a, env)
      @utils.ConsValue(a, eval_t(b, env))
    }
    Left(t) =>
      match eval_t(t, env) {
        @utils.ConsValue(a, _) => a
        _ => @utils.error("left expects a cons cell")
      }
    Right(t) =>
      match eval_t(t, env) {
        @utils.ConsValue(_, b) => b
        _ => @utils.error("right expects a cons cell")
      }
  }
}

///|
fn condition(
  b : Bexp,
  env : Array[(String, @utils.CharTree)],
) -> Bool raise @utils.Fault {
  match b {
    IsAtom(t) =>
      match eval_t(t, env) {
        @utils.AtomValue(_) => true
        _ => false
      }
    Equal(a, b) => {
      let a = eval_t(a, env)
      a == eval_t(b, env)
    }
  }
}

///|
fn small(
  s : Stmt,
  env : Array[(String, @utils.CharTree)],
) -> (Stmt, Array[(String, @utils.CharTree)]) raise @utils.Fault {
  match s {
    Nop => (Nop, env)
    Assign(n, a) => (Nop, @utils.env_set(env, n, eval_t(a, env)))
    Seq(Nop, b) => (b, env)
    Seq(a, b) => {
      let (a, env) = small(a, env)
      (Seq(a, b), env)
    }
    IfEq(c, b) => (if condition(c, env) { b } else { Nop }, env)
    While(c, b) => (IfEq(c, Seq(b, s)), env)
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
) -> @definition.Transition[State, Routput, Foutput] raise @utils.Fault {
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
    AtomChar(c) => @utils.tag("AtomChar", [@utils.jstr(c)])
    Mtoa(t) => @utils.tag("Mtoa", [tj(t)])
  }
}

///|
fn tj(t : Texp) -> Json {
  match t {
    Var(n) => @utils.tag("Var", [@utils.jstr(n)])
    Atom(a) => @utils.tag("Atom", [aj(a)])
    Cons(a, b) => @utils.tag("Cons", [tj(a), tj(b)])
    Left(t) => @utils.tag("Left", [tj(t)])
    Right(t) => @utils.tag("Right", [tj(t)])
  }
}

///|
fn ad(j : Json) -> Aexp raise @utils.Fault {
  match @utils.tag_name(j) {
    "AtomChar" => AtomChar(@utils.decode_scalar(@utils.tag_arg(j, 0)))
    "Mtoa" => Mtoa(td(@utils.tag_arg(j, 0)))
    _ => @utils.error("Invalid atom expression")
  }
}

///|
fn td(j : Json) -> Texp raise @utils.Fault {
  fn a(i : Int32) -> Json raise @utils.Fault {
    @utils.tag_arg(j, i)
  }
  match @utils.tag_name(j) {
    "Var" => Var(@utils.identifier(@utils.string(a(0))))
    "Atom" => Atom(ad(a(0)))
    "Cons" => Cons(td(a(0)), td(a(1)))
    "Left" => Left(td(a(0)))
    "Right" => Right(td(a(0)))
    _ => @utils.error("Invalid tree expression")
  }
}

///|
fn bj(b : Bexp) -> Json {
  match b {
    IsAtom(a) => @utils.tag("IsAtom", [tj(a)])
    Equal(a, b) => @utils.tag("Eq", [tj(a), tj(b)])
  }
}

///|
fn bd(j : Json) -> Bexp raise @utils.Fault {
  match @utils.tag_name(j) {
    "IsAtom" => IsAtom(td(@utils.tag_arg(j, 0)))
    "Eq" => Equal(td(@utils.tag_arg(j, 0)), td(@utils.tag_arg(j, 1)))
    _ => @utils.error("Invalid tree condition")
  }
}

///|
fn sj(s : Stmt) -> Json {
  match s {
    Nop => @utils.tag("Nop", [])
    Assign(n, a) => @utils.tag("Assign", [@utils.jstr(n), tj(a)])
    Seq(a, b) => @utils.tag("Seq", [sj(a), sj(b)])
    IfEq(c, b) => @utils.tag("IfEq", [bj(c), sj(b)])
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
    "Assign" => Assign(@utils.identifier(@utils.string(a(0))), td(a(1)))
    "Seq" => Seq(sd(a(0)), sd(a(1)))
    "IfEq" => IfEq(bd(a(0)), sd(a(1)))
    "While" => While(bd(a(0)), sd(a(1)))
    _ => @utils.error("Invalid tree statement")
  }
}

///|
fn encode(s : State) -> Json {
  @utils.object([
    ("code", sj(s.code)),
    ("stmt", sj(s.stmt)),
    ("env", @utils.env_json(s.env, @utils.tree_json)),
  ])
}

///|
fn decode(j : Json) -> State raise @utils.Fault {
  {
    code: sd(@utils.field(j, "code")),
    stmt: sd(@utils.field(j, "stmt")),
    env: @utils.decode_env(@utils.field(j, "env"), @utils.decode_tree),
  }
}

///|
fn at(a : Aexp) -> String {
  match a {
    AtomChar(c) => @utils.char_text(c)
    Mtoa(t) => "\\mtoa " + tt(t)
  }
}

///|
fn tt(t : Texp) -> String {
  match t {
    Var(n) => n
    Atom(a) => "\\atom " + at(a)
    Cons(a, b) => "\\cons " + tt(a) + " " + tt(b)
    Left(a) => "\\left " + tt(a)
    Right(a) => "\\right " + tt(a)
  }
}

///|
fn bt(b : Bexp) -> String {
  match b {
    IsAtom(a) => "\\is_atom " + tt(a)
    Equal(a, b) => "(" + tt(a) + " == " + tt(b) + ")"
  }
}

///|
fn st(s : Stmt) -> String {
  match s {
    Nop => "\\Nop"
    Assign(n, a) => n + " := " + tt(a)
    Seq(a, b) => st(a) + " ; " + st(b)
    IfEq(c, b) => "\\ifeq " + bt(c) + " \\then " + st(b) + " \\end"
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
