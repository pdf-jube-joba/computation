///|
enum Term {
  S
  K
  I
  Var(String)
  App(Term, Term)
}

///|
fn read(p : @utils.Parser) -> Term raise @utils.Fault {
  fn atom(p : @utils.Parser) -> Array[Term] raise @utils.Fault {
    match p.peek() {
      @utils.Symbol('(') => {
        let inner = p.group('(')
        let term = read(inner)
        inner.finish()
        [term]
      }
      @utils.Ident(word) => {
        ignore(p.bump())
        if word.all(c => c == 'S' || c == 'K' || c == 'I') {
          word
          .to_array()
          .map(c => {
            match c {
              'S' => S
              'K' => K
              _ => I
            }
          })
        } else {
          [Var(word)]
        }
      }
      _ => p.error("Expected a combinator, variable, or group")
    }
  }
  let first = atom(p)
  let mut term = first[0]
  for item in first[1:] {
    term = App(term, item)
  }
  while !p.eof() {
    for item in atom(p) {
      term = App(term, item)
    }
  }
  term
}

///|
fn parse(text : String) -> Term raise @utils.Fault {
  let p = @utils.create(text)
  read(p)
}

///|
fn print(term : Term) -> String {
  match term {
    S => "S"
    K => "K"
    I => "I"
    Var(name) => name
    App(a, b) =>
      print(a) +
      " " +
      (match b {
        App(_, _) => "(" + print(b) + ")"
        _ => print(b)
      })
  }
}

///|
fn reduce(term : Term) -> Term? {
  match term {
    App(I, x) => Some(x)
    App(App(K, x), _) => Some(x)
    App(App(App(S, x), y), z) => Some(App(App(x, z), App(y, z)))
    App(a, b) =>
      match reduce(a) {
        Some(next) => Some(App(next, b))
        None => reduce(b).map(next => App(a, next))
      }
    _ => None
  }
}

///|
fn make(code : Term, _input : Unit) -> Term {
  code
}

///|
fn step(
  term : Term,
  _input : Rinput,
) -> @definition.Transition[Term, Routput, Foutput] {
  match reduce(term) {
    None => @definition.Halt(term)
    Some(next) => @definition.Continue(next, ())
  }
}

///|
fn encode(term : Term) -> Json {
  match term {
    S => @utils.tag("S", [])
    K => @utils.tag("K", [])
    I => @utils.tag("I", [])
    Var(name) => @utils.tag("Var", [@utils.jstr(name)])
    App(a, b) => @utils.tag("App", [encode(a), encode(b)])
  }
}

///|
fn decode(value : Json) -> Term raise @utils.Fault {
  match @utils.tag_name(value) {
    "S" => S
    "K" => K
    "I" => I
    "Var" => {
      let name = @utils.identifier(@utils.string(@utils.tag_arg(value, 0)))
      @utils.require(
        !name.all(c => c == 'S' || c == 'K' || c == 'I'),
        "Combinators cannot be variable names",
      )
      Var(name)
    }
    "App" =>
      App(decode(@utils.tag_arg(value, 0)), decode(@utils.tag_arg(value, 1)))
    _ => @utils.error("Invalid combinator snapshot")
  }
}

///|
fn render(term : Term) -> Array[@definition.Block] {
  [@definition.titled("term", print(term))]
}

///|
pub fn model() -> @definition.Model[
  Code,
  Ainput,
  Rinput,
  Routput,
  Foutput,
  Term,
] {
  @definition.Model(make=(code, input) => make(code, input), step=(state, input) => {
    step(state, input)
  })
}
