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
priv enum Stmt {
  Nop
  Assign(String, Texp)
  Seq(Stmt, Stmt)
  IfEq(Bexp, Stmt)
  While(Bexp, Stmt)
}

///|
priv struct State {
  code : Stmt
  stmt : Stmt
  env : Array[(String, @data.CharTree)]
}

///|
fn aexp(p : @p.Parser) -> Aexp raise @base.Fault {
  if p.eat_keyword("mtoa") {
    Mtoa(texp(p))
  } else if p.peek() == @p.Symbol('(') {
    let g = p.group('(')
    let a = aexp(g)
    g.finish()
    a
  } else {
    AtomChar(p.character())
  }
}

///|
fn texp(p : @p.Parser) -> Texp raise @base.Fault {
  if p.eat_keyword("atom") {
    Atom(aexp(p))
  } else if p.eat_keyword("cons") {
    let a = texp(p)
    Cons(a, texp(p))
  } else if p.eat_keyword("left") {
    Left(texp(p))
  } else if p.eat_keyword("right") {
    Right(texp(p))
  } else if p.peek() == @p.Symbol('(') {
    let g = p.group('(')
    let a = texp(g)
    g.finish()
    a
  } else {
    Var(p.ident())
  }
}

///|
fn relation(p : @p.Parser) -> Bexp raise @base.Fault {
  let a = texp(p)
  p.expect_text("==")
  Equal(a, texp(p))
}

///|
fn bexp(p : @p.Parser) -> Bexp raise @base.Fault {
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
fn single(p : @p.Parser) -> Stmt raise @base.Fault {
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
  } else if p.peek() == @p.Symbol('(') {
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
fn statement(p : @p.Parser) -> Stmt raise @base.Fault {
  let mut s = single(p)
  while p.eat(';') {
    s = Seq(s, single(p))
  }
  s
}

///|
fn eval_a(
  a : Aexp,
  env : Array[(String, @data.CharTree)],
) -> String raise @base.Fault {
  match a {
    AtomChar(c) => c
    Mtoa(t) =>
      match eval_t(t, env) {
        @data.AtomValue(c) => c
        _ => @base.error("mtoa expects an atom")
      }
  }
}

///|
fn eval_t(
  t : Texp,
  env : Array[(String, @data.CharTree)],
) -> @data.CharTree raise @base.Fault {
  match t {
    Var(n) =>
      match @data.env_get(env, n) {
        Some(v) => v
        None => @base.error("uninitialized variable: " + n)
      }
    Atom(a) => @data.AtomValue(eval_a(a, env))
    Cons(a, b) => {
      let a = eval_t(a, env)
      @data.ConsValue(a, eval_t(b, env))
    }
    Left(t) =>
      match eval_t(t, env) {
        @data.ConsValue(a, _) => a
        _ => @base.error("left expects a cons cell")
      }
    Right(t) =>
      match eval_t(t, env) {
        @data.ConsValue(_, b) => b
        _ => @base.error("right expects a cons cell")
      }
  }
}

///|
fn condition(
  b : Bexp,
  env : Array[(String, @data.CharTree)],
) -> Bool raise @base.Fault {
  match b {
    IsAtom(t) =>
      match eval_t(t, env) {
        @data.AtomValue(_) => true
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
  env : Array[(String, @data.CharTree)],
) -> (Stmt, Array[(String, @data.CharTree)]) raise @base.Fault {
  match s {
    Nop => (Nop, env)
    Assign(n, a) => (Nop, @data.env_set(env, n, eval_t(a, env)))
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
fn make(code : String, input : String) -> State raise @base.Fault {
  let p = @p.create(code, character_literals=true)
  let code = statement(p)
  p.finish()
  { code, stmt: code, env: @data.parse_env(input, @data.parse_tree), }
}

///|
fn step(
  s : State,
  input : String,
) -> @runtime.Transition[State] raise @base.Fault {
  @base.unit_input(input)
  match s.stmt {
    Nop => @runtime.Halt(@data.print_env(s.env, @data.tree_text))
    _ => {
      let (stmt, env) = small(s.stmt, s.env)
      @runtime.Continue({ ..s, stmt, env, }, "")
    }
  }
}

///|
fn aj(a : Aexp) -> Json {
  match a {
    AtomChar(c) => @base.tag("AtomChar", [@base.jstr(c)])
    Mtoa(t) => @base.tag("Mtoa", [tj(t)])
  }
}

///|
fn tj(t : Texp) -> Json {
  match t {
    Var(n) => @base.tag("Var", [@base.jstr(n)])
    Atom(a) => @base.tag("Atom", [aj(a)])
    Cons(a, b) => @base.tag("Cons", [tj(a), tj(b)])
    Left(t) => @base.tag("Left", [tj(t)])
    Right(t) => @base.tag("Right", [tj(t)])
  }
}

///|
fn ad(j : Json) -> Aexp raise @base.Fault {
  match @base.tag_name(j) {
    "AtomChar" => AtomChar(@data.decode_scalar(@base.tag_arg(j, 0)))
    "Mtoa" => Mtoa(td(@base.tag_arg(j, 0)))
    _ => @base.error("Invalid atom expression")
  }
}

///|
fn td(j : Json) -> Texp raise @base.Fault {
  fn a(i : Int32) -> Json raise @base.Fault {
    @base.tag_arg(j, i)
  }
  match @base.tag_name(j) {
    "Var" => Var(@base.identifier(@base.string(a(0))))
    "Atom" => Atom(ad(a(0)))
    "Cons" => Cons(td(a(0)), td(a(1)))
    "Left" => Left(td(a(0)))
    "Right" => Right(td(a(0)))
    _ => @base.error("Invalid tree expression")
  }
}

///|
fn bj(b : Bexp) -> Json {
  match b {
    IsAtom(a) => @base.tag("IsAtom", [tj(a)])
    Equal(a, b) => @base.tag("Eq", [tj(a), tj(b)])
  }
}

///|
fn bd(j : Json) -> Bexp raise @base.Fault {
  match @base.tag_name(j) {
    "IsAtom" => IsAtom(td(@base.tag_arg(j, 0)))
    "Eq" => Equal(td(@base.tag_arg(j, 0)), td(@base.tag_arg(j, 1)))
    _ => @base.error("Invalid tree condition")
  }
}

///|
fn sj(s : Stmt) -> Json {
  match s {
    Nop => @base.tag("Nop", [])
    Assign(n, a) => @base.tag("Assign", [@base.jstr(n), tj(a)])
    Seq(a, b) => @base.tag("Seq", [sj(a), sj(b)])
    IfEq(c, b) => @base.tag("IfEq", [bj(c), sj(b)])
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
    "Assign" => Assign(@base.identifier(@base.string(a(0))), td(a(1)))
    "Seq" => Seq(sd(a(0)), sd(a(1)))
    "IfEq" => IfEq(bd(a(0)), sd(a(1)))
    "While" => While(bd(a(0)), sd(a(1)))
    _ => @base.error("Invalid tree statement")
  }
}

///|
fn encode(s : State) -> Json {
  @base.object([
    ("code", sj(s.code)),
    ("stmt", sj(s.stmt)),
    ("env", @data.env_json(s.env, @data.tree_json)),
  ])
}

///|
fn decode(j : Json) -> State raise @base.Fault {
  {
    code: sd(@base.field(j, "code")),
    stmt: sd(@base.field(j, "stmt")),
    env: @data.decode_env(@base.field(j, "env"), @data.decode_tree),
  }
}

///|
fn at(a : Aexp) -> String {
  match a {
    AtomChar(c) => @data.char_text(c)
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
pub fn model() -> @runtime.Model {
  @runtime.model(make, step, encode, decode, s => {
    @base.jarray([
      @base.titled("stmt", st(s.stmt)),
      @base.table(
        "env",
        ["var", "value"],
        s.env.map(pair => [pair.0, @data.tree_text(pair.1)]),
      ),
    ])
  })
}
