///|
pub(all) struct Environment {
  values : Array[(String, @utils.Nat)]
}

///|
pub impl @definition.Text for Stmt with fn parse(text) {
  parse_code(text)
}

///|
pub impl @definition.Text for Stmt with fn print(value) {
  st(value)
}

///|
pub impl @definition.Text for Environment with fn parse(text) {
  let mut env = []
  for line in text.split("\n") {
    if !line.trim().is_empty() {
      let parts = line.split("=").collect()
      @utils.require(parts.length() == 2, "Invalid env line")
      env = set(
        env,
        @utils.identifier(parts[0].trim().to_owned()),
        @utils.nat(parts[1].to_owned()),
      )
    }
  }
  { values: env, }
}

///|
pub impl @definition.Text for Environment with fn print(value) {
  @utils.join(
    value.values.map(pair => pair.0 + " = " + pair.1.to_string() + "\n"),
    "",
  )
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
      s.env.map(pair => [pair.0, pair.1.to_string()]),
    ),
  ]
}
