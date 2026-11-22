///|
pub impl @definition.Text for Term with fn parse(text) {
  parse(text)
}

///|
pub impl @definition.Text for Term with fn print(value) {
  print(value)
}

///|
pub type Code = Term

///|
pub type Ainput = Unit

///|
pub type Rinput = Unit

///|
pub type Routput = Unit

///|
pub type Foutput = Term

///|
pub impl @definition.Snapshot for Term with fn encode(value) {
  encode(value)
}

///|
pub impl @definition.Snapshot for Term with fn decode(value) {
  decode(value)
}

///|
pub impl @definition.Render for Term with fn render(value) {
  render(value)
}
