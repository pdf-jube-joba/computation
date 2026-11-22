///|
pub(all) struct Inputs {
  values : Array[@utils.Nat]
}

///|
pub(all) struct StatementCode {
  expression : Expr
}

///|
pub impl @definition.Text for Expr with fn parse(text) {
  parse(text)
}

///|
pub impl @definition.Text for Expr with fn print(value) {
  print(value)
}

///|
pub impl @definition.Text for StatementCode with fn parse(text) {
  { expression: parse(text, statements=true), }
}

///|
pub impl @definition.Text for StatementCode with fn print(value) {
  print(value.expression)
}

///|
pub impl @definition.Text for Inputs with fn parse(text) {
  { values: inputs(text), }
}

///|
pub impl @definition.Text for Inputs with fn print(value) {
  "(" + @utils.join(value.values.map(n => n.to_string()), ", ") + ")"
}

///|
pub(all) enum Output {
  Silent
  Printed(@utils.Nat)
}

///|
pub impl @definition.Text for Output with fn parse(text) {
  if text.trim().is_empty() {
    Silent
  } else {
    Printed(@utils.nat(text))
  }
}

///|
pub impl @definition.Text for Output with fn print(value) {
  match value {
    Silent => ""
    Printed(n) => n.to_string()
  }
}
