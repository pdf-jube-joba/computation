///|
pub(all) struct Code {
  words : Array[@utils.Nat]
}

///|
pub impl @definition.Text for Code with fn parse(text) {
  { words: @machine.natural_lines(text), }
}

///|
pub type Ainput = @machine.Image

///|
pub type Rinput = Unit

///|
pub type Routput = Unit

///|
pub type Foutput = @machine.Image

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
  @utils.join(value.words.map(n => n.to_string()), "\n")
}
