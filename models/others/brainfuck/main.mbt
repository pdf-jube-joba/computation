///|
priv struct State {
  code : @data.LoopCode
  pc : Int32
  tape : @data.Tape[Int32]
  input : Array[Int32]
  output_rev : Array[Int32]
}

///|
fn parse_code(text : String) -> @data.LoopCode raise @base.Fault {
  let tokens = text
    .to_array()
    .filter(c => "><+-.,[]".contains_char(c))
    .map(c => c.to_string())
  @data.loop_code(tokens, "[", "]", true)
}

///|
fn byte(value : Json) -> Int32 raise @base.Fault {
  let value = @base.int(value)
  @base.require(value >= 0 && value <= 255, "Expected a byte (0..255)")
  value
}

///|
fn make(code : String, input : String) -> State raise @base.Fault {
  {
    code: parse_code(code),
    pc: 0,
    tape: @data.blank(0),
    input: if input.trim().is_empty() {
      []
    } else {
      @base.array(@base.parse_json(input)).map(byte)
    },
    output_rev: [],
  }
}

///|
fn bytes_text(values : Array[Int32]) -> String {
  "[" + @base.join(values.map(n => n.to_string()), ", ") + "]"
}

///|
fn step(
  s : State,
  input : String,
) -> @runtime.Transition[State] raise @base.Fault {
  @base.unit_input(input)
  @base.require(
    s.pc >= 0 && s.pc <= s.code.instructions.length(),
    "Invalid instruction pointer",
  )
  if s.pc == s.code.instructions.length() {
    let values = s.output_rev.copy()
    values.rev_in_place()
    return @runtime.Halt(bytes_text(values))
  }
  let next = { ..s, pc: s.pc + 1, }
  let state = match s.code.instructions[s.pc] {
    ">" => { ..next, tape: s.tape.shift(1, 0), }
    "<" => {
      @base.require(
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
      return @runtime.Continue(
        { ..next, output_rev: output, },
        s.tape.head.to_string(),
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
    _ => @base.error("Invalid instruction")
  }
  @runtime.Continue(state, "")
}

///|
fn encode(s : State) -> Json {
  @base.object([
    ("code", @base.jstr(@base.join(s.code.instructions, ""))),
    ("pc", @base.jint(s.pc)),
    ("tape", s.tape.json(@base.jint)),
    ("input", @base.jarray(s.input.map(@base.jint))),
    ("output_rev", @base.jarray(s.output_rev.map(@base.jint))),
  ])
}

///|
fn decode(value : Json) -> State raise @base.Fault {
  let code = parse_code(@base.string(@base.field(value, "code")))
  let pc = @base.int(@base.field(value, "pc"))
  @base.require(
    pc >= 0 && pc <= code.instructions.length(),
    "Invalid instruction pointer",
  )
  {
    code,
    pc,
    tape: @data.decode_tape(@base.field(value, "tape"), byte),
    input: @base.array(@base.field(value, "input")).map(byte),
    output_rev: @base.array(@base.field(value, "output_rev")).map(byte),
  }
}

///|
fn render(s : State) -> Json {
  let output = s.output_rev.copy()
  output.rev_in_place()
  @base.jarray([
    @base.highlight("code", s.pc, s.code.instructions),
    @base.titled("pc", s.pc.to_string()),
    @base.highlight(
      "tape",
      s.tape.left.length(),
      s.tape.values().map(n => n.to_string()),
    ),
    @base.titled("remaining input (bytes)", bytes_text(s.input)),
    @base.titled("output (bytes)", bytes_text(output)),
  ])
}

///|
pub fn model() -> @runtime.Model {
  @runtime.model(make, step, encode, decode, render)
}
