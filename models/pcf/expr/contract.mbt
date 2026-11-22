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
  encode(value)
}

///|
pub impl @definition.Snapshot for State with fn decode(value) {
  decode(value)
}

///|
pub impl @definition.Render for State with fn render(s) {
  [
    @definition.titled("code", @s.print(s.code)),
    @definition.titled("current", @s.print(s.expr)),
  ]
}
