///|
pub impl @definition.Text for Code with fn parse(text) {
  parse_code(text)
}

///|
pub impl @definition.Text for Code with fn print(value) {
  print_code(value)
}

///|
pub type Ainput = @s.Tape

///|
pub type Rinput = Unit

///|
pub type Routput = Unit

///|
pub type Foutput = @s.Tape

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
