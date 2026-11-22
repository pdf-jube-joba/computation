///|
struct StatementState {
  value : State
}

///|
pub type Code = @s.Expr

///|
pub type Ainput = @s.Inputs

///|
pub type Rinput = Unit

///|
pub type Routput = @s.Output

///|
pub type Foutput = @utils.Nat

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
pub impl @definition.Snapshot for StatementState with fn encode(value) {
  encode(value.value, true)
}

///|
pub impl @definition.Snapshot for StatementState with fn decode(value) {
  { value: decode(value, true), }
}

///|
pub impl @definition.Render for StatementState with fn render(value) {
  render(value.value, true)
}
