///|
pub(all) struct Code {
  value : @utils.LoopCode
}

///|
pub(all) struct NaturalTape {
  value : @utils.Tape[Int32]
}

///|
pub(all) struct Configuration {
  modulus : Int32
  tape : @utils.Tape[Int32]
}

///|
pub impl @definition.Text for Code with fn parse(text) {
  { value: parse_code(text), }
}

///|
pub impl @definition.Text for NaturalTape with fn parse(text) {
  { value: @utils.parse_tape(text, natural), }
}

///|
pub impl @definition.Text for NaturalTape with fn print(value) {
  value.value.text(n => n.to_string())
}

///|
pub impl @definition.Text for Configuration with fn parse(text) {
  let (modulus, tape) = if text.trim().is_empty() {
    (2, @utils.blank(0))
  } else {
    match text.trim().to_owned().split_once("\n") {
      None =>
        @utils.error("Expected alphabet size, then a tape on the next line")
      Some((modulus, tape)) =>
        (
          natural(modulus.to_owned()),
          @utils.parse_tape(tape.to_owned(), natural),
        )
    }
  }
  { modulus, tape, }
}

///|
pub type Ainput = Configuration

///|
pub type Rinput = Unit

///|
pub type Routput = Unit

///|
pub type Foutput = NaturalTape

///|
pub impl @definition.Snapshot for State with fn encode(value) {
  encode(value)
}

///|
pub impl @definition.Snapshot for State with fn decode(value) {
  decode(value)
}

///|
pub impl @definition.Render for State with fn render(value) {
  render(value)
}

///|
pub impl @definition.Text for Code with fn print(value) {
  @utils.join(value.value.instructions, "")
}

///|
pub impl @definition.Text for Configuration with fn print(value) {
  value.modulus.to_string() + "\n" + value.tape.text(n => n.to_string())
}
