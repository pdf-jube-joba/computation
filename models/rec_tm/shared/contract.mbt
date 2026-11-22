///|
pub(all) struct Tape {
  value : @utils.Tape[String]
}

///|
pub impl @definition.Text for Tape with fn parse(text) {
  { value: @utils.parse_tape(text, sign), }
}

///|
pub impl @definition.Text for Tape with fn print(value) {
  value.value.text(s => s)
}
