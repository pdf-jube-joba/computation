///|
pub(all) struct Environment {
  values : Array[(String, Array[String])]
}

///|
pub impl @definition.Text for Stmt with fn parse(text) {
  let p = @utils.create(text, character_literals=true)
  let code = statement(p)
  p.finish()
  code
}

///|
pub impl @definition.Text for Stmt with fn print(value) {
  st(value)
}

///|
pub impl @definition.Text for Environment with fn parse(text) {
  { values: @utils.parse_env(text, @utils.parse_chars), }
}

///|
pub impl @definition.Text for Environment with fn print(value) {
  @utils.print_env(value.values, @utils.chars_text)
}

///|
pub type Code = Stmt

///|
pub type Ainput = Environment

///|
pub type Rinput = Unit

///|
pub type Routput = Unit

///|
pub type Foutput = Environment

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
    @definition.titled("stmt", st(s.stmt)),
    @definition.text_table(
      "env",
      ["var", "value"],
      s.env.map(pair => [pair.0, @utils.chars_text(pair.1)]),
    ),
  ]
}
