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
priv struct State {
  code : Code
  stmt : Stmt
  global_env : @a.Env
  local_env : @a.Env
  call_stack : Array[Frame]
}

///|
fn group(p : @p.Parser) -> Stmt raise @base.Fault {
  let g = p.group('[')
  let s = statement(g)
  g.finish()
  s
}

///|
fn single(p : @p.Parser) -> Stmt raise @base.Fault {
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
  } else if p.peek() == @p.Symbol('[') {
    group(p)
  } else {
    let n = p.ident()
    p.expect_text(":=")
    Assign(n, @a.parse_aexp(p))
  }
}

///|
fn statement(p : @p.Parser) -> Stmt raise @base.Fault {
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
pub fn parse_code(text : String) -> Code raise @base.Fault {
  let p = @p.create(text)
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
  @base.require(!procs.is_empty(), "At least one procedure is required")
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
      @base.join(args, ", ") +
      ") -> " +
      @base.join(rets, ", ")
    Return(vs) => "\\return " + @base.join(vs, ", ")
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
fn validate(code : Code) -> Procedure raise @base.Fault {
  @data.unique(code.statics, "Duplicate static variable: ")
  @data.unique(code.procs.map(p => p.name), "Duplicate procedure name: ")
  for p in code.procs {
    let seen : Map[String, Bool] = Map([])
    for v in p.params {
      @base.require(
        !code.statics.contains(v),
        "Name conflict in " +
        p.name +
        ": parameter " +
        v +
        " conflicts with static",
      )
      @base.require(
        !seen.contains(v),
        "Duplicate parameter " + v + " in " + p.name,
      )
      seen[v] = true
    }
    for v in p.locals {
      @base.require(
        !code.statics.contains(v),
        "Name conflict in " + p.name + ": local " + v + " conflicts with static",
      )
      @base.require(
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
    None => @base.error("Procedure 'main' is required")
  }
  @base.require(main.params.is_empty(), "main must have no parameters")
  main
}

///|
fn make(text : String, input : String) -> State raise @base.Fault {
  let code = parse_code(text)
  let main = validate(code)
  let global_env = @data.env_sorted(@a.parse_env(input))
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
    local_env: @data.env_sorted(main.locals.map(n => (n, @base.zero()))),
    call_stack: [],
  }
}

///|
fn lookup(s : State, n : String) -> @base.Nat raise @base.Fault {
  match @data.env_get(s.local_env, n) {
    Some(v) => v
    None => @data.env_lookup(s.global_env, n)
  }
}

///|
fn assign(s : State, n : String, value : @base.Nat) -> State raise @base.Fault {
  if @data.env_get(s.local_env, n) is Some(_) {
    { ..s, local_env: @data.env_set(s.local_env, n, value), }
  } else {
    ignore(@data.env_lookup(s.global_env, n))
    { ..s, global_env: @data.env_set(s.global_env, n, value), }
  }
}

///|
fn call(
  s : State,
  n : String,
  args : Array[String],
  rets : Array[String],
  continuation : Stmt,
) -> (Stmt, State) raise @base.Fault {
  let p = match s.code.procs.iter().find_first(p => p.name == n) {
    Some(p) => p
    None => @base.error("Unknown procedure: " + n)
  }
  @base.require(
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
    local_env = @data.env_set(local_env, n, @base.zero())
  }
  let call_stack = [
    { stmt: continuation, local_env: s.local_env, ret_vars: rets, },
  ]
  call_stack.append(s.call_stack)
  (
    with_return(p.body),
    { ..s, local_env: @data.env_sorted(local_env), call_stack, },
  )
}

///|
fn small(s : State, stmt : Stmt) -> (Stmt, State) raise @base.Fault {
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
        @base.require(
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
  _input : String,
) -> @runtime.Transition[State] raise @base.Fault {
  match s.stmt {
    Return(_) if s.call_stack.is_empty() =>
      @runtime.Halt(@a.print_env(s.global_env))
    _ => {
      let (stmt, next) = small(s, s.stmt)
      @runtime.Continue({ ..next, stmt, }, st(s.stmt))
    }
  }
}

///|
fn sj(s : Stmt) -> Json {
  match s {
    Nop => @base.tag("Nop", [])
    Seq(a, b) => @base.tag("Seq", [sj(a), sj(b)])
    Assign(n, a) => @base.tag("Assign", [@base.jstr(n), @a.aexp_json(a)])
    If(c, b) => @base.tag("If", [@a.bexp_json(c), sj(b)])
    While(c, b) => @base.tag("While", [@a.bexp_json(c), sj(b)])
    Call(n, args, rets) =>
      @base.tag("Call", [
        @base.jstr(n),
        @a.names_json(args),
        @a.names_json(rets),
      ])
    Return(vs) => @base.tag("Return", [@a.names_json(vs)])
  }
}

///|
fn sd(j : Json) -> Stmt raise @base.Fault {
  fn a(i : Int32) -> Json raise @base.Fault {
    @base.tag_arg(j, i)
  }
  match @base.tag_name(j) {
    "Nop" => Nop
    "Seq" => Seq(sd(a(0)), sd(a(1)))
    "Assign" =>
      Assign(@base.identifier(@base.string(a(0))), @a.decode_aexp(a(1)))
    "If" => If(@a.decode_bexp(a(0)), sd(a(1)))
    "While" => While(@a.decode_bexp(a(0)), sd(a(1)))
    "Call" =>
      Call(
        @base.identifier(@base.string(a(0))),
        @a.decode_names(a(1)),
        @a.decode_names(a(2)),
      )
    "Return" => Return(@a.decode_names(a(0)))
    _ => @base.error("Invalid procedure statement")
  }
}

///|
fn code_json(code : Code) -> Json {
  @base.object([
    ("statics", @a.names_json(code.statics)),
    (
      "procs",
      @base.jarray(
        code.procs.map(p => {
          @base.object([
            ("name", @base.jstr(p.name)),
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
fn decode_code(j : Json) -> Code raise @base.Fault {
  {
    statics: @a.decode_names(@base.field(j, "statics")),
    procs: @base.array(@base.field(j, "procs")).map(j => {
      name: @base.identifier(@base.string(@base.field(j, "name"))),
      params: @a.decode_names(@base.field(j, "params")),
      locals: @a.decode_names(@base.field(j, "locals")),
      body: sd(@base.field(j, "body")),
    }),
  }
}

///|
fn encode(s : State) -> Json {
  @base.object([
    ("code", code_json(s.code)),
    ("stmt", sj(s.stmt)),
    ("global_env", @a.env_json(s.global_env)),
    ("local_env", @a.env_json(s.local_env)),
    (
      "call_stack",
      @base.jarray(
        s.call_stack.map(f => {
          @base.object([
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
fn decode(j : Json) -> State raise @base.Fault {
  {
    code: decode_code(@base.field(j, "code")),
    stmt: sd(@base.field(j, "stmt")),
    global_env: @a.decode_env(@base.field(j, "global_env")),
    local_env: @a.decode_env(@base.field(j, "local_env")),
    call_stack: @base.array(@base.field(j, "call_stack")).map(j => {
      stmt: sd(@base.field(j, "stmt")),
      local_env: @a.decode_env(@base.field(j, "local_env")),
      ret_vars: @a.decode_names(@base.field(j, "ret_vars")),
    }),
  }
}

///|
pub fn model() -> @runtime.Model {
  @runtime.model(make, step, encode, decode, s => {
    @base.jarray([
      @base.titled("stmt", st(s.stmt)),
      @base.titled(
        "global env",
        @a.print_env(@data.env_sorted(s.global_env), trailing_newline=false),
      ),
      @base.titled(
        "local env",
        @a.print_env(@data.env_sorted(s.local_env), trailing_newline=false),
      ),
      @base.titled("call stack depth", s.call_stack.length().to_string()),
    ])
  })
}

///|
test "procedure call restores locals and handles return values" {
  let m = model()
  m.make(
    "\\static x; inc(n)[\\local tmp tmp := n + 1; \\return tmp] main()[\\local a a := 5; \\call inc(a) -> x; \\return]",
    "x=0",
  )
  let mut output = None
  for _ in 0..<50 {
    let saved = m.snapshot()
    let result = m.step("ignored")
    m.restore(saved)
    assert_eq(m.step("ignored"), result)
    let j = @base.parse_json(result)
    if @base.string(@base.field(j, "kind")) == "halt" {
      output = Some(@base.string(@base.field(j, "foutput")))
      break
    }
  }
  assert_eq(output, Some("x = 6\n"))
  assert_true(
    try {
      m.make("\\static x; main()[\\local x \\return]", "x=0")
      false
    } catch {
      _ => true
    },
  )
}
