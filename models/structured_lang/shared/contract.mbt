///|
pub(all) struct Environment {
  values : Env
}

///|
pub impl @definition.Text for Environment with fn parse(text) {
  { values: parse_env(text), }
}

///|
pub impl @definition.Text for Environment with fn print(value) {
  print_env(value.values)
}
