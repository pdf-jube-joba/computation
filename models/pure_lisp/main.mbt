///|
#derive(Eq)
priv enum Sexp {
  Nil
  Symbol(String)
  Cons(Sexp, Sexp)
}

///|
priv enum Frame {
  Operator(Array[Sexp], Array[(String, Sexp)])
  Argument(Sexp, Array[Sexp], Array[Sexp], Array[(String, Sexp)])
  Branch(Sexp, Array[(Sexp, Sexp)], Array[(String, Sexp)])
}

///|
priv enum Control {
  Eval(Sexp, Array[(String, Sexp)])
  Return(Sexp, Array[(String, Sexp)])
  Apply(Sexp, Array[Sexp], Array[(String, Sexp)])
}

///|
priv struct State {
  code : Sexp
  control : Control
  kont : Array[Frame]
}

///|
fn list(xs : Array[Sexp], tail~ : Sexp = Nil) -> Sexp {
  let mut s = tail
  for i = xs.length() - 1; i >= 0; i = i - 1 {
    s = Cons(xs[i], s)
  }
  s
}

///|
fn elements(s : Sexp) -> Array[Sexp] raise @base.Fault {
  let xs = []
  for value = s {
    match value {
      Nil => break
      Cons(a, b) => {
        xs.push(a)
        continue b
      }
      _ => @base.error("Expected a proper list")
    }
  }
  xs
}

///|
fn print(s : Sexp) -> String {
  match s {
    Nil => "nil"
    Symbol(s) => s
    Cons(a, b) => {
      let mut text = "(" + print(a)
      for tail = b {
        match tail {
          Nil => break
          Cons(a, b) => {
            text += " " + print(a)
            continue b
          }
          b => {
            text += " . " + print(b)
            break
          }
        }
      }
      text + ")"
    }
  }
}

///|
priv struct Reader {
  chars : Array[Char]
  mut pos : Int32
}

///|
fn delimiter(c : Char) -> Bool {
  c == ' ' ||
  c == '\t' ||
  c == '\r' ||
  c == '\n' ||
  c == '(' ||
  c == ')' ||
  c == '\'' ||
  c == ';'
}

///|
fn Reader::skip(self : Reader) -> Unit {
  while self.pos < self.chars.length() {
    let c = self.chars[self.pos]
    if c == ' ' || c == '\t' || c == '\r' || c == '\n' {
      self.pos += 1
    } else if c == ';' {
      while self.pos < self.chars.length() && self.chars[self.pos] != '\n' {
        self.pos += 1
      }
    } else {
      break
    }
  }
}

///|
fn Reader::dot(self : Reader) -> Bool {
  self.pos < self.chars.length() &&
  self.chars[self.pos] == '.' &&
  (self.pos + 1 == self.chars.length() || delimiter(self.chars[self.pos + 1]))
}

///|
fn Reader::read(self : Reader) -> Sexp raise @base.Fault {
  self.skip()
  @base.require(self.pos < self.chars.length(), "Expected an S-expression")
  let c = self.chars[self.pos]
  self.pos += 1
  if c == '\'' {
    return list([Symbol("quote"), self.read()])
  }
  if c == '(' {
    let items = []
    while true {
      self.skip()
      @base.require(self.pos < self.chars.length(), "Unclosed list")
      if self.chars[self.pos] == ')' {
        self.pos += 1
        return list(items)
      }
      if self.dot() {
        @base.require(!items.is_empty(), "Dot requires a preceding element")
        self.pos += 1
        let tail = self.read()
        self.skip()
        @base.require(
          self.pos < self.chars.length() && self.chars[self.pos] == ')',
          "Expected ')' after dotted tail",
        )
        self.pos += 1
        return list(items, tail~)
      }
      items.push(self.read())
    }
  }
  @base.require(c != ')', "Unexpected ')'")
  self.pos -= 1
  @base.require(!self.dot(), "Unexpected dot")
  let start = self.pos
  while self.pos < self.chars.length() && !delimiter(self.chars[self.pos]) {
    self.pos += 1
  }
  let name = String::from_array(self.chars[start:self.pos])
  if name == "nil" {
    Nil
  } else {
    Symbol(name)
  }
}

///|
fn parse(text : String) -> Sexp raise @base.Fault {
  let r = { chars: text.to_array(), pos: 0, }
  let s = r.read()
  r.skip()
  @base.require(r.pos == r.chars.length(), "Expected one S-expression")
  s
}

///|
let primitives : Array[String] = ["atom", "eq", "cons", "car", "cdr", "eval"]

///|
fn make(code : String, input : String) -> State raise @base.Fault {
  @base.unit_input(input)
  let code = parse(code)
  { code, control: Eval(code, primitives.map(n => (n, Symbol(n)))), kont: [], }
}

///|
fn binder(s : Sexp) -> String raise @base.Fault {
  match s {
    Symbol(n) if n != "t" => n
    _ => @base.error("Expected a variable name (not nil or t)")
  }
}

///|
fn lambda(s : Sexp) -> (Array[String], Sexp) raise @base.Fault {
  match s {
    Cons(Symbol("lambda"), Cons(params, Cons(body, Nil))) => {
      let names = elements(params).map(binder)
      let seen : Map[String, Bool] = Map([])
      for name in names {
        @base.require(!seen.contains(name), "Duplicate parameter: " + name)
        seen[name] = true
      }
      (names, body)
    }
    _ => @base.error("Expected (lambda (parameters ...) body)")
  }
}

///|
fn labelled(s : Sexp) -> (String, Array[String], Sexp) raise @base.Fault {
  match s {
    Cons(Symbol("label"), Cons(name, Cons(f, Nil))) => {
      let name = binder(name)
      let (params, body) = lambda(f)
      (name, params, body)
    }
    _ => @base.error("Expected (label name (lambda (parameters ...) body))")
  }
}

///|
fn push(frame : Frame, kont : Array[Frame]) -> Array[Frame] {
  let result = [frame]
  result.append(kont)
  result
}

///|
fn branch(
  env : Array[(String, Sexp)],
  kont : Array[Frame],
  choices : Array[(Sexp, Sexp)],
) -> (Control, Array[Frame]) {
  if choices.is_empty() {
    (Return(Nil, env), kont)
  } else {
    (
      Eval(choices[0].0, env),
      push(Branch(choices[0].1, choices[1:].to_owned(), env), kont),
    )
  }
}

///|
fn arguments(
  f : Sexp,
  args : Array[Sexp],
  env : Array[(String, Sexp)],
  kont : Array[Frame],
) -> (Control, Array[Frame]) {
  if args.is_empty() {
    (Apply(f, [], env), kont)
  } else {
    (Eval(args[0], env), push(Argument(f, [], args[1:].to_owned(), env), kont))
  }
}

///|
fn truth(b : Bool) -> Sexp {
  if b {
    Symbol("t")
  } else {
    Nil
  }
}

///|
fn apply(
  f : Sexp,
  args : Array[Sexp],
  env : Array[(String, Sexp)],
) -> Control raise @base.Fault {
  fn bind(
    names : Array[String],
    body : Sexp,
    env : Array[(String, Sexp)],
  ) -> Control raise @base.Fault {
    @base.require(
      names.length() == args.length(),
      "Wrong number of arguments to lambda",
    )
    let e = names.mapi((i, n) => (n, args[i]))
    e.append(env)
    Eval(body, e)
  }
  match f {
    Symbol(name) if primitives.contains(name) => {
      @base.require(
        args.length() == (if name == "eq" || name == "cons" { 2 } else { 1 }),
        "Wrong number of arguments to " + name,
      )
      match name {
        "atom" =>
          Return(
            truth(
              match args[0] {
                Cons(_, _) => false
                _ => true
              },
            ),
            env,
          )
        "eq" =>
          Return(
            truth(
              match (args[0], args[1]) {
                (Nil, Nil) => true
                (Symbol(a), Symbol(b)) => a == b
                _ => false
              },
            ),
            env,
          )
        "cons" => Return(Cons(args[0], args[1]), env)
        "car" | "cdr" =>
          Return(
            match args[0] {
              Nil => Nil
              Cons(a, b) => if name == "car" { a } else { b }
              _ => @base.error("car/cdr expects a pair or nil")
            },
            env,
          )
        _ => Eval(args[0], env)
      }
    }
    Cons(Symbol("lambda"), _) => {
      let (names, body) = lambda(f)
      bind(names, body, env)
    }
    Cons(Symbol("label"), _) => {
      let (name, names, body) = labelled(f)
      let e = [(name, f)]
      e.append(env)
      bind(names, body, e)
    }
    _ => @base.error("Not a function: " + print(f))
  }
}

///|
fn transition(s : State) -> (Control, Array[Frame]) raise @base.Fault {
  match s.control {
    Eval(Nil | Symbol("t") as v, env) => (Return(v, env), s.kont)
    Eval(Symbol(name), env) => {
      for pair in env {
        if pair.0 == name {
          return (Return(pair.1, env), s.kont)
        }
      }
      @base.error("Unbound symbol: " + name)
    }
    Eval(Cons(op, operands) as expression, env) => {
      let args = elements(operands)
      match op {
        Symbol("quote") => {
          @base.require(args.length() == 1, "quote expects one argument")
          (Return(args[0], env), s.kont)
        }
        Symbol("lambda") => {
          ignore(lambda(expression))
          (Return(expression, env), s.kont)
        }
        Symbol("label") => {
          ignore(labelled(expression))
          (Return(expression, env), s.kont)
        }
        Symbol("cond") =>
          branch(
            env,
            s.kont,
            args.map(a => {
              let xs = elements(a)
              @base.require(
                xs.length() == 2,
                "Each cond clause must be (test body)",
              )
              (xs[0], xs[1])
            }),
          )
        _ => (Eval(op, env), push(Operator(args, env), s.kont))
      }
    }
    Apply(f, args, env) => (apply(f, args, env), s.kont)
    Return(value, _) => {
      @base.require(!s.kont.is_empty(), "Evaluation already finished")
      let kont = s.kont[1:].to_owned()
      match s.kont[0] {
        Operator(args, env) => arguments(value, args, env, kont)
        Argument(f, done_rev, rest, env) => {
          let values = [value]
          values.append(done_rev)
          if rest.is_empty() {
            values.rev_in_place()
            (Apply(f, values, env), kont)
          } else {
            (
              Eval(rest[0], env),
              push(Argument(f, values, rest[1:].to_owned(), env), kont),
            )
          }
        }
        Branch(body, rest, env) =>
          if value == Nil {
            branch(env, kont, rest)
          } else {
            (Eval(body, env), kont)
          }
      }
    }
  }
}

///|
fn step(
  s : State,
  input : String,
) -> @runtime.Transition[State] raise @base.Fault {
  @base.unit_input(input)
  match s.control {
    Return(v, _) if s.kont.is_empty() => @runtime.Halt(print(v))
    _ => {
      let (control, kont) = transition(s)
      @runtime.Continue({ ..s, control, kont, }, "")
    }
  }
}

///|
fn sj(s : Sexp) -> Json {
  match s {
    Nil => @base.tag("Nil", [])
    Symbol(n) => @base.tag("Symbol", [@base.jstr(n)])
    Cons(a, b) => @base.tag("Cons", [sj(a), sj(b)])
  }
}

///|
fn sd(j : Json) -> Sexp raise @base.Fault {
  match @base.tag_name(j) {
    "Nil" => Nil
    "Symbol" => Symbol(@base.string(@base.tag_arg(j, 0)))
    "Cons" => Cons(sd(@base.tag_arg(j, 0)), sd(@base.tag_arg(j, 1)))
    _ => @base.error("Invalid S-expression")
  }
}

///|
fn ej(env : Array[(String, Sexp)]) -> Json {
  @base.jarray(env.map(p => @base.jarray([@base.jstr(p.0), sj(p.1)])))
}

///|
fn ed(j : Json) -> Array[(String, Sexp)] raise @base.Fault {
  @base.array(j).map(j => {
    let a = @base.array(j)
    @base.require(a.length() == 2, "Invalid environment entry")
    (@base.string(a[0]), sd(a[1]))
  })
}

///|
fn fj(f : Frame) -> Json {
  match f {
    Operator(args, env) =>
      @base.tag("Operator", [@base.jarray(args.map(sj)), ej(env)])
    Argument(f, done, rest, env) =>
      @base.tag("Argument", [
        sj(f),
        @base.jarray(done.map(sj)),
        @base.jarray(rest.map(sj)),
        ej(env),
      ])
    Branch(b, rest, env) =>
      @base.tag("Branch", [
        sj(b),
        @base.jarray(rest.map(p => @base.jarray([sj(p.0), sj(p.1)]))),
        ej(env),
      ])
  }
}

///|
fn fd(j : Json) -> Frame raise @base.Fault {
  fn a(i : Int32) -> Json raise @base.Fault {
    @base.tag_arg(j, i)
  }
  match @base.tag_name(j) {
    "Operator" => Operator(@base.array(a(0)).map(sd), ed(a(1)))
    "Argument" =>
      Argument(
        sd(a(0)),
        @base.array(a(1)).map(sd),
        @base.array(a(2)).map(sd),
        ed(a(3)),
      )
    "Branch" =>
      Branch(
        sd(a(0)),
        @base.array(a(1)).map(j => {
          let a = @base.array(j)
          @base.require(a.length() == 2, "Invalid branch")
          (sd(a[0]), sd(a[1]))
        }),
        ed(a(2)),
      )
    _ => @base.error("Invalid continuation frame")
  }
}

///|
fn cj(c : Control) -> Json {
  match c {
    Eval(s, env) => @base.tag("Eval", [sj(s), ej(env)])
    Return(s, env) => @base.tag("Return", [sj(s), ej(env)])
    Apply(f, args, env) =>
      @base.tag("Apply", [sj(f), @base.jarray(args.map(sj)), ej(env)])
  }
}

///|
fn cd(j : Json) -> Control raise @base.Fault {
  fn a(i : Int32) -> Json raise @base.Fault {
    @base.tag_arg(j, i)
  }
  match @base.tag_name(j) {
    "Eval" => Eval(sd(a(0)), ed(a(1)))
    "Return" => Return(sd(a(0)), ed(a(1)))
    "Apply" => Apply(sd(a(0)), @base.array(a(1)).map(sd), ed(a(2)))
    _ => @base.error("Invalid control")
  }
}

///|
fn encode(s : State) -> Json {
  @base.object([
    ("code", sj(s.code)),
    ("control", cj(s.control)),
    ("kont", @base.jarray(s.kont.map(fj))),
  ])
}

///|
fn decode(j : Json) -> State raise @base.Fault {
  {
    code: sd(@base.field(j, "code")),
    control: cd(@base.field(j, "control")),
    kont: @base.array(@base.field(j, "kont")).map(fd),
  }
}

///|
pub fn model() -> @runtime.Model {
  @runtime.model(make, step, encode, decode, s => {
    let (mode, focus, env) = match s.control {
      Eval(e, env) => ("eval", print(e), env)
      Return(v, env) => ("return", print(v), env)
      Apply(f, args, env) => {
        let xs = [f]
        xs.append(args)
        ("apply", print(list(xs)), env)
      }
    }
    @base.jarray([
      @base.titled("code", print(s.code)),
      @base.table("state", ["field", "value"], [
        ["mode", mode],
        ["focus", focus],
      ]),
      @base.table("env", ["name", "value"], env.map(p => [p.0, print(p.1)])),
      @base.table(
        "kont",
        ["depth", "frame"],
        s.kont.mapi((i, f) => [i.to_string(), fj(f).stringify()]),
      ),
    ])
  })
}
