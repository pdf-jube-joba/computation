///|
struct Program {
  functions : Array[FunctionDef]
}

///|
pub impl @definition.Text for Program with fn parse(text) {
  { functions: parse(text), }
}

///|
pub impl @definition.Text for Environment with fn print(value) {
  output(value)
}

///|
pub type Code = Program

///|
pub type Ainput = UInt32

///|
pub type Rinput = UInt32

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

///|
pub impl @definition.Text for Program with fn print(value) {
  @utils.join(
    value.functions.map(f => {
      "\\fn " + f.name + " { " + @utils.join(f.body.map(stext), "; ") + " }"
    }),
    "\n",
  )
}

///|
pub impl @definition.Text for Environment with fn parse(text) {
  let p = @utils.create(text)
  let mut vars = []
  let mut task_ids = []
  while !p.eof() {
    if p.eat('$') {
      let name = p.ident()
      p.expect('=')
      let id = if p.eat_keyword("none") { None } else { Some(p.integer()) }
      task_ids = @utils.env_set(task_ids, name, id)
    } else {
      let name = p.ident()
      p.expect('=')
      vars = @utils.env_set(vars, name, p.natural())
    }
    ignore(p.eat(';'))
  }
  { vars: @utils.env_sorted(vars), task_ids: @utils.env_sorted(task_ids), }
}
