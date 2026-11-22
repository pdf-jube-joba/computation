///|
struct Signals {
  values : Signal
}

///|
pub impl @definition.Text for Circuit with fn parse(text) {
  parse_code(text)
}

///|
pub impl @definition.Text for Signals with fn parse(text) {
  { values: read_signal(text), }
}

///|
pub impl @definition.Text for Signals with fn print(value) {
  signal_text(value.values)
}

///|
pub type Code = Circuit

///|
pub type Ainput = Unit

///|
pub type Rinput = Signals

///|
pub type Routput = Signals

///|
pub type Foutput = Unit

///|
pub impl @definition.Snapshot for Circuit with fn encode(value) {
  json_circuit(value)
}

///|
pub impl @definition.Snapshot for Circuit with fn decode(value) {
  decode(value)
}

///|
pub impl @definition.Render for Circuit with fn render(value) {
  render(value)
}

///|
pub impl @definition.Text for Circuit with fn print(value) {
  json_circuit(value).stringify()
}
