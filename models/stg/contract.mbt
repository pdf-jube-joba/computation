///|
struct Code {
  bindings : Bindings
}

///|
pub impl @definition.Text for Code with fn parse(text) {
  { bindings: parse(text), }
}

///|
pub impl @definition.Text for ResultValue with fn print(value) {
  result_text(value)
}

///|
pub type Rinput = Unit

///|
pub type Routput = Unit

///|
pub type Foutput = ResultValue

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

///|
pub type Ainput = Unit

///|
pub impl @definition.Text for Code with fn print(value) {
  code_text(value.bindings)
}

///|
pub impl @definition.Text for ResultValue with fn parse(text) {
  let p = @utils.create(text)
  let value = if p.eat_keyword("con") {
    let name = p.ident()
    let g = p.group('(')
    let args = []
    while !g.eof() {
      args.push(
        if g.eat('@') {
          Addr(g.integer())
        } else {
          Integer(g.natural())
        },
      )
      if !g.eof() {
        g.expect(',')
      }
    }
    Data(name, args)
  } else if p.eat_keyword("function") {
    p.expect('@')
    Function(p.integer())
  } else {
    Number(p.natural())
  }
  p.finish()
  value
}
