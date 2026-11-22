///|
struct State {
  code : @utils.LoopCode
  pc : Int32
  modulus : Int32
  tape : @utils.Tape[Int32]
}

///|
fn parse_code(text : String) -> @utils.LoopCode raise @utils.Fault {
  let tokens = []
  for c in text.to_array() {
    if c.is_ascii_whitespace() {
      continue
    }
    @utils.require(
      c == 'R' || c == 'λ' || c == '(' || c == ')',
      "Expected R, λ, ( or )",
    )
    tokens.push(c.to_string())
  }
  @utils.loop_code(tokens, "(", ")", false)
}

///|
fn natural(text : String) -> Int32 raise @utils.Fault {
  let n = @utils.integer(text)
  @utils.require(n >= 0, "Expected natural cell")
  n
}

///|
fn validate(s : State) -> State raise @utils.Fault {
  @utils.require(s.modulus >= 2, "Alphabet size must be at least 2")
  @utils.require(
    s.tape.values().all(n => n >= 0 && n < s.modulus),
    "Tape symbols must be smaller than the alphabet size",
  )
  @utils.require(
    s.pc >= 0 && s.pc <= s.code.instructions.length(),
    "Invalid instruction pointer",
  )
  s
}

///|
fn make(code : Code, input : Configuration) -> State raise @utils.Fault {
  validate({
    code: code.value,
    pc: 0,
    modulus: input.modulus,
    tape: input.tape,
  })
}

///|
fn step(
  s : State,
  _input : Rinput,
) -> @definition.Transition[State, Routput, Foutput] raise @utils.Fault {
  if s.pc == s.code.instructions.length() {
    return @definition.Halt(NaturalTape::{ value: s.tape, })
  }
  let next = { ..s, pc: s.pc + 1, }
  let state = match s.code.instructions[s.pc] {
    "R" =>
      if s.tape.right.is_empty() {
        next
      } else {
        { ..next, tape: s.tape.shift(1, 0), }
      }
    "λ" =>
      {
        ..next,
        tape: s.tape
        .write(if s.tape.head == s.modulus - 1 { 0 } else { s.tape.head + 1 })
        .shift(-1, 0),
      }
    "(" =>
      if s.tape.head == 0 {
        { ..next, pc: s.code.jumps[s.pc] + 1, }
      } else {
        next
      }
    ")" =>
      if s.tape.head != 0 {
        { ..next, pc: s.code.jumps[s.pc] + 1, }
      } else {
        next
      }
    _ => @utils.error("Invalid instruction")
  }
  @definition.Continue(state, ())
}

///|
fn encode(s : State) -> Json {
  @utils.object([
    ("code", @utils.jstr(@utils.join(s.code.instructions, ""))),
    ("pc", @utils.jint(s.pc)),
    (
      "config",
      @utils.object([
        ("modulus", @utils.jint(s.modulus)),
        ("tape", s.tape.json(@utils.jint)),
      ]),
    ),
  ])
}

///|
fn decode(value : Json) -> State raise @utils.Fault {
  let config = @utils.field(value, "config")
  validate({
    code: parse_code(@utils.string(@utils.field(value, "code"))),
    pc: @utils.int(@utils.field(value, "pc")),
    modulus: @utils.int(@utils.field(config, "modulus")),
    tape: @utils.decode_tape(@utils.field(config, "tape"), @utils.int),
  })
}

///|
fn render(s : State) -> Array[@definition.Block] {
  [
    @definition.highlight("code", s.pc, s.code.instructions),
    @definition.titled("pc", s.pc.to_string()),
    @definition.titled("alphabet size", s.modulus.to_string()),
    @definition.highlight(
      "tape (infinite to the left; right boundary shown)",
      s.tape.left.length(),
      s.tape.values().map(n => n.to_string()),
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
