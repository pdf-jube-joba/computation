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
  BreakResult(String, @utils.Nat)
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
struct State {
  code : Code
  env : @a.Env
  exec : Exec
  stack : Array[Frame]
}

///|
fn label(p : @utils.Parser) -> String raise @utils.Fault {
  p.expect(':')
  p.ident()
}

///|
fn block(p : @utils.Parser, opening : Char) -> Stmt raise @utils.Fault {
  let g = p.group(opening)
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
  } else if p.peek() == @utils.Symbol('{') {
    block(p, '{')
  } else {
    let n = p.ident()
    p.expect_text(":=")
    Assign(n, @a.parse_aexp(p))
  }
}

///|
fn statement(p : @utils.Parser) -> Stmt raise @utils.Fault {
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
fn make(code : Code, input : @a.Environment) -> State raise @utils.Fault {
  let statics = code.statics
  let env = input.values
  @a.validate_env(
    statics,
    env,
    duplicate="duplicate static variable: ",
    extra="ainput contains undeclared static variable: ",
    missing="ainput missing static variable: ",
  )
  { code, env, exec: Stmt(code.body), stack: [], }
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
  _input : Rinput,
) -> @definition.Transition[State, Routput, Foutput] raise @utils.Fault {
  let output = et(s.exec)
  let next = match s.exec {
    Stmt(Nop) => { ..s, exec: Result(Normal), }
    Stmt(Seq(a, b)) =>
      { ..s, exec: Stmt(a), stack: push(SeqFrame(b), s.stack), }
    Stmt(Assign(n, a)) =>
      {
        ..s,
        exec: Result(Normal),
        env: @utils.env_set(
          s.env,
          n,
          @a.eval_aexp(a, n => @a.env_get(s.env, n)),
        ),
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
          BreakResult(l, _) => @utils.error("break outside loop: " + l)
          ContinueResult(l) => @utils.error("continue outside loop: " + l)
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
                  env: @utils.env_set(s.env, n, value),
                }
              _ => { ..s, stack, exec: Result(control), }
            }
        }
      }
  }
  match (s.exec, next.exec) {
    (Result(_), Result(Normal)) if next.stack.is_empty() =>
      @definition.Halt(@a.Environment::{ values: next.env, })
    _ => @definition.Continue(next, output)
  }
}

///|
fn sj(s : Stmt) -> Json {
  match s {
    Nop => @utils.tag("Nop", [])
    Seq(a, b) => @utils.tag("Seq", [sj(a), sj(b)])
    Assign(n, a) => @utils.tag("Assign", [@utils.jstr(n), @a.aexp_json(a)])
    If(c, b) => @utils.tag("If", [@a.bexp_json(c), sj(b)])
    Break(l, n) => @utils.tag("Break", [@utils.jstr(l), @utils.jstr(n)])
    Continue(l) => @utils.tag("Continue", [@utils.jstr(l)])
    Loop(l, b, n) => @utils.tag("Loop", [@utils.jstr(l), sj(b), @utils.jstr(n)])
  }
}

///|
fn sd(j : Json) -> Stmt raise @utils.Fault {
  fn a(i : Int32) -> Json raise @utils.Fault {
    @utils.tag_arg(j, i)
  }
  fn n(i : Int32) -> String raise @utils.Fault {
    @utils.identifier(@utils.string(a(i)))
  }
  match @utils.tag_name(j) {
    "Nop" => Nop
    "Seq" => Seq(sd(a(0)), sd(a(1)))
    "Assign" => Assign(n(0), @a.decode_aexp(a(1)))
    "If" => If(@a.decode_bexp(a(0)), sd(a(1)))
    "Break" => Break(n(0), n(1))
    "Continue" => Continue(n(0))
    "Loop" => Loop(n(0), sd(a(1)), n(2))
    _ => @utils.error("Invalid control statement")
  }
}

///|
fn cj(c : Control) -> Json {
  match c {
    Normal => @utils.tag("Normal", [])
    BreakResult(l, n) =>
      @utils.tag("Break_result", [@utils.jstr(l), @utils.jstr(n.to_string())])
    ContinueResult(l) => @utils.tag("Continue_result", [@utils.jstr(l)])
  }
}

///|
fn cd(j : Json) -> Control raise @utils.Fault {
  match @utils.tag_name(j) {
    "Normal" => Normal
    "Break_result" =>
      BreakResult(
        @utils.identifier(@utils.string(@utils.tag_arg(j, 0))),
        @utils.json_nat(@utils.tag_arg(j, 1)),
      )
    "Continue_result" =>
      ContinueResult(@utils.identifier(@utils.string(@utils.tag_arg(j, 0))))
    _ => @utils.error("Invalid control result")
  }
}

///|
fn ej(e : Exec) -> Json {
  match e {
    Stmt(s) => @utils.tag("Stmt", [sj(s)])
    Result(c) => @utils.tag("Result", [cj(c)])
  }
}

///|
fn ed(j : Json) -> Exec raise @utils.Fault {
  match @utils.tag_name(j) {
    "Stmt" => Stmt(sd(@utils.tag_arg(j, 0)))
    "Result" => Result(cd(@utils.tag_arg(j, 0)))
    _ => @utils.error("Invalid execution state")
  }
}

///|
fn fj(f : Frame) -> Json {
  match f {
    SeqFrame(s) => @utils.tag("Seq_frame", [sj(s)])
    LoopFrame(l, b, n) =>
      @utils.tag("Loop_frame", [@utils.jstr(l), sj(b), @utils.jstr(n)])
  }
}

///|
fn fd(j : Json) -> Frame raise @utils.Fault {
  match @utils.tag_name(j) {
    "Seq_frame" => SeqFrame(sd(@utils.tag_arg(j, 0)))
    "Loop_frame" =>
      LoopFrame(
        @utils.identifier(@utils.string(@utils.tag_arg(j, 0))),
        sd(@utils.tag_arg(j, 1)),
        @utils.identifier(@utils.string(@utils.tag_arg(j, 2))),
      )
    _ => @utils.error("Invalid control frame")
  }
}

///|
fn encode(s : State) -> Json {
  @utils.object([
    (
      "code",
      @utils.object([
        ("statics", @a.names_json(s.code.statics)),
        ("body", sj(s.code.body)),
      ]),
    ),
    ("env", @a.env_json(s.env)),
    ("exec", ej(s.exec)),
    ("stack", @utils.jarray(s.stack.map(fj))),
  ])
}

///|
fn decode(j : Json) -> State raise @utils.Fault {
  let code = @utils.field(j, "code")
  {
    code: {
      statics: @a.decode_names(@utils.field(code, "statics")),
      body: sd(@utils.field(code, "body")),
    },
    env: @a.decode_env(@utils.field(j, "env")),
    exec: ed(@utils.field(j, "exec")),
    stack: @utils.array(@utils.field(j, "stack")).map(fd),
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

///|
pub fn parse_code(text : String) -> Code raise @utils.Fault {
  let p = @utils.create(text)
  p.expect_keyword("static")
  let statics = @a.names(p)
  p.expect(';')
  let body = statement(p)
  p.finish()
  { statics, body, }
}
