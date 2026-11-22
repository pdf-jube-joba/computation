///|
pub(all) enum Func {
  Zero
  Succ
  Proj(Int32, Int32)
  Comp(Func, Array[Func])
  Prim(Func, Func)
  Mu(Func)
}

///|
priv enum Process {
  Call(Func, Array[Process])
  Search(@utils.Nat, Array[@utils.Nat], Func, Process)
  Value(@utils.Nat)
}

///|
struct State {
  function_ : Func
  input : Array[@utils.Nat]
  process : Process
}

///|
pub fn arity(f : Func) -> Int32 {
  match f {
    Zero => 0
    Succ => 1
    Proj(n, _) => n
    Comp(_, args) => if args.is_empty() { 0 } else { arity(args[0]) }
    Prim(z, _) => arity(z) + 1
    Mu(f) => arity(f) - 1
  }
}

///|
pub fn validate(f : Func) -> Unit raise @utils.Fault {
  match f {
    Zero | Succ => ()
    Proj(n, i) => @utils.require(n > i && i >= 0, "Projection out of range")
    Comp(f, args) => {
      validate(f)
      for a in args {
        validate(a)
      }
      @utils.require(
        !args.is_empty() &&
        args.length() == arity(f) &&
        args.all(a => arity(a) == arity(args[0])),
        "Composition arity mismatch",
      )
    }
    Prim(z, s) => {
      validate(z)
      validate(s)
      @utils.require(
        arity(s) == arity(z) + 2,
        "Primitive recursion arity mismatch",
      )
    }
    Mu(f) => {
      validate(f)
      @utils.require(arity(f) > 0, "Mu requires a function of positive arity")
    }
  }
}

///|
pub fn parse_code(text : String) -> Func raise @utils.Fault {
  let p = @utils.create(text)
  let bindings : Map[String, Func] = Map([])
  fn func(p : @utils.Parser) -> Func raise @utils.Fault {
    match p.bump() {
      @utils.Keyword("ZERO") => Zero
      @utils.Keyword("SUCC") => Succ
      @utils.Keyword("PROJ") => {
        let g = p.group('[')
        let n = g.integer()
        g.expect(',')
        let i = g.integer()
        g.finish()
        Proj(n, i)
      }
      @utils.Keyword("COMP") => {
        let g = p.group('[')
        let f = func(g)
        g.expect(':')
        let args = []
        fn items(p : @utils.Parser) -> Unit raise @utils.Fault {
          while !p.eof() {
            if p.eat(',') {
              continue
            }
            args.push(func(p))
            if !p.eof() {
              p.expect(',')
            }
          }
        }
        if g.peek() == @utils.Symbol('(') {
          let inner = g.group('(')
          items(inner)
          ignore(g.eat(','))
          items(g)
        } else {
          args.push(func(g))
          if g.eat(',') {
            items(g)
          }
        }
        g.finish()
        Comp(f, args)
      }
      @utils.Keyword("PRIM") => {
        let g = p.group('[')
        g.expect_text("\\z:")
        let z = func(g)
        g.expect_text("\\s:")
        let s = func(g)
        g.finish()
        Prim(z, s)
      }
      @utils.Keyword("MUOP") => {
        let g = p.group('[')
        let f = func(g)
        g.finish()
        Mu(f)
      }
      @utils.Ident(name) =>
        match bindings.get(name) {
          Some(f) => f
          None => p.error("Unknown function: " + name)
        }
      _ => p.error("Expected function")
    }
  }
  while p.eat_keyword("let") {
    let name = p.ident()
    @utils.require(!bindings.contains(name), "Duplicate function name")
    p.expect('=')
    let f = func(p)
    p.expect('.')
    bindings[name] = f
  }
  let f = func(p)
  p.finish()
  validate(f)
  f
}

///|
pub fn print_code(f : Func) -> String {
  match f {
    Zero => "\\ZERO"
    Succ => "\\SUCC"
    Proj(n, i) => "\\PROJ[\{n},\{i}]"
    Comp(f, args) =>
      "\\COMP[" +
      print_code(f) +
      ": " +
      @utils.join(args.map(print_code), ", ") +
      "]"
    Prim(z, s) => "\\PRIM[\\z:" + print_code(z) + " \\s:" + print_code(s) + "]"
    Mu(f) => "\\MUOP[" + print_code(f) + "]"
  }
}

///|
pub fn parse_input(text : String) -> Array[@utils.Nat] raise @utils.Fault {
  let p = @utils.create(text)
  let g = p.group('(')
  p.finish()
  let args = []
  while !g.eof() {
    args.push(g.natural())
    if g.eof() {
      break
    }
    g.expect(',')
  }
  args
}

///|
fn make(function_ : Func, input : Ainput) -> State raise @utils.Fault {
  validate(function_)
  let input = input.values
  @utils.require(
    input.length() == arity(function_),
    "Function argument count mismatch",
  )
  { function_, input, process: Call(function_, input.map(n => Value(n))), }
}

///|
fn evaluate(p : Process) -> Process? raise @utils.Fault {
  match p {
    Value(_) => None
    Call(f, args) => {
      for i, a in args {
        match evaluate(a) {
          Some(a) => {
            let args = args.copy()
            args[i] = a
            return Some(Call(f, args))
          }
          None => ()
        }
      }
      let numbers = args.map(a => {
        match a {
          Value(n) => n
          _ => @utils.error("Invalid recursive-function process")
        }
      })
      let next = match f {
        Zero => {
          @utils.require(numbers.is_empty(), "Invalid zero arguments")
          Value(@utils.zero())
        }
        Succ => {
          @utils.require(numbers.length() == 1, "Invalid successor arguments")
          Value(numbers[0] + @utils.one())
        }
        Proj(_, i) => {
          @utils.require(
            i >= 0 && i < numbers.length(),
            "Projection out of range",
          )
          Value(numbers[i])
        }
        Comp(outer, inners) => Call(outer, inners.map(f => Call(f, args)))
        Prim(z, s) => {
          @utils.require(!numbers.is_empty(), "Invalid primitive arguments")
          let tail = numbers[1:].to_owned()
          if numbers[0].is_zero() {
            Call(z, tail.map(n => Value(n)))
          } else {
            let ns = [@utils.pred(numbers[0])]
            ns.append(tail)
            let args = [Call(f, ns.map(n => Value(n)))]
            args.append(ns.map(n => Value(n)))
            Call(s, args)
          }
        }
        Mu(f) => {
          let ns = [@utils.zero()]
          ns.append(numbers)
          Search(@utils.zero(), numbers, f, Call(f, ns.map(n => Value(n))))
        }
      }
      Some(next)
    }
    Search(index, args, f, p) =>
      match evaluate(p) {
        Some(next) => Some(Search(index, args, f, next))
        None =>
          match p {
            Value(n) =>
              if n.is_zero() {
                Some(Value(index))
              } else {
                let index = index + @utils.one()
                let ns = [index]
                ns.append(args)
                Some(Search(index, args, f, Call(f, ns.map(n => Value(n)))))
              }
            _ => @utils.error("Invalid search process")
          }
      }
  }
}

///|
fn step(
  s : State,
  _input : Rinput,
) -> @definition.Transition[State, Routput, Foutput] raise @utils.Fault {
  match evaluate(s.process) {
    Some(process) => @definition.Continue({ ..s, process, }, ())
    None =>
      match s.process {
        Value(n) => @definition.Halt(n)
        _ => @utils.error("Invalid recursive-function process")
      }
  }
}

///|
fn fj(f : Func) -> Json {
  match f {
    Zero => @utils.tag("Zero", [])
    Succ => @utils.tag("Succ", [])
    Proj(n, i) => @utils.tag("Proj", [@utils.jint(n), @utils.jint(i)])
    Comp(f, args) => @utils.tag("Comp", [fj(f), @utils.jarray(args.map(fj))])
    Prim(z, s) => @utils.tag("Prim", [fj(z), fj(s)])
    Mu(f) => @utils.tag("Mu", [fj(f)])
  }
}

///|
fn fd(j : Json) -> Func raise @utils.Fault {
  fn arg(i : Int32) -> Json raise @utils.Fault {
    @utils.tag_arg(j, i)
  }
  match @utils.tag_name(j) {
    "Zero" => Zero
    "Succ" => Succ
    "Proj" => Proj(@utils.int(arg(0)), @utils.int(arg(1)))
    "Comp" => Comp(fd(arg(0)), @utils.array(arg(1)).map(fd))
    "Prim" => Prim(fd(arg(0)), fd(arg(1)))
    "Mu" => Mu(fd(arg(0)))
    _ => @utils.error("Invalid function snapshot")
  }
}

///|
fn pj(p : Process) -> Json {
  match p {
    Value(n) => @utils.tag("Result", [@utils.jstr(n.to_string())])
    Call(f, args) => @utils.tag("Call", [fj(f), @utils.jarray(args.map(pj))])
    Search(i, args, f, p) =>
      @utils.tag("Search", [
        @utils.jstr(i.to_string()),
        @utils.jarray(args.map(n => @utils.jstr(n.to_string()))),
        fj(f),
        pj(p),
      ])
  }
}

///|
fn pd(j : Json) -> Process raise @utils.Fault {
  fn arg(i : Int32) -> Json raise @utils.Fault {
    @utils.tag_arg(j, i)
  }
  match @utils.tag_name(j) {
    "Result" => Value(@utils.json_nat(arg(0)))
    "Call" => {
      let f = fd(arg(0))
      validate(f)
      let args = @utils.array(arg(1)).map(pd)
      @utils.require(args.length() == arity(f), "Invalid process arity")
      Call(f, args)
    }
    "Search" => {
      let f = fd(arg(2))
      validate(f)
      let args = @utils.array(arg(1)).map(@utils.json_nat)
      @utils.require(args.length() + 1 == arity(f), "Invalid search arity")
      Search(@utils.json_nat(arg(0)), args, f, pd(arg(3)))
    }
    _ => @utils.error("Invalid process snapshot")
  }
}

///|
fn encode(s : State) -> Json {
  @utils.object([
    ("function_", fj(s.function_)),
    ("input", @utils.jarray(s.input.map(n => @utils.jstr(n.to_string())))),
    ("process", pj(s.process)),
  ])
}

///|
fn decode(j : Json) -> State raise @utils.Fault {
  let function_ = fd(@utils.field(j, "function_"))
  validate(function_)
  let input = @utils.array(@utils.field(j, "input")).map(@utils.json_nat)
  @utils.require(
    input.length() == arity(function_),
    "Invalid snapshot input arity",
  )
  { function_, input, process: pd(@utils.field(j, "process")), }
}

///|
fn pt(p : Process) -> String {
  match p {
    Value(n) => n.to_string()
    Call(f, args) => print_code(f) + "(" + @utils.join(args.map(pt), ", ") + ")"
    Search(n, _, _, p) => "Mu[" + n.to_string() + "](" + pt(p) + ")"
  }
}

///|
pub fn model() -> @definition.Model[
  Code,
  Ainput,
  Rinput,
  Routput,
  Foutput,
  State,
] {
  @definition.Model(make=(code, input) => make(code, input), step=(state, input) => {
    step(state, input)
  })
}
