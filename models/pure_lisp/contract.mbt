///|
pub impl @definition.Text for Sexp with fn parse(text) {
  parse(text)
}

///|
pub impl @definition.Text for Sexp with fn print(value) {
  print(value)
}

///|
pub type Code = Sexp

///|
pub type Ainput = Unit

///|
pub type Rinput = Unit

///|
pub type Routput = Unit

///|
pub type Foutput = Sexp

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
  let (mode, focus, env) = match s.control {
    Eval(e, env) => ("eval", print(e), env)
    Return(v, env) => ("return", print(v), env)
    Apply(f, args, env) => {
      let xs = [f]
      xs.append(args)
      ("apply", print(list(xs)), env)
    }
  }
  [
    @definition.titled("code", print(s.code)),
    @definition.text_table("state", ["field", "value"], [
      ["mode", mode],
      ["focus", focus],
    ]),
    @definition.text_table(
      "env",
      ["name", "value"],
      env.map(p => [p.0, print(p.1)]),
    ),
    @definition.text_table(
      "kont",
      ["depth", "frame"],
      s.kont.mapi((i, f) => [i.to_string(), fj(f).stringify()]),
    ),
  ]
}
