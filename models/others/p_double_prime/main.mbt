///|
priv struct State {
  code : @data.LoopCode
  pc : Int32
  modulus : Int32
  tape : @data.Tape[Int32]
}

///|
fn parse_code(text : String) -> @data.LoopCode raise @base.Fault {
  let tokens = []
  for c in text.to_array() {
    if c.is_ascii_whitespace() {
      continue
    }
    @base.require(
      c == 'R' || c == 'λ' || c == '(' || c == ')',
      "Expected R, λ, ( or )",
    )
    tokens.push(c.to_string())
  }
  @data.loop_code(tokens, "(", ")", false)
}

///|
fn natural(text : String) -> Int32 raise @base.Fault {
  let n = @base.integer(text)
  @base.require(n >= 0, "Expected natural cell")
  n
}

///|
fn validate(s : State) -> State raise @base.Fault {
  @base.require(s.modulus >= 2, "Alphabet size must be at least 2")
  @base.require(
    s.tape.values().all(n => n >= 0 && n < s.modulus),
    "Tape symbols must be smaller than the alphabet size",
  )
  @base.require(
    s.pc >= 0 && s.pc <= s.code.instructions.length(),
    "Invalid instruction pointer",
  )
  s
}

///|
fn make(code : String, input : String) -> State raise @base.Fault {
  let (modulus, tape) = if input.trim().is_empty() {
    (2, @data.blank(0))
  } else {
    match input.trim().to_owned().split_once("\n") {
      None =>
        @base.error("Expected alphabet size, then a tape on the next line")
      Some((modulus, tape)) =>
        (
          natural(modulus.to_owned()),
          @data.parse_tape(tape.to_owned(), natural),
        )
    }
  }
  validate({ code: parse_code(code), pc: 0, modulus, tape, })
}

///|
fn step(
  s : State,
  input : String,
) -> @runtime.Transition[State] raise @base.Fault {
  @base.unit_input(input)
  if s.pc == s.code.instructions.length() {
    return @runtime.Halt(s.tape.text(n => n.to_string()))
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
    _ => @base.error("Invalid instruction")
  }
  @runtime.Continue(state, "")
}

///|
fn encode(s : State) -> Json {
  @base.object([
    ("code", @base.jstr(@base.join(s.code.instructions, ""))),
    ("pc", @base.jint(s.pc)),
    (
      "config",
      @base.object([
        ("modulus", @base.jint(s.modulus)),
        ("tape", s.tape.json(@base.jint)),
      ]),
    ),
  ])
}

///|
fn decode(value : Json) -> State raise @base.Fault {
  let config = @base.field(value, "config")
  validate({
    code: parse_code(@base.string(@base.field(value, "code"))),
    pc: @base.int(@base.field(value, "pc")),
    modulus: @base.int(@base.field(config, "modulus")),
    tape: @data.decode_tape(@base.field(config, "tape"), @base.int),
  })
}

///|
fn render(s : State) -> Json {
  @base.jarray([
    @base.highlight("code", s.pc, s.code.instructions),
    @base.titled("pc", s.pc.to_string()),
    @base.titled("alphabet size", s.modulus.to_string()),
    @base.highlight(
      "tape (infinite to the left; right boundary shown)",
      s.tape.left.length(),
      s.tape.values().map(n => n.to_string()),
    ),
  ])
}

///|
pub fn model() -> @runtime.Model {
  @runtime.model(make, step, encode, decode, render)
}
