///|
pub(all) enum Stmt {
  Nop
  Seq(Stmt, Stmt)
  Assign(String, @a.Aexp)
  If(@a.Bexp, Stmt)
  Break(String, String)
  Continue(String)
  Loop(String, Stmt, String)
}

///|
pub(all) struct Code {
  statics : Array[String]
  body : Stmt
}

///|
priv enum Control {
  Normal
  BreakResult(String, @base.Nat)
  ContinueResult(String)
}

///|
priv enum Exec {
  Stmt(Stmt)
  Result(Control)
}

///|
priv enum Frame {
  LoopFrame(String, Stmt, String)
  SeqFrame(Stmt)
}

///|
priv struct State {
  code : Code
  env : @a.Env
  exec : Exec
  stack : Array[Frame]
}

///|
fn label(p : @p.Parser) -> String raise @base.Fault {
  p.expect(':')
  p.ident()
}

///|
fn block(p : @p.Parser, opening : Char) -> Stmt raise @base.Fault {
  let g = p.group(opening)
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
    If(c, block(p, '{'))
  } else if p.eat_keyword("break") {
    let l = label(p)
    Break(l, p.ident())
  } else if p.eat_keyword("continue") {
    Continue(label(p))
  } else if p.eat_keyword("loop") {
    let l = label(p)
    let b = block(p, '(')
    p.expect_text("->")
    Loop(l, b, p.ident())
  } else if p.peek() == @p.Symbol('{') {
    block(p, '{')
  } else {
    let n = p.ident()
    p.expect_text(":=")
    Assign(n, @a.parse_aexp(p))
  }
}

///|
fn statement(p : @p.Parser) -> Stmt raise @base.Fault {
  let mut s = single(p)
  while p.eat(';') {
    if p.eof() {
      break
    }
    s = Seq(s, single(p))
  }
  s
}

///|
fn make(code : String, input : String) -> State raise @base.Fault {
  let p = @p.create(code)
  p.expect_keyword("static")
  let statics = @a.names(p)
  p.expect(';')
  let body = statement(p)
  p.finish()
  let code = { statics, body, }
  let env = @a.parse_env(input)
  @a.validate_env(
    statics,
    env,
    duplicate="duplicate static variable: ",
    extra="ainput contains undeclared static variable: ",
    missing="ainput missing static variable: ",
  )
  { code, env, exec: Stmt(body), stack: [], }
}

///|
fn st(s : Stmt) -> String {
  match s {
    Nop => "\\Nop"
    Seq(a, b) => st(a) + "; " + st(b)
    Assign(n, a) => n + " := " + @a.aexp_text(a)
    If(c, b) => "\\if " + @a.bexp_text(c) + " { " + st(b) + " }"
    Break(l, n) => "\\break :" + l + " " + n
    Continue(l) => "\\continue :" + l
    Loop(l, b, n) => "\\loop :" + l + " (" + st(b) + ") -> " + n
  }
}

///|
fn et(e : Exec) -> String {
  match e {
    Stmt(s) => st(s)
    Result(Normal) => "()"
    Result(BreakResult(l, n)) => "break " + l + " " + n.to_string()
    Result(ContinueResult(l)) => "continue " + l
  }
}

///|
fn push(f : Frame, stack : Array[Frame]) -> Array[Frame] {
  let result = [f]
  result.append(stack)
  result
}

///|
fn step(
  s : State,
  input : String,
) -> @runtime.Transition[State] raise @base.Fault {
  @base.unit_input(input)
  let output = et(s.exec)
  let next = match s.exec {
    Stmt(Nop) => { ..s, exec: Result(Normal), }
    Stmt(Seq(a, b)) =>
      { ..s, exec: Stmt(a), stack: push(SeqFrame(b), s.stack), }
    Stmt(Assign(n, a)) =>
      {
        ..s,
        exec: Result(Normal),
        env: @data.env_set(s.env, n, @a.eval_aexp(a, n => @a.env_get(s.env, n))),
      }
    Stmt(If(c, b)) =>
      {
        ..s,
        exec: if @a.eval_bexp(c, n => @a.env_get(s.env, n)) {
          Stmt(b)
        } else {
          Result(Normal)
        },
      }
    Stmt(Break(l, n)) =>
      { ..s, exec: Result(BreakResult(l, @a.env_get(s.env, n))), }
    Stmt(Continue(l)) => { ..s, exec: Result(ContinueResult(l)), }
    Stmt(Loop(l, b, n)) =>
      { ..s, exec: Result(Normal), stack: push(LoopFrame(l, b, n), s.stack), }
    Result(control) =>
      if s.stack.is_empty() {
        match control {
          Normal => s
          BreakResult(l, _) => @base.error("break outside loop: " + l)
          ContinueResult(l) => @base.error("continue outside loop: " + l)
        }
      } else {
        let stack = s.stack[1:].to_owned()
        match s.stack[0] {
          SeqFrame(b) =>
            {
              ..s,
              stack,
              exec: match control {
                Normal => Stmt(b)
                _ => Result(control)
              },
            }
          LoopFrame(l, b, n) =>
            match control {
              Normal => { ..s, exec: Stmt(b), }
              ContinueResult(got) if got == l => { ..s, exec: Result(Normal), }
              BreakResult(got, value) if got == l =>
                {
                  ..s,
                  stack,
                  exec: Result(Normal),
                  env: @data.env_set(s.env, n, value),
                }
              _ => { ..s, stack, exec: Result(control), }
            }
        }
      }
  }
  match (s.exec, next.exec) {
    (Result(_), Result(Normal)) if next.stack.is_empty() =>
      @runtime.Halt(@a.print_env(next.env))
    _ => @runtime.Continue(next, output)
  }
}

///|
fn sj(s : Stmt) -> Json {
  match s {
    Nop => @base.tag("Nop", [])
    Seq(a, b) => @base.tag("Seq", [sj(a), sj(b)])
    Assign(n, a) => @base.tag("Assign", [@base.jstr(n), @a.aexp_json(a)])
    If(c, b) => @base.tag("If", [@a.bexp_json(c), sj(b)])
    Break(l, n) => @base.tag("Break", [@base.jstr(l), @base.jstr(n)])
    Continue(l) => @base.tag("Continue", [@base.jstr(l)])
    Loop(l, b, n) => @base.tag("Loop", [@base.jstr(l), sj(b), @base.jstr(n)])
  }
}

///|
fn sd(j : Json) -> Stmt raise @base.Fault {
  fn a(i : Int32) -> Json raise @base.Fault {
    @base.tag_arg(j, i)
  }
  fn n(i : Int32) -> String raise @base.Fault {
    @base.identifier(@base.string(a(i)))
  }
  match @base.tag_name(j) {
    "Nop" => Nop
    "Seq" => Seq(sd(a(0)), sd(a(1)))
    "Assign" => Assign(n(0), @a.decode_aexp(a(1)))
    "If" => If(@a.decode_bexp(a(0)), sd(a(1)))
    "Break" => Break(n(0), n(1))
    "Continue" => Continue(n(0))
    "Loop" => Loop(n(0), sd(a(1)), n(2))
    _ => @base.error("Invalid control statement")
  }
}

///|
fn cj(c : Control) -> Json {
  match c {
    Normal => @base.tag("Normal", [])
    BreakResult(l, n) =>
      @base.tag("Break_result", [@base.jstr(l), @base.jstr(n.to_string())])
    ContinueResult(l) => @base.tag("Continue_result", [@base.jstr(l)])
  }
}

///|
fn cd(j : Json) -> Control raise @base.Fault {
  match @base.tag_name(j) {
    "Normal" => Normal
    "Break_result" =>
      BreakResult(
        @base.identifier(@base.string(@base.tag_arg(j, 0))),
        @base.json_nat(@base.tag_arg(j, 1)),
      )
    "Continue_result" =>
      ContinueResult(@base.identifier(@base.string(@base.tag_arg(j, 0))))
    _ => @base.error("Invalid control result")
  }
}

///|
fn ej(e : Exec) -> Json {
  match e {
    Stmt(s) => @base.tag("Stmt", [sj(s)])
    Result(c) => @base.tag("Result", [cj(c)])
  }
}

///|
fn ed(j : Json) -> Exec raise @base.Fault {
  match @base.tag_name(j) {
    "Stmt" => Stmt(sd(@base.tag_arg(j, 0)))
    "Result" => Result(cd(@base.tag_arg(j, 0)))
    _ => @base.error("Invalid execution state")
  }
}

///|
fn fj(f : Frame) -> Json {
  match f {
    SeqFrame(s) => @base.tag("Seq_frame", [sj(s)])
    LoopFrame(l, b, n) =>
      @base.tag("Loop_frame", [@base.jstr(l), sj(b), @base.jstr(n)])
  }
}

///|
fn fd(j : Json) -> Frame raise @base.Fault {
  match @base.tag_name(j) {
    "Seq_frame" => SeqFrame(sd(@base.tag_arg(j, 0)))
    "Loop_frame" =>
      LoopFrame(
        @base.identifier(@base.string(@base.tag_arg(j, 0))),
        sd(@base.tag_arg(j, 1)),
        @base.identifier(@base.string(@base.tag_arg(j, 2))),
      )
    _ => @base.error("Invalid control frame")
  }
}

///|
fn encode(s : State) -> Json {
  @base.object([
    (
      "code",
      @base.object([
        ("statics", @a.names_json(s.code.statics)),
        ("body", sj(s.code.body)),
      ]),
    ),
    ("env", @a.env_json(s.env)),
    ("exec", ej(s.exec)),
    ("stack", @base.jarray(s.stack.map(fj))),
  ])
}

///|
fn decode(j : Json) -> State raise @base.Fault {
  let code = @base.field(j, "code")
  {
    code: {
      statics: @a.decode_names(@base.field(code, "statics")),
      body: sd(@base.field(code, "body")),
    },
    env: @a.decode_env(@base.field(j, "env")),
    exec: ed(@base.field(j, "exec")),
    stack: @base.array(@base.field(j, "stack")).map(fd),
  }
}

///|
pub fn model() -> @runtime.Model {
  @runtime.model(make, step, encode, decode, s => {
    let stack = s.stack.rev()
    @base.jarray([
      @base.titled("exec", et(s.exec)),
      @base.titled("env", @a.print_env(s.env, trailing_newline=false)),
      @base.titled(
        "stack",
        @base.join(
          stack.map(f => {
            match f {
              SeqFrame(b) => "seq " + st(b)
              LoopFrame(l, _, n) => "loop " + l + " -> " + n
            }
          }),
          "\n",
        ),
      ),
    ])
  })
}

///|
pub fn parse_code(text : String) -> Code raise @base.Fault {
  let p = @p.create(text)
  p.expect_keyword("static")
  let statics = @a.names(p)
  p.expect(';')
  let body = statement(p)
  p.finish()
  { statics, body, }
}
