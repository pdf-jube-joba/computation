///|
pub impl @definition.Text for Code with fn parse(text) {
  parse_code(text)
}

///|
pub type Ainput = @a.Environment

///|
pub type Rinput = Unit

///|
pub type Routput = String

///|
pub type Foutput = @a.Environment

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
  let stack = s.stack.rev()
  [
    @definition.titled("exec", et(s.exec)),
    @definition.titled("env", @a.print_env(s.env, trailing_newline=false)),
    @definition.titled(
      "stack",
      @utils.join(
        stack.map(f => {
          match f {
            SeqFrame(b) => "seq " + st(b)
            LoopFrame(l, _, n) => "loop " + l + " -> " + n
          }
        }),
        "\n",
      ),
    ),
  ]
}

///|
pub impl @definition.Text for Code with fn print(value) {
  "\\static " + @utils.join(value.statics, ", ") + "; " + st(value.body)
}
