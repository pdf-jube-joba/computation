///|
pub impl @definition.Text for Net with fn parse(text) {
  parse(text)
}

///|
pub impl @definition.Text for Net with fn print(value) {
  print(value)
}

///|
pub type Code = Net

///|
pub type Ainput = Unit

///|
pub type Rinput = Selection

///|
pub type Routput = String

///|
pub type Foutput = Net

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
pub(all) struct Selection {
  pair : (String, String)?
}

///|
pub impl @definition.Text for Selection with fn parse(text) {
  let pair = if text.trim().is_empty() {
    None
  } else {
    let p = @utils.create(text)
    let a = p.ident()
    let b = p.ident()
    p.finish()
    Some((a, b))
  }
  { pair, }
}

///|
pub impl @definition.Text for Selection with fn print(value) {
  match value.pair {
    None => ""
    Some((a, b)) => a + " " + b
  }
}
