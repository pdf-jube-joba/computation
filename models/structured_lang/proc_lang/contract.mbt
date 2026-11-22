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
  [
    @definition.titled("stmt", st(s.stmt)),
    @definition.titled(
      "global env",
      @a.print_env(@utils.env_sorted(s.global_env), trailing_newline=false),
    ),
    @definition.titled(
      "local env",
      @a.print_env(@utils.env_sorted(s.local_env), trailing_newline=false),
    ),
    @definition.titled("call stack depth", s.call_stack.length().to_string()),
  ]
}

///|
pub impl @definition.Text for Code with fn print(value) {
  "\\static " +
  @utils.join(value.statics, ", ") +
  ";\n" +
  @utils.join(
    value.procs.map(p => {
      p.name +
      "(" +
      @utils.join(p.params, ", ") +
      ") [ \\local " +
      @utils.join(p.locals, ", ") +
      " " +
      st(p.body) +
      " ]"
    }),
    "\n",
  )
}
