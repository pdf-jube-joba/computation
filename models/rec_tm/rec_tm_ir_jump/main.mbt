///|
#derive(Eq)
pub(all) enum Stmt {
  Lt
  Rt
  Assign(@s.Lvalue, @s.Rvalue)
  Jump(Int32, @s.Condition?)
}

///|
#derive(Eq)
pub(all) struct Code {
  alphabet : Array[String]
  body : Array[Stmt]
}

///|
priv struct State {
  program : Code
  pc : Int32
  env : @s.Environment
  tape : @data.Tape[String]
}

///|
pub fn parse_code(text : String) -> Code raise @base.Fault {
  let p = @p.create(text)
  let alphabet = @s.alphabet(p)
  let body = []
  while !p.eof() {
    let stmt = if p.eat_keyword("LT") {
      Lt
    } else if p.eat_keyword("RT") {
      Rt
    } else if p.eat_keyword("jump") {
      let c = @s.condition(p)
      Jump(p.integer(), c)
    } else {
      let dst = @s.lvalue(p)
      p.expect_text(":=")
      Assign(dst, @s.rvalue(p))
    }
    body.push(stmt)
  }
  { alphabet, body, }
}

///|
pub fn stmt_text(stmt : Stmt) -> String {
  match stmt {
    Lt => "\\LT"
    Rt => "\\RT"
    Assign(dst, src) => @s.ltext(dst) + " := " + @s.rtext(src)
    Jump(target, c) => "\\jump" + @s.ctext(c) + " " + target.to_string()
  }
}

///|
pub fn print_code(code : Code) -> String {
  "\\alphabet: (" +
  @base.join(code.alphabet, ", ") +
  ")\n" +
  @base.join(code.body.map(s => stmt_text(s) + "\n"), "")
}

///|
pub fn collect_vars(body : Array[Stmt]) -> Array[String] {
  let names = []
  for stmt in body {
    match stmt {
      Assign(dst, src) => {
        names.append(@s.lvars(dst))
        names.append(@s.rvars(src))
      }
      Jump(_, c) => names.append(@s.cvars(c))
      _ => ()
    }
  }
  @s.env_create(names).names.map(pair => pair.0)
}

///|
pub fn validate(code : Code) -> Unit raise @base.Fault {
  let allowed = @s.normalized_alphabet(code.alphabet)
  for i, stmt in code.body {
    match stmt {
      Assign(_, src) => @s.validate_value(allowed, src)
      Jump(target, c) => {
        @base.require(
          target >= 0 && target <= code.body.length(),
          "jump target out of range at " +
          i.to_string() +
          ": " +
          target.to_string(),
        )
        @s.validate_condition(allowed, c)
      }
      _ => ()
    }
  }
}

///|
fn make(text : String, input : String) -> State raise @base.Fault {
  let program = parse_code(text)
  validate(program)
  let tape = @data.parse_tape(input, @s.sign)
  @s.validate_tape(@s.normalized_alphabet(program.alphabet), tape)
  { program, pc: 0, env: @s.env_create(collect_vars(program.body)), tape, }
}

///|
fn step(
  s : State,
  input : String,
) -> @runtime.Transition[State] raise @base.Fault {
  @base.unit_input(input)
  if s.pc >= s.program.body.length() {
    return @runtime.Halt(s.tape.text(v => v))
  }
  let next = { ..s, pc: s.pc + 1, }
  let next = match s.program.body[s.pc] {
    Lt => { ..next, tape: s.tape.shift(-1, "-"), }
    Rt => { ..next, tape: s.tape.shift(1, "-"), }
    Assign(dst, src) => {
      let (env, tape) = @s.assign(
        s.env,
        s.tape,
        dst,
        @s.eval(s.env, s.tape, src),
      )
      { ..next, env, tape, }
    }
    Jump(target, c) => {
      @base.require(
        target >= 0 && target <= s.program.body.length(),
        "jump target out of range",
      )
      {
        ..next,
        pc: if @s.eval_condition(s.env, s.tape, c) {
          target
        } else {
          s.pc + 1
        },
      }
    }
  }
  @runtime.Continue(next, "")
}

///|
pub fn sj(s : Stmt) -> Json {
  match s {
    Lt => @base.tag("Lt", [])
    Rt => @base.tag("Rt", [])
    Assign(dst, src) => @base.tag("Assign", [@s.lj(dst), @s.rj(src)])
    Jump(target, c) => @base.tag("Jump", [@base.jint(target), @s.cj(c)])
  }
}

///|
pub fn sd(j : Json) -> Stmt raise @base.Fault {
  match @base.tag_name(j) {
    "Lt" => Lt
    "Rt" => Rt
    "Assign" => Assign(@s.ld(@base.tag_arg(j, 0)), @s.rd(@base.tag_arg(j, 1)))
    "Jump" => Jump(@base.int(@base.tag_arg(j, 0)), @s.cd(@base.tag_arg(j, 1)))
    _ => @base.error("Invalid tape jump statement")
  }
}

///|
pub fn code_json(code : Code) -> Json {
  @base.object([
    ("alphabet", @base.jarray(code.alphabet.map(@s.sign_json))),
    ("body", @base.jarray(code.body.map(sj))),
  ])
}

///|
pub fn decode_code(j : Json) -> Code raise @base.Fault {
  {
    alphabet: @base.array(@base.field(j, "alphabet")).map(@s.decode_sign),
    body: @base.array(@base.field(j, "body")).map(sd),
  }
}

///|
fn encode(s : State) -> Json {
  @base.object([
    ("program", code_json(s.program)),
    ("pc", @base.jint(s.pc)),
    ("env", @s.ej(s.env)),
    ("tape", s.tape.json(@s.sign_json)),
  ])
}

///|
fn decode(j : Json) -> State raise @base.Fault {
  let program = decode_code(@base.field(j, "program"))
  validate(program)
  let pc = @base.int(@base.field(j, "pc"))
  @base.require(pc >= 0 && pc <= program.body.length(), "Invalid pc")
  let tape = @data.decode_tape(@base.field(j, "tape"), @s.decode_sign)
  @s.validate_tape(@s.normalized_alphabet(program.alphabet), tape)
  {
    program,
    pc,
    env: @s.ed(@base.field(j, "env"), collect_vars(program.body)),
    tape,
  }
}

///|
pub fn model() -> @runtime.Model {
  @runtime.model(make, step, encode, decode, s => {
    @base.jarray([
      @base.titled("pc", s.pc.to_string()),
      @base.titled(
        "next",
        if s.pc >= s.program.body.length() {
          "halt"
        } else {
          stmt_text(s.program.body[s.pc])
        },
      ),
      @base.table(
        "env",
        ["var", "value"],
        @s.env_named(s.env).map(v => [v.0, v.1]),
      ),
      @base.highlight("tape", s.tape.left.length(), s.tape.values()),
    ])
  })
}

///|
test "conditional jump, implicit blank alphabet, and sorted variable ids" {
  let m = model()
  m.make(
    "\\alphabet: (x) z := @ a := \\const x \\jump \\if z == \\const - 4 @ := \\const - @ := a",
    "|-|",
  )
  for _ in 0..<4 {
    ignore(m.step(""))
  }
  assert_eq(
    @base.string(@base.field(@base.parse_json(m.step("")), "foutput")),
    "|x|",
  )
  assert_true(
    try {
      m.make("\\alphabet: (x) \\jump 2", "|-|")
      false
    } catch {
      _ => true
    },
  )
}
