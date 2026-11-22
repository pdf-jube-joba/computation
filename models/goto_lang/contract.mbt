///|
pub(all) struct Code {
  commands : Array[Command]
}

///|
pub(all) struct Environment {
  values : Array[(String, @utils.Nat)]
}

///|
pub impl @definition.Text for Code with fn parse(text) {
  { commands: parse_code(text), }
}

///|
pub impl @definition.Text for Code with fn print(value) {
  @utils.join(value.commands.map(command => print_command(command)), "\n")
}

///|
pub impl @definition.Text for Environment with fn parse(text) {
  { values: parse_env(text), }
}

///|
pub impl @definition.Text for Environment with fn print(value) {
  print_env(value.values)
}

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
pub impl @definition.Render for State with fn render(value) {
  render(value)
}
