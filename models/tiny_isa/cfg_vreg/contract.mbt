///|
pub(all) struct Memory {
  values : Array[@utils.Nat]
}

///|
pub impl @definition.Text for Code with fn parse(text) {
  parse(text)
}

///|
pub impl @definition.Text for Memory with fn parse(text) {
  { values: @utils.natural_list(text), }
}

///|
pub impl @definition.Text for Memory with fn print(value) {
  @utils.natural_list_text(value.values)
}

///|
pub type Ainput = Memory

///|
pub type Rinput = Unit

///|
pub type Routput = Unit

///|
pub type Foutput = Memory

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
pub impl @definition.Text for Code with fn print(value) {
  @utils.join(
    value.statics.map(s => "@" + s.label + " " + s.value.to_string() + "\n"),
    "",
  ) +
  @utils.join(
    value.blocks.map(b => {
      "@" +
      b.label +
      " {\n" +
      @utils.join(b.stmts.map(s => stmt_text(s) + "\n"), "") +
      @utils.join(
        b.cont.ifs.map(j => {
          let condition = match j.cond {
            Lt(a, b) => value_text(a) + " < " + value_text(b)
            Eq(a, b) => value_text(a) + " = " + value_text(b)
            Gt(a, b) => value_text(a) + " > " + value_text(b)
          }
          "\\if " + condition + " \\then " + addr_text(j.target) + ";\n"
        }),
        "",
      ) +
      "\\goto " +
      addr_text(b.cont.jump) +
      ";\n}"
    }),
    "\n",
  )
}
