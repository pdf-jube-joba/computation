///|
priv enum Term {
  Var(String)
  Lam(String, Term)
  App(Term, Term)
  Zero
  Succ(Term)
  Pred(Term)
  IfZ(Term, Term, Term)
  Let(String, Term, Term)
  Rec(String, String, Term)
  Abort(Term)
  Control(Term)
}

///|
priv struct State {
  code : Term
  current : Term
}

///|
fn number(n : @base.Nat) -> Term {
  let mut t = Zero
  let mut n = n
  while !n.is_zero() {
    t = Succ(t)
    n = @base.pred(n)
  }
  t
}

///|
fn as_number(t : Term) -> @base.Nat? {
  match t {
    Zero => Some(@base.zero())
    Succ(t) => as_number(t).map(n => n + @base.one())
    _ => None
  }
}

///|
fn is_value(t : Term) -> Bool {
  match t {
    Lam(_, _) => true
    _ => as_number(t) is Some(_)
  }
}

///|
fn view(t : Term) -> @names.View[Term] {
  match t {
    Var(n) => @names.Variable(n)
    Lam(x, b) => @names.Node([x], [([0], b)], (ns, cs) => Lam(ns[0], cs[0]))
    Rec(f, x, b) =>
      @names.Node([f, x], [([0, 1], b)], (ns, cs) => Rec(ns[0], ns[1], cs[0]))
    Let(x, e, b) =>
      @names.Node([x], [([], e), ([0], b)], (ns, cs) => Let(ns[0], cs[0], cs[1]))
    App(a, b) =>
      @names.Node([], [([], a), ([], b)], (_, cs) => App(cs[0], cs[1]))
    IfZ(c, t, f) =>
      @names.Node([], [([], c), ([], t), ([], f)], (_, cs) => {
        IfZ(cs[0], cs[1], cs[2])
      })
    Succ(t) => @names.Node([], [([], t)], (_, cs) => Succ(cs[0]))
    Pred(t) => @names.Node([], [([], t)], (_, cs) => Pred(cs[0]))
    Abort(t) => @names.Node([], [([], t)], (_, cs) => Abort(cs[0]))
    Control(t) => @names.Node([], [([], t)], (_, cs) => Control(cs[0]))
    Zero => @names.Node([], [], (_, _) => Zero)
  }
}

///|
fn subst(t : Term, x : String, r : Term) -> Term {
  @names.substitute({ view, variable: x => Var(x), }, t, x, r)
}

///|
fn read(p : @p.Parser) -> Term raise @base.Fault {
  if p.eat_keyword("fun") {
    let x = p.ident()
    p.expect_text("=>")
    Lam(x, read(p))
  } else if p.eat('@') {
    let g = p.group('(')
    let a = read(g)
    let b = read(g)
    g.finish()
    App(a, b)
  } else if p.peek() == @p.Symbol('(') {
    let g = p.group('(')
    let a = read(g)
    let t = if g.eof() { a } else { App(a, read(g)) }
    g.finish()
    t
  } else if p.eat_keyword("zero") {
    Zero
  } else if p.peek() == @p.Number("0") {
    ignore(p.bump())
    Zero
  } else if p.eat_keyword("succ") || p.eat_keyword("S") {
    Succ(read(p))
  } else if p.eat_keyword("pred") || p.eat_keyword("P") {
    Pred(read(p))
  } else if p.eat_keyword("ifz") || p.eat_keyword("if") {
    let c = read(p)
    p.expect_keyword("then")
    let t = read(p)
    p.expect_keyword("else")
    IfZ(c, t, read(p))
  } else if p.eat_keyword("let") {
    let x = p.ident()
    p.expect('=')
    let e = read(p)
    p.expect_keyword("in")
    Let(x, e, read(p))
  } else if p.eat_keyword("rec") {
    let f = p.ident()
    let x = p.ident()
    p.expect('=')
    Rec(f, x, read(p))
  } else {
    Var(p.ident())
  }
}

///|
fn pretty(t : Term) -> String {
  match t {
    Var(x) => x
    Lam(x, b) => "\\fun " + x + " => " + pretty(b)
    App(a, b) => "(" + pretty(a) + " " + pretty(b) + ")"
    Zero => "0"
    Succ(t) => "\\S " + pretty(t)
    Pred(t) => "\\P " + pretty(t)
    IfZ(c, t, f) =>
      "\\if " + pretty(c) + " \\then " + pretty(t) + " \\else " + pretty(f)
    Let(x, e, b) => "\\let " + x + " = " + pretty(e) + " \\in \n " + pretty(b)
    Rec(f, x, b) => "\\rec " + f + " " + x + " = " + pretty(b)
    Abort(t) => "\\abort " + pretty(t)
    Control(t) => "\\control " + pretty(t)
  }
}

///|
fn reduce(t : Term) -> Term? {
  match t {
    Var(_) | Lam(_, _) | Zero => None
    App(f, a) =>
      if is_value(f) {
        if is_value(a) {
          match f {
            Lam(x, b) => Some(subst(b, x, a))
            _ => None
          }
        } else {
          reduce(a).map(a => App(f, a))
        }
      } else {
        reduce(f).map(f => App(f, a))
      }
    Succ(t) => if is_value(t) { None } else { reduce(t).map(t => Succ(t)) }
    Pred(t) =>
      match as_number(t) {
        Some(n) => Some(number(@base.pred(n)))
        None => if is_value(t) { None } else { reduce(t).map(t => Pred(t)) }
      }
    IfZ(c, t, f) =>
      match as_number(c) {
        Some(n) => Some(if n.is_zero() { t } else { f })
        None =>
          if is_value(c) {
            None
          } else {
            reduce(c).map(c => IfZ(c, t, f))
          }
      }
    Let(x, e, b) => Some(App(Lam(x, b), e))
    Rec(f, x, b) as r => Some(Lam(x, subst(b, f, r)))
    Abort(_) | Control(_) => None
  }
}

///|
fn make(text : String, input : String) -> State raise @base.Fault {
  let p = @p.create(text)
  let code = read(p)
  p.finish()
  let code = @names.normalize({ view, variable: x => Var(x), }, code)
  let mut current = code
  for n in @pcf.inputs(input) {
    current = App(current, number(n))
  }
  { code, current, }
}

///|
fn step(
  s : State,
  input : String,
) -> @runtime.Transition[State] raise @base.Fault {
  @base.unit_input(input)
  match reduce(s.current) {
    Some(current) => @runtime.Continue({ ..s, current, }, "")
    None => {
      @base.require(is_value(s.current), "stuck term: " + pretty(s.current))
      @runtime.Halt(pretty(s.current))
    }
  }
}

///|
fn encode(s : State) -> Json {
  @base.object([("code", term_json(s.code)), ("current", term_json(s.current))])
}

///|
fn decode(j : Json) -> State raise @base.Fault {
  {
    code: decode_term(@base.field(j, "code")),
    current: decode_term(@base.field(j, "current")),
  }
}

///|
pub fn model() -> @runtime.Model {
  @runtime.model(make, step, encode, decode, s => {
    @base.jarray([
      @base.titled("code", pretty(s.code)),
      @base.titled("current", pretty(s.current)),
    ])
  })
}

///|
test "natural recursion and let desugaring" {
  let m = model()
  m.make("\\let x = \\S \\S 0 \\in \\P x", "()")
  for _ in 0..<3 {
    ignore(m.step(""))
  }
  assert_eq(
    @base.string(@base.field(@base.parse_json(m.step("")), "foutput")),
    "\\S 0",
  )
}
