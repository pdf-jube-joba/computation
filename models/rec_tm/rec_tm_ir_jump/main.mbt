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
struct State {
  program : Code
  pc : Int32
  env : @s.Environment
  tape : @utils.Tape[String]
}

///|
pub fn parse_code(text : String) -> Code raise @utils.Fault {
  let p = @utils.create(text)
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
  @utils.join(code.alphabet, ", ") +
  ")\n" +
  @utils.join(code.body.map(s => stmt_text(s) + "\n"), "")
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
pub fn validate(code : Code) -> Unit raise @utils.Fault {
  let allowed = @s.normalized_alphabet(code.alphabet)
  for i, stmt in code.body {
    match stmt {
      Assign(_, src) => @s.validate_value(allowed, src)
      Jump(target, c) => {
        @utils.require(
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
fn make(program : Code, input : @s.Tape) -> State raise @utils.Fault {
  validate(program)
  let tape = input.value
  @s.validate_tape(@s.normalized_alphabet(program.alphabet), tape)
  { program, pc: 0, env: @s.env_create(collect_vars(program.body)), tape, }
}

///|
fn step(
  s : State,
  _input : Rinput,
) -> @definition.Transition[State, Routput, Foutput] raise @utils.Fault {
  if s.pc >= s.program.body.length() {
    return @definition.Halt(@s.Tape::{ value: s.tape, })
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
      @utils.require(
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
  @definition.Continue(next, ())
}

///|
pub fn sj(s : Stmt) -> Json {
  match s {
    Lt => @utils.tag("Lt", [])
    Rt => @utils.tag("Rt", [])
    Assign(dst, src) => @utils.tag("Assign", [@s.lj(dst), @s.rj(src)])
    Jump(target, c) => @utils.tag("Jump", [@utils.jint(target), @s.cj(c)])
  }
}

///|
pub fn sd(j : Json) -> Stmt raise @utils.Fault {
  match @utils.tag_name(j) {
    "Lt" => Lt
    "Rt" => Rt
    "Assign" => Assign(@s.ld(@utils.tag_arg(j, 0)), @s.rd(@utils.tag_arg(j, 1)))
    "Jump" =>
      Jump(@utils.int(@utils.tag_arg(j, 0)), @s.cd(@utils.tag_arg(j, 1)))
    _ => @utils.error("Invalid tape jump statement")
  }
}

///|
pub fn code_json(code : Code) -> Json {
  @utils.object([
    ("alphabet", @utils.jarray(code.alphabet.map(@s.sign_json))),
    ("body", @utils.jarray(code.body.map(sj))),
  ])
}

///|
pub fn decode_code(j : Json) -> Code raise @utils.Fault {
  {
    alphabet: @utils.array(@utils.field(j, "alphabet")).map(@s.decode_sign),
    body: @utils.array(@utils.field(j, "body")).map(sd),
  }
}

///|
fn encode(s : State) -> Json {
  @utils.object([
    ("program", code_json(s.program)),
    ("pc", @utils.jint(s.pc)),
    ("env", @s.ej(s.env)),
    ("tape", s.tape.json(@s.sign_json)),
  ])
}

///|
fn decode(j : Json) -> State raise @utils.Fault {
  let program = decode_code(@utils.field(j, "program"))
  validate(program)
  let pc = @utils.int(@utils.field(j, "pc"))
  @utils.require(pc >= 0 && pc <= program.body.length(), "Invalid pc")
  let tape = @utils.decode_tape(@utils.field(j, "tape"), @s.decode_sign)
  @s.validate_tape(@s.normalized_alphabet(program.alphabet), tape)
  {
    program,
    pc,
    env: @s.ed(@utils.field(j, "env"), collect_vars(program.body)),
    tape,
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
