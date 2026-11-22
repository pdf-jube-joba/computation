///|
pub(all) struct Code {
  instructions : Array[Instr]
}

///|
pub impl @definition.Text for Code with fn parse(text) {
  { instructions: parse_code(text), }
}

///|
pub impl @definition.Text for Code with fn print(value) {
  print_code(value.instructions)
}

///|
pub type Ainput = @s.Inputs

///|
pub type Rinput = Unit

///|
pub type Routput = @s.Output

///|
pub type Foutput = @utils.Nat

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
