///|
pub(all) struct MemoryImage {
  values : Memory
}

///|
pub impl @definition.Text for Code with fn parse(text) {
  parse(text)
}

///|
pub impl @definition.Text for Code with fn print(value) {
  print_code(value)
}

///|
pub impl @definition.Text for MemoryImage with fn parse(text) {
  { values: parse_input(text), }
}

///|
pub impl @definition.Text for MemoryImage with fn print(value) {
  output(value.values)
}

///|
pub type Ainput = MemoryImage

///|
pub type Rinput = String

///|
pub type Routput = String

///|
pub type Foutput = MemoryImage

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
