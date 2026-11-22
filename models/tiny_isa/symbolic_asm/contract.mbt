///|
struct Code {
  items : Array[Item]
}

///|
pub impl @definition.Text for Code with fn parse(text) {
  { items: parse(text), }
}

///|
pub type Ainput = @machine.Image

///|
pub type Rinput = Unit

///|
pub type Routput = Unit

///|
pub type Foutput = @machine.Image

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
    value.items.map(item => {
      match item {
        Directive(Text) => "\\text"
        Directive(Data) => "\\data"
        Directive(Equ(name, n)) => "\\equ " + name + " " + n.to_string()
        LabelDef(name) => "@" + name + ":"
        DataValue(v) => raw_text(v)
        Inst(i) =>
          "\\" +
          i.mnemonic +
          (if i.cond_flag { " \\f" } else { "" }) +
          @utils.join(
            i.operands.map(operand => {
              " " +
              (match operand {
                Reg(n) => "%r" + n.to_string()
                Value(v) => raw_text(v)
              })
            }),
            "",
          )
      }
    }),
    "\n",
  )
}
