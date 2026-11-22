///|
pub impl @definition.Text for Term with fn parse(text) {
  let p = @utils.create(text)
  let code = read(p)
  p.finish()
  @utils.normalize({ view, variable: x => Var(x), }, code)
}

///|
pub impl @definition.Text for Term with fn print(value) {
  pretty(value)
}

///|
pub type Code = Term

///|
pub type Ainput = @pcf.Inputs

///|
pub type Rinput = Unit

///|
pub type Routput = Unit

///|
pub type Foutput = Term

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
    @definition.titled("code", pretty(s.code)),
    @definition.titled("current", pretty(s.current)),
  ]
}
