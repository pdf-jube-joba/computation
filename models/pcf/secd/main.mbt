///|
pub(all) enum Instr {
  PushNat(@utils.Nat)
  PushBool(Bool)
  PushUnit
  Print(@utils.Nat)
  Acc(String)
  MkCls(String, Array[Instr])
  MkRec(String, String, Array[Instr])
  BinOp(@s.Binop)
  UnOp(@s.Unop)
  Ap
  Ret
  If(Array[Instr], Array[Instr])
}

///|
priv enum Value {
  VNat(@utils.Nat)
  VBool(Bool)
  VUnit
  VFun(String, Array[Instr], Array[(String, Value)])
  VRec(String, String, Array[Instr], Array[(String, Value)])
}

///|
type Env = Array[(String, Value)]

///|
priv struct DumpFrame {
  stack : Array[Value]
  env : Env
  control : Array[Instr]
}

///|
struct State {
  code : Array[Instr]
  stack : Array[Value]
  env : Env
  control : Array[Instr]
  dump : Array[DumpFrame]
}

///|
fn nat_json(n : @utils.Nat) -> Json {
  Json::number(n.to_double(), repr=n.to_string())
}

///|
pub fn instr_json(i : Instr) -> Json {
  match i {
    PushNat(n) => @utils.object([("PushNat", nat_json(n))])
    PushBool(b) => @utils.object([("PushBool", @utils.jbool(b))])
    PushUnit => @utils.jstr("PushUnit")
    Print(n) => @utils.object([("Print", nat_json(n))])
    Acc(x) => @utils.object([("Acc", @utils.jstr(x))])
    MkCls(x, c) =>
      @utils.object([
        (
          "MkCls",
          @utils.object([("param", @utils.jstr(x)), ("code", code_json(c))]),
        ),
      ])
    MkRec(f, x, c) =>
      @utils.object([
        (
          "MkRec",
          @utils.object([
            ("name", @utils.jstr(f)),
            ("param", @utils.jstr(x)),
            ("code", code_json(c)),
          ]),
        ),
      ])
    BinOp(o) =>
      @utils.object([
        (
          "BinOp",
          @utils.jstr(
            match o {
              @s.Add => "Add"
              @s.Sub => "Sub"
              @s.And => "And"
            },
          ),
        ),
      ])
    UnOp(o) =>
      @utils.object([
        (
          "UnOp",
          @utils.jstr(
            match o {
              @s.Inc => "Inc"
              @s.Dec => "Dec"
              @s.Not => "Not"
            },
          ),
        ),
      ])
    Ap => @utils.jstr("Ap")
    Ret => @utils.jstr("Ret")
    If(t, e) =>
      @utils.object([
        (
          "If",
          @utils.object([
            ("then_code", code_json(t)),
            ("else_code", code_json(e)),
          ]),
        ),
      ])
  }
}

///|
pub fn code_json(c : Array[Instr]) -> Json {
  @utils.jarray(c.map(instr_json))
}

///|
pub fn parse_instr(j : Json) -> Instr raise @utils.Fault {
  match j {
    String("PushUnit") => PushUnit
    String("Ap") => Ap
    String("Ret") => Ret
    Object(m) if m.length() == 1 => {
      let (key, value) = m.iter().head().unwrap()
      match key {
        "PushNat" => PushNat(@utils.json_nat(value))
        "PushBool" => PushBool(@utils.boolean(value))
        "Print" => Print(@utils.json_nat(value))
        "Acc" => Acc(@utils.identifier(@utils.string(value)))
        "MkCls" =>
          MkCls(
            @utils.identifier(@utils.string(@utils.field(value, "param"))),
            parse_code_json(@utils.field(value, "code")),
          )
        "MkRec" =>
          MkRec(
            @utils.identifier(@utils.string(@utils.field(value, "name"))),
            @utils.identifier(@utils.string(@utils.field(value, "param"))),
            parse_code_json(@utils.field(value, "code")),
          )
        "BinOp" =>
          BinOp(
            match @utils.string(value) {
              "Add" => @s.Add
              "Sub" => @s.Sub
              "And" => @s.And
              _ => @utils.error("unknown binop")
            },
          )
        "UnOp" =>
          UnOp(
            match @utils.string(value) {
              "Inc" => @s.Inc
              "Dec" => @s.Dec
              "Not" => @s.Not
              _ => @utils.error("unknown unop")
            },
          )
        "If" =>
          If(
            parse_code_json(@utils.field(value, "then_code")),
            parse_code_json(@utils.field(value, "else_code")),
          )
        _ => @utils.error("invalid SECD instruction")
      }
    }
    _ => @utils.error("invalid SECD instruction")
  }
}

///|
pub fn parse_code_json(j : Json) -> Array[Instr] raise @utils.Fault {
  @utils.array(j).map(parse_instr)
}

///|
pub fn parse_code(text : String) -> Array[Instr] raise @utils.Fault {
  parse_code_json(@utils.parse_json(text))
}

///|
pub fn print_code(code : Array[Instr]) -> String {
  code_json(code).stringify(indent=2)
}

///|
fn value_text(v : Value) -> String {
  match v {
    VNat(n) => n.to_string()
    VBool(b) => if b { "\\true" } else { "\\false" }
    VUnit => "\\unit"
    VFun(x, c, _) => "closure " + x + " " + print_code(c)
    VRec(f, x, c, _) => "recursive closure " + f + " " + x + " " + print_code(c)
  }
}

///|
fn expr(v : Value) -> @s.Expr {
  match v {
    VNat(n) => @s.Nat(n)
    VBool(b) => @s.Bool(b)
    _ => @s.Unit
  }
}

///|
fn scalar(e : @s.Expr) -> Value raise @utils.Fault {
  match e {
    @s.Nat(n) => VNat(n)
    @s.Bool(b) => VBool(b)
    _ => @utils.error("Invalid scalar result")
  }
}

///|
fn bind(env : Env, x : String, v : Value) -> Env {
  let e = [(x, v)]
  e.append(env)
  e
}

///|
fn push(v : Value, xs : Array[Value]) -> Array[Value] {
  let a = [v]
  a.append(xs)
  a
}

///|
fn make(program : Code, input : @s.Inputs) -> State {
  let code = program.instructions
  let control = code.copy()
  for n in input.values {
    control.push(PushNat(n))
    control.push(Ap)
  }
  { code, control, stack: [], env: [], dump: [], }
}

///|
fn step_once(s : State) -> (State, @s.Output) raise @utils.Fault {
  @utils.require(!s.control.is_empty(), "no instruction to execute")
  let i = s.control[0]
  let s = { ..s, control: s.control[1:].to_owned(), }
  let next = match i {
    PushNat(n) => { ..s, stack: push(VNat(n), s.stack), }
    PushBool(b) => { ..s, stack: push(VBool(b), s.stack), }
    PushUnit => { ..s, stack: push(VUnit, s.stack), }
    Print(n) => return ({ ..s, stack: push(VUnit, s.stack), }, @s.Printed(n))
    Acc(x) => {
      let v = match @utils.env_get(s.env, x) {
        Some(v) => v
        None => @utils.error("unbound variable: " + x)
      }
      { ..s, stack: push(v, s.stack), }
    }
    MkCls(x, c) => { ..s, stack: push(VFun(x, c, s.env), s.stack), }
    MkRec(f, x, c) => { ..s, stack: push(VRec(f, x, c, s.env), s.stack), }
    BinOp(o) => {
      @utils.require(s.stack.length() >= 2, "stack underflow")
      let v = scalar(@s.binop(o, expr(s.stack[1]), expr(s.stack[0])))
      { ..s, stack: push(v, s.stack[2:].to_owned()), }
    }
    UnOp(o) => {
      @utils.require(!s.stack.is_empty(), "stack underflow")
      let v = scalar(@s.unop(o, expr(s.stack[0])))
      { ..s, stack: push(v, s.stack[1:].to_owned()), }
    }
    Ap => {
      @utils.require(s.stack.length() >= 2, "stack underflow")
      let arg = s.stack[0]
      let f = s.stack[1]
      let frame = {
        stack: s.stack[2:].to_owned(),
        env: s.env,
        control: s.control,
      }
      let dump = [frame]
      dump.append(s.dump)
      match f {
        VFun(x, c, env) =>
          { ..s, stack: [], env: bind(env, x, arg), control: c, dump, }
        VRec(f, x, c, env) as r =>
          {
            ..s,
            stack: [],
            env: bind(bind(env, f, r), x, arg),
            control: c,
            dump,
          }
        _ =>
          @utils.error("application expects a closure, found " + value_text(f))
      }
    }
    Ret => {
      @utils.require(!s.stack.is_empty(), "stack underflow")
      @utils.require(!s.dump.is_empty(), "ret without a caller frame")
      let f = s.dump[0]
      {
        ..s,
        stack: push(s.stack[0], f.stack),
        env: f.env,
        control: f.control,
        dump: s.dump[1:].to_owned(),
      }
    }
    If(t, e) => {
      @utils.require(!s.stack.is_empty(), "stack underflow")
      let b = match s.stack[0] {
        VBool(b) => b
        v =>
          @utils.error("if expects a boolean condition, found " + value_text(v))
      }
      let c = (if b { t } else { e }).copy()
      c.append(s.control)
      { ..s, stack: s.stack[1:].to_owned(), control: c, }
    }
  }
  (next, @s.Silent)
}

///|
fn step(
  s : State,
  _input : Rinput,
) -> @definition.Transition[State, Routput, Foutput] raise @utils.Fault {
  if s.control.is_empty() && s.dump.is_empty() {
    @utils.require(!s.stack.is_empty(), "empty stack at halt")
    match s.stack[0] {
      VNat(n) => @definition.Halt(n)
      _ => @utils.error("final value is not a natural number")
    }
  } else {
    let (next, out) = step_once(s)
    @definition.Continue(next, out)
  }
}

///|
fn frame_json(f : DumpFrame) -> Json {
  @utils.object([
    ("stack", @utils.jarray(f.stack.map(value_json))),
    ("env", env_json(f.env)),
    ("control", @utils.jarray(f.control.map(instr_snapshot_json))),
  ])
}

///|
fn decode_frame(j : Json) -> DumpFrame raise @utils.Fault {
  {
    stack: @utils.array(@utils.field(j, "stack")).map(decode_value),
    env: decode_env(@utils.field(j, "env")),
    control: @utils.array(@utils.field(j, "control")).map(decode_instr),
  }
}

///|
fn encode(s : State) -> Json {
  @utils.object([
    ("code", @utils.jarray(s.code.map(instr_snapshot_json))),
    ("stack", @utils.jarray(s.stack.map(value_json))),
    ("env", env_json(s.env)),
    ("control", @utils.jarray(s.control.map(instr_snapshot_json))),
    ("dump", @utils.jarray(s.dump.map(frame_json))),
  ])
}

///|
fn decode(j : Json) -> State raise @utils.Fault {
  {
    code: @utils.array(@utils.field(j, "code")).map(decode_instr),
    stack: @utils.array(@utils.field(j, "stack")).map(decode_value),
    env: decode_env(@utils.field(j, "env")),
    control: @utils.array(@utils.field(j, "control")).map(decode_instr),
    dump: @utils.array(@utils.field(j, "dump")).map(decode_frame),
  }
}

///|
fn render_env(env : Env) -> @definition.Block {
  @definition.text_table(
    "env",
    ["name", "value"],
    env.rev().map(p => [p.0, value_text(p.1)]),
  )
}

///|
fn render(s : State) -> Array[@definition.Block] {
  [
    @definition.titled("code", print_code(s.code)),
    @definition.text_table("status", ["field", "value"], [
      [
        "next",
        if s.control.is_empty() {
          "ret"
        } else {
          instr_json(s.control[0]).stringify()
        },
      ],
      ["dump_depth", s.dump.length().to_string()],
    ]),
    @definition.text_table(
      "stack",
      ["depth", "value"],
      s.stack.mapi((i, v) => [i.to_string(), value_text(v)]),
    ),
    render_env(s.env),
    @definition.text_table(
      "control",
      ["pc", "instr"],
      s.control.mapi((i, v) => [i.to_string(), instr_json(v).stringify()]),
    ),
    @definition.text_table(
      "dump",
      ["depth", "stack", "env", "control"],
      s.dump.mapi((i, f) => {
        [
          i.to_string(),
          f.stack.length().to_string(),
          f.env.length().to_string(),
          f.control.length().to_string(),
        ]
      }),
    ),
  ]
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
