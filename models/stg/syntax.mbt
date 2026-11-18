///|
#derive(Eq)
priv enum Atom {
  Var(String)
  Lit(@base.Nat)
}

///|
#derive(Eq)
priv enum UpdateFlag {
  Updatable
  Reentrant
}

///|
#derive(Eq)
priv enum Prim {
  Add
  Sub
  Mul
  Eq
  Lt
}

///|
#derive(Eq)
priv enum Expr {
  App(String, Array[Atom])
  Con(String, Array[Atom])
  Nat(@base.Nat)
  Prim(Prim, Atom, Atom)
  Let(Bool, Array[(String, Lambda)], Expr)
  Case(Expr, Array[(Pattern, Expr)])
}

///|
#derive(Eq)
priv struct Lambda {
  free : Array[String]
  update : UpdateFlag
  args : Array[String]
  body : Expr
}

///|
#derive(Eq)
priv enum Pattern {
  Constructor(String, Array[String])
  Literal(@base.Nat)
  Default(String?)
}

///|
type Code = Array[(String, Lambda)]

///|
fn[T] list(
  p : @p.Parser,
  opening : Char,
  read : (@p.Parser) -> T raise @base.Fault,
) -> Array[T] raise @base.Fault {
  let g = p.group(opening)
  let xs = []
  while !g.eof() {
    xs.push(read(g))
    if g.eof() {
      break
    }
    g.expect(',')
    @base.require(!g.eof(), "Expected item after comma")
  }
  xs
}

///|
fn read_atom(p : @p.Parser) -> Atom raise @base.Fault {
  match p.peek() {
    @p.Ident(_) => Var(p.ident())
    @p.Number(_) => Lit(p.natural())
    _ => p.error("Expected a variable or natural number")
  }
}

///|
fn read_expr(p : @p.Parser) -> Expr raise @base.Fault {
  if p.peek() == @p.Symbol('(') {
    let g = p.group('(')
    let e = read_expr(g)
    g.finish()
    return e
  }
  match p.peek() {
    @p.Number(_) => Nat(p.natural())
    @p.Ident(_) => {
      let n = p.ident()
      App(n, list(p, '(', read_atom))
    }
    @p.Keyword("con") => {
      ignore(p.bump())
      let n = p.ident()
      Con(n, list(p, '(', read_atom))
    }
    @p.Keyword("let") | @p.Keyword("letrec") => {
      let rec = p.keyword() == "letrec"
      let g = p.group('{')
      let bs = read_bindings(g)
      p.expect_keyword("in")
      Let(rec, bs, read_expr(p))
    }
    @p.Keyword("case") => {
      ignore(p.bump())
      let e = read_expr(p)
      p.expect_keyword("of")
      let g = p.group('{')
      let alts = []
      while !g.eof() {
        alts.push(read_alternative(g))
        if g.eof() {
          break
        }
        g.expect(';')
      }
      Case(e, alts)
    }
    @p.Keyword("add")
    | @p.Keyword("sub")
    | @p.Keyword("mul")
    | @p.Keyword("eq")
    | @p.Keyword("lt") => {
      let o = match p.keyword() {
        "add" => Add
        "sub" => Sub
        "mul" => Mul
        "eq" => Eq
        _ => Lt
      }
      let xs = list(p, '(', read_atom)
      @base.require(xs.length() == 2, "Primitive operations require two atoms")
      Prim(o, xs[0], xs[1])
    }
    _ => p.error("Expected an STG expression")
  }
}

///|
fn read_bindings(p : @p.Parser) -> Code raise @base.Fault {
  let bs = []
  while !p.eof() {
    let n = p.ident()
    p.expect('=')
    p.expect_keyword("fun")
    let free = list(p, '[', p => p.ident())
    let update = match p.keyword() {
      "u" => Updatable
      "n" => Reentrant
      _ => p.error("Expected update flag \\u or \\n")
    }
    let args = list(p, '[', p => p.ident())
    p.expect_text("->")
    bs.push((n, { free, update, args, body: read_expr(p), }))
    if p.eof() {
      break
    }
    p.expect(';')
  }
  bs
}

///|
fn read_alternative(p : @p.Parser) -> (Pattern, Expr) raise @base.Fault {
  let pattern = match p.peek() {
    @p.Number(_) => Literal(p.natural())
    @p.Ident(_) => {
      let n = p.ident()
      Constructor(n, list(p, '(', p => p.ident()))
    }
    @p.Keyword("default") => {
      ignore(p.bump())
      Default(
        match p.peek() {
          @p.Ident(_) => Some(p.ident())
          _ => None
        },
      )
    }
    _ => p.error("Expected a constructor, literal or \\default alternative")
  }
  p.expect_text("->")
  (pattern, read_expr(p))
}

///|
fn unique(label : String, names : Array[String]) -> Unit raise @base.Fault {
  @data.unique(names, "Duplicate " + label + ": ")
}

///|
fn[T] append(a : Array[T], b : Array[T]) -> Array[T] {
  let a = a.copy()
  a.append(b)
  a
}

///|
fn bound(scope : Array[String], x : String) -> Unit raise @base.Fault {
  @base.require(scope.contains(x), "Unbound variable: " + x)
}

///|
fn check_atom(scope : Array[String], a : Atom) -> Unit raise @base.Fault {
  match a {
    Var(x) => bound(scope, x)
    _ => ()
  }
}

///|
fn check_expr(
  globals : Array[String],
  scope : Array[String],
  e : Expr,
) -> Unit raise @base.Fault {
  match e {
    Nat(_) => ()
    App(f, xs) => {
      bound(scope, f)
      for a in xs {
        check_atom(scope, a)
      }
    }
    Con(_, xs) =>
      for a in xs {
        check_atom(scope, a)
      }
    Prim(_, a, b) => {
      check_atom(scope, a)
      check_atom(scope, b)
    }
    Let(rec, bs, body) => {
      @base.require(!bs.is_empty(), "Empty let binding group")
      let names = bs.map(b => b.0)
      unique("binding", names)
      let inner = append(names, scope)
      for b in bs {
        check_lambda(globals, if rec { inner } else { scope }, b.1)
      }
      check_expr(globals, inner, body)
    }
    Case(e, alts) => {
      check_expr(globals, scope, e)
      check_alternatives(globals, scope, alts)
    }
  }
}

///|
fn check_lambda(
  globals : Array[String],
  scope : Array[String],
  lf : Lambda,
) -> Unit raise @base.Fault {
  unique("capture/parameter", append(lf.free, lf.args))
  @base.require(
    !(lf.update == Updatable && !lf.args.is_empty()),
    "An updatable lambda must have no parameters",
  )
  for x in lf.free {
    bound(scope, x)
  }
  let globals = globals.filter(x => scope.filter(y => y == x).length() <= 1)
  check_expr(globals, append(lf.args, append(lf.free, globals)), lf.body)
}

///|
fn check_alternatives(
  globals : Array[String],
  scope : Array[String],
  alts : Array[(Pattern, Expr)],
) -> Unit raise @base.Fault {
  @base.require(!alts.is_empty(), "Empty case alternatives")
  let keys = alts.map(a => {
    match a.0 {
      Constructor(c, _) => "con:" + c
      Literal(n) => "nat:" + n.to_string()
      Default(_) => "default"
    }
  })
  unique("case alternative", keys)
  for alt in alts {
    let vars = match alt.0 {
      Constructor(_, xs) => xs
      Default(Some(x)) => [x]
      _ => []
    }
    unique("pattern variable", vars)
    check_expr(globals, append(vars, scope), alt.1)
  }
}

///|
fn validate(code : Code) -> Unit raise @base.Fault {
  let globals = code.map(b => b.0)
  unique("global binding", globals)
  @base.require(globals.contains("main"), "Missing main binding")
  for b in code {
    @base.require(
      b.1.free.is_empty(),
      "Top-level lambdas cannot capture local variables",
    )
    check_lambda(globals, globals, b.1)
  }
  let main = code.iter().find_first(b => b.0 == "main").unwrap().1
  @base.require(main.args.is_empty(), "main must have no parameters")
}

///|
fn parse(text : String) -> Code raise @base.Fault {
  let p = @p.create(text)
  let code = read_bindings(p)
  validate(code)
  code
}

///|
fn atom_text(a : Atom) -> String {
  match a {
    Var(x) => x
    Lit(n) => n.to_string()
  }
}

///|
fn atoms(xs : Array[Atom]) -> String {
  "(" + @base.join(xs.map(atom_text), ", ") + ")"
}

///|
fn prim_text(o : Prim) -> String {
  match o {
    Add => "add"
    Sub => "sub"
    Mul => "mul"
    Eq => "eq"
    Lt => "lt"
  }
}

///|
fn expr_text(e : Expr) -> String {
  match e {
    App(f, xs) => f + atoms(xs)
    Con(c, xs) => "\\con " + c + atoms(xs)
    Nat(n) => n.to_string()
    Prim(o, a, b) => "\\" + prim_text(o) + atoms([a, b])
    Let(rec, bs, b) =>
      (if rec { "\\letrec" } else { "\\let" }) +
      " { " +
      code_text(bs) +
      " } \\in " +
      expr_text(b)
    Case(e, alts) =>
      "\\case " +
      expr_text(e) +
      " \\of { " +
      @base.join(
        alts.map(a => pattern_text(a.0) + " -> " + expr_text(a.1)),
        "; ",
      ) +
      " }"
  }
}

///|
fn pattern_text(p : Pattern) -> String {
  match p {
    Constructor(c, xs) => c + "(" + @base.join(xs, ", ") + ")"
    Literal(n) => n.to_string()
    Default(None) => "\\default"
    Default(Some(x)) => "\\default " + x
  }
}

///|
fn lambda_text(lf : Lambda) -> String {
  "\\fun [" +
  @base.join(lf.free, ", ") +
  "] " +
  (if lf.update == Updatable { "\\u" } else { "\\n" }) +
  " [" +
  @base.join(lf.args, ", ") +
  "] -> " +
  expr_text(lf.body)
}

///|
fn code_text(bs : Code) -> String {
  @base.join(bs.map(b => b.0 + " = " + lambda_text(b.1)), "; ")
}
