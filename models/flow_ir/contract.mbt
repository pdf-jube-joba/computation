///|
pub(all) struct Environment {
  values : Array[(String, FlowValue)]
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
pub impl @definition.Text for Environment with fn parse(text) {
  { values: parse_input(text), }
}

///|
pub impl @definition.Text for Environment with fn print(value) {
  @utils.print_env(@utils.env_sorted(value.values), flow_text)
}

///|
pub type Ainput = Environment

///|
pub type Rinput = String

///|
pub type Routput = String

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
pub impl @definition.Render for State with fn render(value) {
  render(value)
}
