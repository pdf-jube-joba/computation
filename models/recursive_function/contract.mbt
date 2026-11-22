///|
pub(all) struct Ainput {
  values : Array[@utils.Nat]
}

///|
pub impl @definition.Text for Func with fn parse(text) {
  parse_code(text)
}

///|
pub impl @definition.Text for Func with fn print(value) {
  print_code(value)
}

///|
pub impl @definition.Text for Ainput with fn parse(text) {
  { values: parse_input(text), }
}

///|
pub impl @definition.Text for Ainput with fn print(value) {
  "(" + @utils.join(value.values.map(n => n.to_string()), ", ") + ")"
}

///|
pub type Code = Func

///|
pub type Rinput = Unit

///|
pub type Routput = Unit

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
pub impl @definition.Render for State with fn render(s) {
  [
    @definition.titled("function", print_code(s.function_)),
    @definition.text_table(
      "input",
      ["arg", "value"],
      s.input.mapi((i, n) => [i.to_string(), n.to_string()]),
    ),
    @definition.titled("process", pt(s.process)),
  ]
}
