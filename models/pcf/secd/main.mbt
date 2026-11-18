///|
pub(all) enum Instr {
  PushNat(@base.Nat)
  PushBool(Bool)
  PushUnit
  Print(@base.Nat)
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
  VNat(@base.Nat)
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
priv struct State {
  code : Array[Instr]
  stack : Array[Value]
  env : Env
  control : Array[Instr]
  dump : Array[DumpFrame]
}

///|
fn nat_json(n : @base.Nat) -> Json {
  Json::number(n.to_double(), repr=n.to_string())
}

///|
pub fn instr_json(i : Instr) -> Json {
  match i {
    PushNat(n) => @base.object([("PushNat", nat_json(n))])
    PushBool(b) => @base.object([("PushBool", @base.jbool(b))])
    PushUnit => @base.jstr("PushUnit")
    Print(n) => @base.object([("Print", nat_json(n))])
    Acc(x) => @base.object([("Acc", @base.jstr(x))])
    MkCls(x, c) =>
      @base.object([
        (
          "MkCls",
          @base.object([("param", @base.jstr(x)), ("code", code_json(c))]),
        ),
      ])
    MkRec(f, x, c) =>
      @base.object([
        (
          "MkRec",
          @base.object([
            ("name", @base.jstr(f)),
            ("param", @base.jstr(x)),
            ("code", code_json(c)),
          ]),
        ),
      ])
    BinOp(o) =>
      @base.object([
        (
          "BinOp",
          @base.jstr(
            match o {
              @s.Add => "Add"
              @s.Sub => "Sub"
              @s.And => "And"
            },
          ),
        ),
      ])
    UnOp(o) =>
      @base.object([
        (
          "UnOp",
          @base.jstr(
            match o {
              @s.Inc => "Inc"
              @s.Dec => "Dec"
              @s.Not => "Not"
            },
          ),
        ),
      ])
    Ap => @base.jstr("Ap")
    Ret => @base.jstr("Ret")
    If(t, e) =>
      @base.object([
        (
          "If",
          @base.object([
            ("then_code", code_json(t)),
            ("else_code", code_json(e)),
          ]),
        ),
      ])
  }
}

///|
pub fn code_json(c : Array[Instr]) -> Json {
  @base.jarray(c.map(instr_json))
}

///|
pub fn parse_instr(j : Json) -> Instr raise @base.Fault {
  match j {
    String("PushUnit") => PushUnit
    String("Ap") => Ap
    String("Ret") => Ret
    Object(m) if m.length() == 1 => {
      let (key, value) = m.iter().head().unwrap()
      match key {
        "PushNat" => PushNat(@base.json_nat(value))
        "PushBool" => PushBool(@base.boolean(value))
        "Print" => Print(@base.json_nat(value))
        "Acc" => Acc(@base.identifier(@base.string(value)))
        "MkCls" =>
          MkCls(
            @base.identifier(@base.string(@base.field(value, "param"))),
            parse_code_json(@base.field(value, "code")),
          )
        "MkRec" =>
          MkRec(
            @base.identifier(@base.string(@base.field(value, "name"))),
            @base.identifier(@base.string(@base.field(value, "param"))),
            parse_code_json(@base.field(value, "code")),
          )
        "BinOp" =>
          BinOp(
            match @base.string(value) {
              "Add" => @s.Add
              "Sub" => @s.Sub
              "And" => @s.And
              _ => @base.error("unknown binop")
            },
          )
        "UnOp" =>
          UnOp(
            match @base.string(value) {
              "Inc" => @s.Inc
              "Dec" => @s.Dec
              "Not" => @s.Not
              _ => @base.error("unknown unop")
            },
          )
        "If" =>
          If(
            parse_code_json(@base.field(value, "then_code")),
            parse_code_json(@base.field(value, "else_code")),
          )
        _ => @base.error("invalid SECD instruction")
      }
    }
    _ => @base.error("invalid SECD instruction")
  }
}

///|
pub fn parse_code_json(j : Json) -> Array[Instr] raise @base.Fault {
  @base.array(j).map(parse_instr)
}

///|
pub fn parse_code(text : String) -> Array[Instr] raise @base.Fault {
  parse_code_json(@base.parse_json(text))
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
fn scalar(e : @s.Expr) -> Value raise @base.Fault {
  match e {
    @s.Nat(n) => VNat(n)
    @s.Bool(b) => VBool(b)
    _ => @base.error("Invalid scalar result")
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
fn make(text : String, input : String) -> State raise @base.Fault {
  let code = parse_code(text)
  let control = code.copy()
  for n in @s.inputs(input) {
    control.push(PushNat(n))
    control.push(Ap)
  }
  { code, control, stack: [], env: [], dump: [], }
}

///|
fn step_once(s : State) -> (State, String) raise @base.Fault {
  @base.require(!s.control.is_empty(), "no instruction to execute")
  let i = s.control[0]
  let s = { ..s, control: s.control[1:].to_owned(), }
  let next = match i {
    PushNat(n) => { ..s, stack: push(VNat(n), s.stack), }
    PushBool(b) => { ..s, stack: push(VBool(b), s.stack), }
    PushUnit => { ..s, stack: push(VUnit, s.stack), }
    Print(n) => return ({ ..s, stack: push(VUnit, s.stack), }, n.to_string())
    Acc(x) => {
      let v = match @data.env_get(s.env, x) {
        Some(v) => v
        None => @base.error("unbound variable: " + x)
      }
      { ..s, stack: push(v, s.stack), }
    }
    MkCls(x, c) => { ..s, stack: push(VFun(x, c, s.env), s.stack), }
    MkRec(f, x, c) => { ..s, stack: push(VRec(f, x, c, s.env), s.stack), }
    BinOp(o) => {
      @base.require(s.stack.length() >= 2, "stack underflow")
      let v = scalar(@s.binop(o, expr(s.stack[1]), expr(s.stack[0])))
      { ..s, stack: push(v, s.stack[2:].to_owned()), }
    }
    UnOp(o) => {
      @base.require(!s.stack.is_empty(), "stack underflow")
      let v = scalar(@s.unop(o, expr(s.stack[0])))
      { ..s, stack: push(v, s.stack[1:].to_owned()), }
    }
    Ap => {
      @base.require(s.stack.length() >= 2, "stack underflow")
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
          @base.error("application expects a closure, found " + value_text(f))
      }
    }
    Ret => {
      @base.require(!s.stack.is_empty(), "stack underflow")
      @base.require(!s.dump.is_empty(), "ret without a caller frame")
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
      @base.require(!s.stack.is_empty(), "stack underflow")
      let b = match s.stack[0] {
        VBool(b) => b
        v =>
          @base.error("if expects a boolean condition, found " + value_text(v))
      }
      let c = (if b { t } else { e }).copy()
      c.append(s.control)
      { ..s, stack: s.stack[1:].to_owned(), control: c, }
    }
  }
  (next, "")
}

///|
fn step(
  s : State,
  input : String,
) -> @runtime.Transition[State] raise @base.Fault {
  @base.unit_input(input)
  if s.control.is_empty() && s.dump.is_empty() {
    @base.require(!s.stack.is_empty(), "empty stack at halt")
    match s.stack[0] {
      VNat(n) => @runtime.Halt(n.to_string())
      _ => @base.error("final value is not a natural number")
    }
  } else {
    let (next, out) = step_once(s)
    @runtime.Continue(next, out)
  }
}

///|
fn frame_json(f : DumpFrame) -> Json {
  @base.object([
    ("stack", @base.jarray(f.stack.map(value_json))),
    ("env", env_json(f.env)),
    ("control", @base.jarray(f.control.map(instr_snapshot_json))),
  ])
}

///|
fn decode_frame(j : Json) -> DumpFrame raise @base.Fault {
  {
    stack: @base.array(@base.field(j, "stack")).map(decode_value),
    env: decode_env(@base.field(j, "env")),
    control: @base.array(@base.field(j, "control")).map(decode_instr),
  }
}

///|
fn encode(s : State) -> Json {
  @base.object([
    ("code", @base.jarray(s.code.map(instr_snapshot_json))),
    ("stack", @base.jarray(s.stack.map(value_json))),
    ("env", env_json(s.env)),
    ("control", @base.jarray(s.control.map(instr_snapshot_json))),
    ("dump", @base.jarray(s.dump.map(frame_json))),
  ])
}

///|
fn decode(j : Json) -> State raise @base.Fault {
  {
    code: @base.array(@base.field(j, "code")).map(decode_instr),
    stack: @base.array(@base.field(j, "stack")).map(decode_value),
    env: decode_env(@base.field(j, "env")),
    control: @base.array(@base.field(j, "control")).map(decode_instr),
    dump: @base.array(@base.field(j, "dump")).map(decode_frame),
  }
}

///|
fn render_env(env : Env) -> Json {
  @base.table(
    "env",
    ["name", "value"],
    env.rev().map(p => [p.0, value_text(p.1)]),
  )
}

///|
fn render(s : State) -> Json {
  @base.jarray([
    @base.titled("code", print_code(s.code)),
    @base.table("status", ["field", "value"], [
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
    @base.table(
      "stack",
      ["depth", "value"],
      s.stack.mapi((i, v) => [i.to_string(), value_text(v)]),
    ),
    render_env(s.env),
    @base.table(
      "control",
      ["pc", "instr"],
      s.control.mapi((i, v) => [i.to_string(), instr_json(v).stringify()]),
    ),
    @base.table(
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
  ])
}

///|
pub fn model() -> @runtime.Model {
  @runtime.model(make, step, encode, decode, render)
}
