///|
struct State {
  code : @utils.LoopCode
  pc : Int32
  tape : @utils.Tape[Int32]
  input : Array[Int32]
  output_rev : Array[Int32]
}

///|
fn parse_code(text : String) -> @utils.LoopCode raise @utils.Fault {
  let tokens = text
    .to_array()
    .filter(c => "><+-.,[]".contains_char(c))
    .map(c => c.to_string())
  @utils.loop_code(tokens, "[", "]", true)
}

///|
fn byte(value : Json) -> Int32 raise @utils.Fault {
  let value = @utils.int(value)
  @utils.require(value >= 0 && value <= 255, "Expected a byte (0..255)")
  value
}

///|
fn make(code : Code, input : ByteArray) -> State {
  {
    code: code.value,
    pc: 0,
    tape: @utils.blank(0),
    input: input.values,
    output_rev: [],
  }
}

///|
fn bytes_text(values : Array[Int32]) -> String {
  "[" + @utils.join(values.map(n => n.to_string()), ", ") + "]"
}

///|
fn step(
  s : State,
  _input : Rinput,
) -> @definition.Transition[State, Routput, Foutput] raise @utils.Fault {
  @utils.require(
    s.pc >= 0 && s.pc <= s.code.instructions.length(),
    "Invalid instruction pointer",
  )
  if s.pc == s.code.instructions.length() {
    let values = s.output_rev.copy()
    values.rev_in_place()
    return @definition.Halt(ByteArray::{ values, })
  }
  let next = { ..s, pc: s.pc + 1, }
  let state = match s.code.instructions[s.pc] {
    ">" => { ..next, tape: s.tape.shift(1, 0), }
    "<" => {
      @utils.require(
        !s.tape.left.is_empty(),
        "Cannot move left of the tape origin",
      )
      { ..next, tape: s.tape.shift(-1, 0), }
    }
    "+" => { ..next, tape: s.tape.write((s.tape.head + 1) % 256), }
    "-" => { ..next, tape: s.tape.write((s.tape.head + 255) % 256), }
    "." => {
      let output = [s.tape.head]
      output.append(s.output_rev)
      return @definition.Continue(
        { ..next, output_rev: output, },
        Wrote(s.tape.head.to_byte()),
      )
    }
    "," =>
      if s.input.is_empty() {
        { ..next, tape: s.tape.write(0), }
      } else {
        {
          ..next,
          tape: s.tape.write(s.input[0]),
          input: s.input[1:].to_owned(),
        }
      }
    "[" =>
      if s.tape.head == 0 {
        { ..next, pc: s.code.jumps[s.pc] + 1, }
      } else {
        next
      }
    "]" =>
      if s.tape.head != 0 {
        { ..next, pc: s.code.jumps[s.pc] + 1, }
      } else {
        next
      }
    _ => @utils.error("Invalid instruction")
  }
  @definition.Continue(state, Silent)
}

///|
fn encode(s : State) -> Json {
  @utils.object([
    ("code", @utils.jstr(@utils.join(s.code.instructions, ""))),
    ("pc", @utils.jint(s.pc)),
    ("tape", s.tape.json(@utils.jint)),
    ("input", @utils.jarray(s.input.map(@utils.jint))),
    ("output_rev", @utils.jarray(s.output_rev.map(@utils.jint))),
  ])
}

///|
fn decode(value : Json) -> State raise @utils.Fault {
  let code = parse_code(@utils.string(@utils.field(value, "code")))
  let pc = @utils.int(@utils.field(value, "pc"))
  @utils.require(
    pc >= 0 && pc <= code.instructions.length(),
    "Invalid instruction pointer",
  )
  {
    code,
    pc,
    tape: @utils.decode_tape(@utils.field(value, "tape"), byte),
    input: @utils.array(@utils.field(value, "input")).map(byte),
    output_rev: @utils.array(@utils.field(value, "output_rev")).map(byte),
  }
}

///|
fn render(s : State) -> Array[@definition.Block] {
  let output = s.output_rev.copy()
  output.rev_in_place()
  [
    @definition.highlight("code", s.pc, s.code.instructions),
    @definition.titled("pc", s.pc.to_string()),
    @definition.highlight(
      "tape",
      s.tape.left.length(),
      s.tape.values().map(n => n.to_string()),
    ),
    @definition.titled("remaining input (bytes)", bytes_text(s.input)),
    @definition.titled("output (bytes)", bytes_text(output)),
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
