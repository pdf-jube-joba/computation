///|
pub(all) enum Stmt {
  Nop
  Seq(Stmt, Stmt)
  Assign(String, @a.Aexp)
  If(@a.Bexp, Stmt)
  While(@a.Bexp, Stmt)
  Call(String, Array[String], Array[String])
  Return(Array[String])
}

///|
pub(all) struct Procedure {
  name : String
  params : Array[String]
  locals : Array[String]
  body : Stmt
}

///|
pub(all) struct Code {
  statics : Array[String]
  procs : Array[Procedure]
}

///|
priv struct Frame {
  stmt : Stmt
  local_env : @a.Env
  ret_vars : Array[String]
}

///|
struct State {
  code : Code
  stmt : Stmt
  global_env : @a.Env
  local_env : @a.Env
  call_stack : Array[Frame]
}

///|
fn group(p : @utils.Parser) -> Stmt raise @utils.Fault {
  let g = p.group('[')
  let s = statement(g)
  g.finish()
  s
}

///|
fn single(p : @utils.Parser) -> Stmt raise @utils.Fault {
  if p.eat_keyword("Nop") {
    Nop
  } else if p.eat_keyword("if") {
    let c = @a.parse_bexp(p)
    If(c, group(p))
  } else if p.eat_keyword("while") {
    let c = @a.parse_bexp(p)
    While(c, group(p))
  } else if p.eat_keyword("call") {
    let n = p.ident()
    let g = p.group('(')
    let args = @a.names(g)
    g.finish()
    p.expect_text("->")
    Call(n, args, @a.names(p))
  } else if p.eat_keyword("return") {
    Return(@a.names(p))
  } else if p.peek() == @utils.Symbol('[') {
    group(p)
  } else {
    let n = p.ident()
    p.expect_text(":=")
    Assign(n, @a.parse_aexp(p))
  }
}

///|
fn statement(p : @utils.Parser) -> Stmt raise @utils.Fault {
  let statements = [single(p)]
  while p.eat(';') {
    if p.eof() {
      break
    }
    statements.push(single(p))
  }
  let mut s = statements[statements.length() - 1]
  for i = statements.length() - 2; i >= 0; i = i - 1 {
    s = Seq(statements[i], s)
  }
  s
}

///|
pub fn parse_code(text : String) -> Code raise @utils.Fault {
  let p = @utils.create(text)
  p.expect_keyword("static")
  let statics = @a.names(p)
  p.expect(';')
  let procs = []
  while !p.eof() {
    let name = p.ident()
    let g = p.group('(')
    let params = @a.names(g)
    g.finish()
    let g = p.group('[')
    g.expect_keyword("local")
    let locals = @a.names(g)
    let body = statement(g)
    g.finish()
    procs.push({ name, params, locals, body, })
  }
  @utils.require(!procs.is_empty(), "At least one procedure is required")
  { statics, procs, }
}

///|
fn st(s : Stmt) -> String {
  match s {
    Nop => "\\Nop"
    Seq(a, b) => st(a) + "; " + st(b)
    Assign(n, a) => n + " := " + @a.aexp_text(a)
    If(c, b) => "\\if " + @a.bexp_text(c) + " [" + st(b) + "]"
    While(c, b) => "\\while " + @a.bexp_text(c) + " [" + st(b) + "]"
    Call(n, args, rets) =>
      "\\call " +
      n +
      "(" +
      @utils.join(args, ", ") +
      ") -> " +
      @utils.join(rets, ", ")
    Return(vs) => "\\return " + @utils.join(vs, ", ")
  }
}

///|
pub fn with_return(s : Stmt) -> Stmt {
  fn ends(s : Stmt) -> Bool {
    match s {
      Return(_) => true
      Seq(_, b) => ends(b)
      _ => false
    }
  }
  if ends(s) {
    s
  } else {
    Seq(s, Return([]))
  }
}

///|
fn validate(code : Code) -> Procedure raise @utils.Fault {
  @utils.unique(code.statics, "Duplicate static variable: ")
  @utils.unique(code.procs.map(p => p.name), "Duplicate procedure name: ")
  for p in code.procs {
    let seen : Map[String, Bool] = Map([])
    for v in p.params {
      @utils.require(
        !code.statics.contains(v),
        "Name conflict in " +
        p.name +
        ": parameter " +
        v +
        " conflicts with static",
      )
      @utils.require(
        !seen.contains(v),
        "Duplicate parameter " + v + " in " + p.name,
      )
      seen[v] = true
    }
    for v in p.locals {
      @utils.require(
        !code.statics.contains(v),
        "Name conflict in " + p.name + ": local " + v + " conflicts with static",
      )
      @utils.require(
        !seen.contains(v),
        "Name conflict in " +
        p.name +
        ": local " +
        v +
        " conflicts with parameter/local",
      )
      seen[v] = true
    }
  }
  let main = match code.procs.iter().find_first(p => p.name == "main") {
    Some(p) => p
    None => @utils.error("Procedure 'main' is required")
  }
  @utils.require(main.params.is_empty(), "main must have no parameters")
  main
}

///|
fn make(code : Code, input : @a.Environment) -> State raise @utils.Fault {
  let main = validate(code)
  let global_env = @utils.env_sorted(input.values)
  @a.validate_env(
    code.statics,
    global_env,
    duplicate="Duplicate static variable: ",
    extra="AInput contains undeclared static variable: ",
    missing="AInput missing static variable: ",
  )
  {
    code,
    stmt: with_return(main.body),
    global_env,
    local_env: @utils.env_sorted(main.locals.map(n => (n, @utils.zero()))),
    call_stack: [],
  }
}

///|
fn lookup(s : State, n : String) -> @utils.Nat raise @utils.Fault {
  match @utils.env_get(s.local_env, n) {
    Some(v) => v
    None => @utils.env_lookup(s.global_env, n)
  }
}

///|
fn assign(
  s : State,
  n : String,
  value : @utils.Nat,
) -> State raise @utils.Fault {
  if @utils.env_get(s.local_env, n) is Some(_) {
    { ..s, local_env: @utils.env_set(s.local_env, n, value), }
  } else {
    ignore(@utils.env_lookup(s.global_env, n))
    { ..s, global_env: @utils.env_set(s.global_env, n, value), }
  }
}

///|
fn call(
  s : State,
  n : String,
  args : Array[String],
  rets : Array[String],
  continuation : Stmt,
) -> (Stmt, State) raise @utils.Fault {
  let p = match s.code.procs.iter().find_first(p => p.name == n) {
    Some(p) => p
    None => @utils.error("Unknown procedure: " + n)
  }
  @utils.require(
    args.length() == p.params.length(),
    "Argument count mismatch for " +
    n +
    ": expected " +
    p.params.length().to_string() +
    ", got " +
    args.length().to_string(),
  )
  let values = args.map(n => lookup(s, n))
  let mut local_env = p.params.mapi((i, n) => (n, values[i]))
  for n in p.locals {
    local_env = @utils.env_set(local_env, n, @utils.zero())
  }
  let call_stack = [
    { stmt: continuation, local_env: s.local_env, ret_vars: rets, },
  ]
  call_stack.append(s.call_stack)
  (
    with_return(p.body),
    { ..s, local_env: @utils.env_sorted(local_env), call_stack, },
  )
}

///|
fn small(s : State, stmt : Stmt) -> (Stmt, State) raise @utils.Fault {
  match stmt {
    Nop => (Nop, s)
    Assign(n, a) => (Nop, assign(s, n, @a.eval_aexp(a, n => lookup(s, n))))
    If(c, b) => (if @a.eval_bexp(c, n => lookup(s, n)) { b } else { Nop }, s)
    While(c, b) => (If(c, Seq(b, stmt)), s)
    Seq(Nop, b) => (b, s)
    Seq(Return(vs), _) => (Return(vs), s)
    Seq(Call(n, args, rets), b) => call(s, n, args, rets, b)
    Seq(a, b) => {
      let (a, s) = small(s, a)
      (Seq(a, b), s)
    }
    Call(n, args, rets) => call(s, n, args, rets, Nop)
    Return(vs) =>
      if s.call_stack.is_empty() {
        (stmt, s)
      } else {
        let frame = s.call_stack[0]
        @utils.require(
          vs.length() == frame.ret_vars.length(),
          "Return count mismatch: expected " +
          frame.ret_vars.length().to_string() +
          ", got " +
          vs.length().to_string(),
        )
        let values = vs.map(n => lookup(s, n))
        let mut s = {
          ..s,
          call_stack: s.call_stack[1:].to_owned(),
          local_env: frame.local_env,
        }
        for i, n in frame.ret_vars {
          s = assign(s, n, values[i])
        }
        (frame.stmt, s)
      }
  }
}

///|
fn step(
  s : State,
  __input : Rinput,
) -> @definition.Transition[State, Routput, Foutput] raise @utils.Fault {
  match s.stmt {
    Return(_) if s.call_stack.is_empty() =>
      @definition.Halt(@a.Environment::{ values: s.global_env, })
    _ => {
      let (stmt, next) = small(s, s.stmt)
      @definition.Continue({ ..next, stmt, }, st(s.stmt))
    }
  }
}

///|
fn sj(s : Stmt) -> Json {
  match s {
    Nop => @utils.tag("Nop", [])
    Seq(a, b) => @utils.tag("Seq", [sj(a), sj(b)])
    Assign(n, a) => @utils.tag("Assign", [@utils.jstr(n), @a.aexp_json(a)])
    If(c, b) => @utils.tag("If", [@a.bexp_json(c), sj(b)])
    While(c, b) => @utils.tag("While", [@a.bexp_json(c), sj(b)])
    Call(n, args, rets) =>
      @utils.tag("Call", [
        @utils.jstr(n),
        @a.names_json(args),
        @a.names_json(rets),
      ])
    Return(vs) => @utils.tag("Return", [@a.names_json(vs)])
  }
}

///|
fn sd(j : Json) -> Stmt raise @utils.Fault {
  fn a(i : Int32) -> Json raise @utils.Fault {
    @utils.tag_arg(j, i)
  }
  match @utils.tag_name(j) {
    "Nop" => Nop
    "Seq" => Seq(sd(a(0)), sd(a(1)))
    "Assign" =>
      Assign(@utils.identifier(@utils.string(a(0))), @a.decode_aexp(a(1)))
    "If" => If(@a.decode_bexp(a(0)), sd(a(1)))
    "While" => While(@a.decode_bexp(a(0)), sd(a(1)))
    "Call" =>
      Call(
        @utils.identifier(@utils.string(a(0))),
        @a.decode_names(a(1)),
        @a.decode_names(a(2)),
      )
    "Return" => Return(@a.decode_names(a(0)))
    _ => @utils.error("Invalid procedure statement")
  }
}

///|
fn code_json(code : Code) -> Json {
  @utils.object([
    ("statics", @a.names_json(code.statics)),
    (
      "procs",
      @utils.jarray(
        code.procs.map(p => {
          @utils.object([
            ("name", @utils.jstr(p.name)),
            ("params", @a.names_json(p.params)),
            ("locals", @a.names_json(p.locals)),
            ("body", sj(p.body)),
          ])
        }),
      ),
    ),
  ])
}

///|
fn decode_code(j : Json) -> Code raise @utils.Fault {
  {
    statics: @a.decode_names(@utils.field(j, "statics")),
    procs: @utils.array(@utils.field(j, "procs")).map(j => {
      name: @utils.identifier(@utils.string(@utils.field(j, "name"))),
      params: @a.decode_names(@utils.field(j, "params")),
      locals: @a.decode_names(@utils.field(j, "locals")),
      body: sd(@utils.field(j, "body")),
    }),
  }
}

///|
fn encode(s : State) -> Json {
  @utils.object([
    ("code", code_json(s.code)),
    ("stmt", sj(s.stmt)),
    ("global_env", @a.env_json(s.global_env)),
    ("local_env", @a.env_json(s.local_env)),
    (
      "call_stack",
      @utils.jarray(
        s.call_stack.map(f => {
          @utils.object([
            ("stmt", sj(f.stmt)),
            ("local_env", @a.env_json(f.local_env)),
            ("ret_vars", @a.names_json(f.ret_vars)),
          ])
        }),
      ),
    ),
  ])
}

///|
fn decode(j : Json) -> State raise @utils.Fault {
  {
    code: decode_code(@utils.field(j, "code")),
    stmt: sd(@utils.field(j, "stmt")),
    global_env: @a.decode_env(@utils.field(j, "global_env")),
    local_env: @a.decode_env(@utils.field(j, "local_env")),
    call_stack: @utils.array(@utils.field(j, "call_stack")).map(j => {
      stmt: sd(@utils.field(j, "stmt")),
      local_env: @a.decode_env(@utils.field(j, "local_env")),
      ret_vars: @a.decode_names(@utils.field(j, "ret_vars")),
    }),
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
