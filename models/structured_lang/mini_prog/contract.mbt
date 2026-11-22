///|
pub(all) struct Ainput {
  values : Array[@utils.Nat]
}

///|
pub impl @definition.Text for Code with fn parse(text) {
  parse_code(text)
}

///|
pub impl @definition.Text for Ainput with fn parse(text) {
  let p = @utils.create(text)
  let g = p.group('(')
  let args = []
  while !g.eof() {
    if g.eat(',') {
      continue
    }
    ignore(g.eat('+'))
    args.push(g.natural())
    if !g.eof() {
      g.expect(',')
    }
  }
  g.finish()
  p.finish()
  { values: args, }
}

///|
pub type Rinput = Unit

///|
pub type Routput = Unit

///|
pub type Foutput = @utils.Nat

///|
pub impl @definition.Snapshot for State with fn encode(value) {
  json_state(value)
}

///|
pub impl @definition.Snapshot for State with fn decode(value) {
  decode(value)
}

///|
pub impl @definition.Render for State with fn render(value) {
  render(value)
}

///|
pub impl @definition.Text for Ainput with fn print(value) {
  "(" + @utils.join(value.values.map(n => n.to_string()), ", ") + ")"
}

///|
pub impl @definition.Text for Code with fn print(value) {
  @utils.join(
    value.statics.map(s => "\\static " + s.name + ": " + type_text(s.ty) + "\n"),
    "",
  ) +
  @utils.join(
    value.functions.map(f => "\\fn " + f.name + block_text(f.block)),
    "\n",
  )
}
