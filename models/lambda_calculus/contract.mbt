///|
pub(all) struct Ainput {
  values : Array[Term]
}

///|
pub impl @definition.Text for Term with fn parse(text) {
  parse_code(text)
}

///|
pub impl @definition.Text for Term with fn print(value) {
  print_code(value)
}

///|
pub impl @definition.Text for Ainput with fn parse(text) {
  { values: parse_input(text), }
}

///|
pub impl @definition.Text for Ainput with fn print(value) {
  "(" + @utils.join(value.values.map(print_code), ", ") + ")"
}

///|
pub type Code = Term

///|
pub type Rinput = UInt32

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
pub impl @definition.Render for Term with fn render(t) {
  [
    @definition.titled("term", print_code(t)),
    @definition.text_table(
      "redexes",
      ["index", "term"],
      redexes(t).mapi((i, t) => [i.to_string(), print_code(t)]),
    ),
  ]
}
