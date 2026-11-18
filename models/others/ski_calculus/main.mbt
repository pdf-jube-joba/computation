///|
priv enum Term {
  S
  K
  I
  Var(String)
  App(Term, Term)
}

///|
fn read(p : @p.Parser) -> Term raise @base.Fault {
  fn atom(p : @p.Parser) -> Array[Term] raise @base.Fault {
    match p.peek() {
      @p.Symbol('(') => {
        let inner = p.group('(')
        let term = read(inner)
        inner.finish()
        [term]
      }
      @p.Ident(word) => {
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
fn parse(text : String) -> Term raise @base.Fault {
  let p = @p.create(text)
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
fn make(code : String, input : String) -> Term raise @base.Fault {
  @base.unit_input(input)
  parse(code)
}

///|
fn step(
  term : Term,
  input : String,
) -> @runtime.Transition[Term] raise @base.Fault {
  @base.unit_input(input)
  match reduce(term) {
    None => @runtime.Halt(print(term))
    Some(next) => @runtime.Continue(next, "")
  }
}

///|
fn encode(term : Term) -> Json {
  match term {
    S => @base.tag("S", [])
    K => @base.tag("K", [])
    I => @base.tag("I", [])
    Var(name) => @base.tag("Var", [@base.jstr(name)])
    App(a, b) => @base.tag("App", [encode(a), encode(b)])
  }
}

///|
fn decode(value : Json) -> Term raise @base.Fault {
  match @base.tag_name(value) {
    "S" => S
    "K" => K
    "I" => I
    "Var" => {
      let name = @base.identifier(@base.string(@base.tag_arg(value, 0)))
      @base.require(
        !name.all(c => c == 'S' || c == 'K' || c == 'I'),
        "Combinators cannot be variable names",
      )
      Var(name)
    }
    "App" =>
      App(decode(@base.tag_arg(value, 0)), decode(@base.tag_arg(value, 1)))
    _ => @base.error("Invalid combinator snapshot")
  }
}

///|
fn render(term : Term) -> Json {
  @base.jarray([@base.titled("term", print(term))])
}

///|
pub fn model() -> @runtime.Model {
  @runtime.model(make, step, encode, decode, render)
}
