///|
pub(all) struct Variable {
  id : Int32
  name : String
}

///|
pub(all) enum Term {
  Var(Variable)
  Abs(Variable, Term)
  App(Term, Term)
}

///|
pub fn fresh(next : Ref[Int32], name : String) -> Variable {
  let id = next.val
  next.val += 1
  { id, name, }
}

///|
fn read(p : @p.Parser) -> Term raise @base.Fault {
  let next : Ref[Int32] = { val: 0, }
  let free : Map[String, Variable] = Map([])
  application(p, Map([]), next, free)
}

///|
fn application(
  p : @p.Parser,
  env : Map[String, Variable],
  next : Ref[Int32],
  free : Map[String, Variable],
) -> Term raise @base.Fault {
  let mut term = atom(p, env, next, free)
  while (match p.peek() {
          @p.Ident(_) | @p.Keyword("fun") | @p.Symbol('(') => true
          _ => false
        }) {
    term = App(term, atom(p, env, next, free))
  }
  term
}

///|
fn atom(
  p : @p.Parser,
  env : Map[String, Variable],
  next : Ref[Int32],
  free : Map[String, Variable],
) -> Term raise @base.Fault {
  match p.peek() {
    @p.Ident(name) => {
      ignore(p.bump())
      let v = match env.get(name) {
        Some(v) => v
        None =>
          match free.get(name) {
            Some(v) => v
            None => {
              let v = fresh(next, name)
              free[name] = v
              v
            }
          }
      }
      Var(v)
    }
    @p.Symbol('(') => {
      let g = p.group('(')
      let t = application(g, env, next, free)
      g.finish()
      t
    }
    @p.Keyword("fun") => {
      ignore(p.bump())
      let env = env.copy()
      let binders = []
      while (match p.peek() {
              @p.Ident(_) => true
              _ => false
            }) {
        let name = p.ident()
        let v = fresh(next, name)
        env[name] = v
        binders.push(v)
      }
      @base.require(!binders.is_empty(), "Expected binders followed by '.'")
      p.expect('.')
      let mut t = application(p, env, next, free)
      binders.rev_in_place()
      for v in binders {
        t = Abs(v, t)
      }
      t
    }
    _ => p.error("Expected a lambda term")
  }
}

///|
pub fn parse_code(text : String) -> Term raise @base.Fault {
  let p = @p.create(text)
  let t = read(p)
  p.finish()
  t
}

///|
pub fn parse_input(text : String) -> Array[Term] raise @base.Fault {
  let p = @p.create(text)
  let terms = []
  while !p.eof() {
    terms.push(read(p))
    if p.eof() {
      break
    }
    p.expect(',')
    @base.require(!p.eof(), "Expected lambda term after comma")
  }
  terms
}

///|
pub fn max_id(t : Term) -> Int32 {
  match t {
    Var(v) => v.id
    Abs(v, t) => Int32::max(v.id, max_id(t))
    App(a, b) => Int32::max(max_id(a), max_id(b))
  }
}

///|
pub fn freshen(t : Term, next : Ref[Int32]) -> Term {
  let vars : Map[Int32, Variable] = Map([])
  fn variable(v : Variable) -> Variable {
    match vars.get(v.id) {
      Some(v) => v
      None => {
        let n = fresh(next, v.name)
        vars[v.id] = n
        n
      }
    }
  }
  fn visit(t : Term) -> Term {
    match t {
      Var(v) => Var(variable(v))
      Abs(v, t) => {
        let v = variable(v)
        Abs(v, visit(t))
      }
      App(a, b) => {
        let a = visit(a)
        App(a, visit(b))
      }
    }
  }
  visit(t)
}

///|
pub fn freshen_bound(t : Term, next : Ref[Int32]) -> Term {
  fn visit(t : Term, env : Map[Int32, Variable]) -> Term {
    match t {
      Var(v) => Var(env.get(v.id).unwrap_or(v))
      Abs(v, t) => {
        let n = fresh(next, v.name)
        let env = env.copy()
        env[v.id] = n
        Abs(n, visit(t, env))
      }
      App(a, b) => {
        let a = visit(a, env)
        App(a, visit(b, env))
      }
    }
  }
  visit(t, Map([]))
}

///|
fn make(code : String, input : String) -> Term raise @base.Fault {
  let mut t = parse_code(code)
  let next : Ref[Int32] = { val: max_id(t) + 1, }
  for arg in parse_input(input) {
    t = App(t, freshen(arg, next))
  }
  t
}

///|
fn substitute(t : Term, id : Int32, replacement : Term) -> Term {
  match t {
    Var(v) => if v.id == id { replacement } else { t }
    Abs(v, body) =>
      if v.id == id {
        t
      } else {
        Abs(v, substitute(body, id, replacement))
      }
    App(a, b) =>
      App(substitute(a, id, replacement), substitute(b, id, replacement))
  }
}

///|
fn redexes(t : Term) -> Array[Term] {
  match t {
    Var(_) => []
    Abs(_, b) => redexes(b)
    App(a, b) => {
      let result = match a {
        Abs(_, _) => [t]
        _ => []
      }
      result.append(redexes(a))
      result.append(redexes(b))
      result
    }
  }
}

///|
fn step(
  t : Term,
  input : String,
) -> @runtime.Transition[Term] raise @base.Fault {
  let index : Ref[Int32] = { val: @base.index(@base.nat(input)), }
  let done : Ref[Bool] = { val: false, }
  let next : Ref[Int32] = { val: max_id(t) + 1, }
  fn visit(t : Term) -> Term {
    if done.val {
      return t
    }
    match t {
      Var(_) => t
      Abs(v, b) => Abs(v, visit(b))
      App(a, b) => {
        match a {
          Abs(_, _) => {
            if index.val == 0 {
              done.val = true
              match freshen_bound(a, next) {
                Abs(v, body) => return substitute(body, v.id, b)
                _ => ()
              }
            }
            index.val -= 1
          }
          _ => ()
        }
        let a = visit(a)
        App(a, visit(b))
      }
    }
  }
  let t = visit(t)
  @base.require(done.val, "No redex found at the given index")
  if redexes(t).is_empty() {
    @runtime.Halt(print_code(t))
  } else {
    @runtime.Continue(t, "")
  }
}

///|
pub fn print_code(t : Term) -> String {
  let names : Map[Int32, String] = Map([])
  let used : Map[String, Bool] = Map([])
  fn name(v : Variable) -> String {
    match names.get(v.id) {
      Some(n) => n
      None => {
        let mut n = v.name
        let mut i = 0
        while used.contains(n) {
          n = v.name + "_" + i.to_string()
          i += 1
        }
        names[v.id] = n
        used[n] = true
        n
      }
    }
  }
  fn free(t : Term, bound : Array[Int32]) -> Unit {
    match t {
      Var(v) => if !bound.contains(v.id) { ignore(name(v)) }
      Abs(v, b) => {
        let bound = bound.copy()
        bound.push(v.id)
        free(b, bound)
      }
      App(a, b) => {
        free(a, bound)
        free(b, bound)
      }
    }
  }
  free(t, [])
  fn print(t : Term) -> String {
    match t {
      Var(v) => name(v)
      Abs(v, b) => {
        let n = name(v)
        "\\fun " + n + ". " + print(b)
      }
      App(a, b) => {
        fn operand(t : Term) -> String {
          match t {
            Abs(_, _) => "(" + print(t) + ")"
            _ => print(t)
          }
        }
        let a = operand(a)
        "(" + a + " " + operand(b) + ")"
      }
    }
  }
  print(t)
}

///|
fn encode(t : Term) -> Json {
  fn variable(v : Variable) -> Json {
    @base.object([("id", @base.jint(v.id)), ("name", @base.jstr(v.name))])
  }
  match t {
    Var(v) => @base.tag("var", [variable(v)])
    Abs(v, b) => @base.tag("abs", [variable(v), encode(b)])
    App(a, b) => @base.tag("app", [encode(a), encode(b)])
  }
}

///|
fn decode(j : Json) -> Term raise @base.Fault {
  let vars : Map[Int32, Variable] = Map([])
  let binders : Array[Int32] = []
  fn collect(j : Json) -> Unit raise @base.Fault {
    match @base.tag_name(j) {
      "abs" => {
        binders.push(@base.int(@base.field(@base.tag_arg(j, 0), "id")))
        collect(@base.tag_arg(j, 1))
      }
      "app" => {
        collect(@base.tag_arg(j, 0))
        collect(@base.tag_arg(j, 1))
      }
      _ => ()
    }
  }
  collect(j)
  fn variable(j : Json) -> Variable raise @base.Fault {
    let id = @base.int(@base.field(j, "id"))
    @base.require(id >= 0 && id < 2147483647, "Invalid variable ID")
    let name = @base.identifier(@base.string(@base.field(j, "name")))
    match vars.get(id) {
      Some(v) => {
        @base.require(v.name == name, "Conflicting variable names")
        v
      }
      None => {
        let v = { id, name, }
        vars[id] = v
        v
      }
    }
  }
  fn visit(j : Json, env : Array[Int32]) -> Term raise @base.Fault {
    let a = @base.array(j)
    match @base.tag_name(j) {
      "var" => {
        @base.require(a.length() == 2, "Invalid lambda snapshot")
        let v = variable(a[1])
        @base.require(
          !binders.contains(v.id) || env.contains(v.id),
          "Bound variable occurs outside its scope",
        )
        Var(v)
      }
      "abs" => {
        @base.require(a.length() == 3, "Invalid lambda snapshot")
        let v = variable(a[1])
        @base.require(!env.contains(v.id), "Repeated binder ID")
        let env = env.copy()
        env.push(v.id)
        Abs(v, visit(a[2], env))
      }
      "app" => {
        @base.require(a.length() == 3, "Invalid lambda snapshot")
        let a = visit(a[1], env)
        App(a, visit(@base.tag_arg(j, 1), env))
      }
      _ => @base.error("Invalid lambda snapshot")
    }
  }
  visit(j, [])
}

///|
pub fn model() -> @runtime.Model {
  @runtime.model(make, step, encode, decode, t => {
    @base.jarray([
      @base.titled("term", print_code(t)),
      @base.table(
        "redexes",
        ["index", "term"],
        redexes(t).mapi((i, t) => [i.to_string(), print_code(t)]),
      ),
    ])
  })
}
