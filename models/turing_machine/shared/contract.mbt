///|
pub(all) struct Tape {
  value : @utils.Tape[String]
}

///|
pub(all) struct Tapes {
  values : Array[@utils.Tape[String]]
}

///|
pub(all) struct MultiCode {
  definition : Definition
}

///|
struct MultiState {
  value : State
}

///|
pub type Code = Definition

///|
pub type Ainput = Tape

///|
pub type Rinput = Unit

///|
pub type Routput = Unit

///|
pub type Foutput = Tape

///|
pub impl @definition.Text for Definition with fn parse(text) {
  parse_code(text, false)
}

///|
pub impl @definition.Text for Definition with fn print(value) {
  print_code(value)
}

///|
pub impl @definition.Text for MultiCode with fn parse(text) {
  { definition: parse_code(text, true), }
}

///|
pub impl @definition.Text for Tape with fn parse(text) {
  { value: @utils.parse_tape(text, sign), }
}

///|
pub impl @definition.Text for Tape with fn print(value) {
  value.value.text(s => s)
}

///|
pub impl @definition.Text for Tapes with fn parse(text) {
  {
    values: text
    .trim()
    .to_owned()
    .split("\n")
    .map(s => s.to_owned())
    .collect()
    .map(s => @utils.parse_tape(s, sign)),
  }
}

///|
pub impl @definition.Text for Tapes with fn print(value) {
  @utils.join(value.values.map(t => t.text(s => s)), "\n")
}

///|
pub impl @definition.Snapshot for State with fn encode(value) {
  encode(value, false)
}

///|
pub impl @definition.Snapshot for State with fn decode(value) {
  decode(value, false)
}

///|
pub impl @definition.Render for State with fn render(value) {
  render(value, false)
}

///|
pub impl @definition.Snapshot for MultiState with fn encode(value) {
  encode(value.value, true)
}

///|
pub impl @definition.Snapshot for MultiState with fn decode(value) {
  { value: decode(value, true), }
}

///|
pub impl @definition.Render for MultiState with fn render(value) {
  render(value.value, true)
}

///|
pub impl @definition.Text for MultiCode with fn print(value) {
  "tapes: " +
  value.definition.count.to_string() +
  "\nalphabet: " +
  @utils.join(value.definition.alphabet, ",") +
  "\n" +
  print_code(value.definition)
}
