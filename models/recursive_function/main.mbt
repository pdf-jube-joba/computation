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
  Search(@base.Nat, Array[@base.Nat], Func, Process)
  Value(@base.Nat)
}

///|
priv struct State {
  function_ : Func
  input : Array[@base.Nat]
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
pub fn validate(f : Func) -> Unit raise @base.Fault {
  match f {
    Zero | Succ => ()
    Proj(n, i) => @base.require(n > i && i >= 0, "Projection out of range")
    Comp(f, args) => {
      validate(f)
      for a in args {
        validate(a)
      }
      @base.require(
        !args.is_empty() &&
        args.length() == arity(f) &&
        args.all(a => arity(a) == arity(args[0])),
        "Composition arity mismatch",
      )
    }
    Prim(z, s) => {
      validate(z)
      validate(s)
      @base.require(
        arity(s) == arity(z) + 2,
        "Primitive recursion arity mismatch",
      )
    }
    Mu(f) => {
      validate(f)
      @base.require(arity(f) > 0, "Mu requires a function of positive arity")
    }
  }
}

///|
pub fn parse_code(text : String) -> Func raise @base.Fault {
  let p = @p.create(text)
  let bindings : Map[String, Func] = Map([])
  fn func(p : @p.Parser) -> Func raise @base.Fault {
    match p.bump() {
      @p.Keyword("ZERO") => Zero
      @p.Keyword("SUCC") => Succ
      @p.Keyword("PROJ") => {
        let g = p.group('[')
        let n = g.integer()
        g.expect(',')
        let i = g.integer()
        g.finish()
        Proj(n, i)
      }
      @p.Keyword("COMP") => {
        let g = p.group('[')
        let f = func(g)
        g.expect(':')
        let args = []
        fn items(p : @p.Parser) -> Unit raise @base.Fault {
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
        if g.peek() == @p.Symbol('(') {
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
      @p.Keyword("PRIM") => {
        let g = p.group('[')
        g.expect_text("\\z:")
        let z = func(g)
        g.expect_text("\\s:")
        let s = func(g)
        g.finish()
        Prim(z, s)
      }
      @p.Keyword("MUOP") => {
        let g = p.group('[')
        let f = func(g)
        g.finish()
        Mu(f)
      }
      @p.Ident(name) =>
        match bindings.get(name) {
          Some(f) => f
          None => p.error("Unknown function: " + name)
        }
      _ => p.error("Expected function")
    }
  }
  while p.eat_keyword("let") {
    let name = p.ident()
    @base.require(!bindings.contains(name), "Duplicate function name")
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
      @base.join(args.map(print_code), ", ") +
      "]"
    Prim(z, s) => "\\PRIM[\\z:" + print_code(z) + " \\s:" + print_code(s) + "]"
    Mu(f) => "\\MUOP[" + print_code(f) + "]"
  }
}

///|
pub fn parse_input(text : String) -> Array[@base.Nat] raise @base.Fault {
  let p = @p.create(text)
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
fn make(code : String, input : String) -> State raise @base.Fault {
  let function_ = parse_code(code)
  let input = parse_input(input)
  @base.require(
    input.length() == arity(function_),
    "Function argument count mismatch",
  )
  { function_, input, process: Call(function_, input.map(n => Value(n))), }
}

///|
fn evaluate(p : Process) -> Process? raise @base.Fault {
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
          _ => @base.error("Invalid recursive-function process")
        }
      })
      let next = match f {
        Zero => {
          @base.require(numbers.is_empty(), "Invalid zero arguments")
          Value(@base.zero())
        }
        Succ => {
          @base.require(numbers.length() == 1, "Invalid successor arguments")
          Value(numbers[0] + @base.one())
        }
        Proj(_, i) => {
          @base.require(
            i >= 0 && i < numbers.length(),
            "Projection out of range",
          )
          Value(numbers[i])
        }
        Comp(outer, inners) => Call(outer, inners.map(f => Call(f, args)))
        Prim(z, s) => {
          @base.require(!numbers.is_empty(), "Invalid primitive arguments")
          let tail = numbers[1:].to_owned()
          if numbers[0].is_zero() {
            Call(z, tail.map(n => Value(n)))
          } else {
            let ns = [@base.pred(numbers[0])]
            ns.append(tail)
            let args = [Call(f, ns.map(n => Value(n)))]
            args.append(ns.map(n => Value(n)))
            Call(s, args)
          }
        }
        Mu(f) => {
          let ns = [@base.zero()]
          ns.append(numbers)
          Search(@base.zero(), numbers, f, Call(f, ns.map(n => Value(n))))
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
                let index = index + @base.one()
                let ns = [index]
                ns.append(args)
                Some(Search(index, args, f, Call(f, ns.map(n => Value(n)))))
              }
            _ => @base.error("Invalid search process")
          }
      }
  }
}

///|
fn step(
  s : State,
  input : String,
) -> @runtime.Transition[State] raise @base.Fault {
  @base.unit_input(input)
  match evaluate(s.process) {
    Some(process) => @runtime.Continue({ ..s, process, }, "")
    None =>
      match s.process {
        Value(n) => @runtime.Halt(n.to_string())
        _ => @base.error("Invalid recursive-function process")
      }
  }
}

///|
fn fj(f : Func) -> Json {
  match f {
    Zero => @base.tag("Zero", [])
    Succ => @base.tag("Succ", [])
    Proj(n, i) => @base.tag("Proj", [@base.jint(n), @base.jint(i)])
    Comp(f, args) => @base.tag("Comp", [fj(f), @base.jarray(args.map(fj))])
    Prim(z, s) => @base.tag("Prim", [fj(z), fj(s)])
    Mu(f) => @base.tag("Mu", [fj(f)])
  }
}

///|
fn fd(j : Json) -> Func raise @base.Fault {
  fn arg(i : Int32) -> Json raise @base.Fault {
    @base.tag_arg(j, i)
  }
  match @base.tag_name(j) {
    "Zero" => Zero
    "Succ" => Succ
    "Proj" => Proj(@base.int(arg(0)), @base.int(arg(1)))
    "Comp" => Comp(fd(arg(0)), @base.array(arg(1)).map(fd))
    "Prim" => Prim(fd(arg(0)), fd(arg(1)))
    "Mu" => Mu(fd(arg(0)))
    _ => @base.error("Invalid function snapshot")
  }
}

///|
fn pj(p : Process) -> Json {
  match p {
    Value(n) => @base.tag("Result", [@base.jstr(n.to_string())])
    Call(f, args) => @base.tag("Call", [fj(f), @base.jarray(args.map(pj))])
    Search(i, args, f, p) =>
      @base.tag("Search", [
        @base.jstr(i.to_string()),
        @base.jarray(args.map(n => @base.jstr(n.to_string()))),
        fj(f),
        pj(p),
      ])
  }
}

///|
fn pd(j : Json) -> Process raise @base.Fault {
  fn arg(i : Int32) -> Json raise @base.Fault {
    @base.tag_arg(j, i)
  }
  match @base.tag_name(j) {
    "Result" => Value(@base.json_nat(arg(0)))
    "Call" => {
      let f = fd(arg(0))
      validate(f)
      let args = @base.array(arg(1)).map(pd)
      @base.require(args.length() == arity(f), "Invalid process arity")
      Call(f, args)
    }
    "Search" => {
      let f = fd(arg(2))
      validate(f)
      let args = @base.array(arg(1)).map(@base.json_nat)
      @base.require(args.length() + 1 == arity(f), "Invalid search arity")
      Search(@base.json_nat(arg(0)), args, f, pd(arg(3)))
    }
    _ => @base.error("Invalid process snapshot")
  }
}

///|
fn encode(s : State) -> Json {
  @base.object([
    ("function_", fj(s.function_)),
    ("input", @base.jarray(s.input.map(n => @base.jstr(n.to_string())))),
    ("process", pj(s.process)),
  ])
}

///|
fn decode(j : Json) -> State raise @base.Fault {
  let function_ = fd(@base.field(j, "function_"))
  validate(function_)
  let input = @base.array(@base.field(j, "input")).map(@base.json_nat)
  @base.require(
    input.length() == arity(function_),
    "Invalid snapshot input arity",
  )
  { function_, input, process: pd(@base.field(j, "process")), }
}

///|
fn pt(p : Process) -> String {
  match p {
    Value(n) => n.to_string()
    Call(f, args) => print_code(f) + "(" + @base.join(args.map(pt), ", ") + ")"
    Search(n, _, _, p) => "Mu[" + n.to_string() + "](" + pt(p) + ")"
  }
}

///|
pub fn model() -> @runtime.Model {
  @runtime.model(make, step, encode, decode, s => {
    @base.jarray([
      @base.titled("function", print_code(s.function_)),
      @base.table(
        "input",
        ["arg", "value"],
        s.input.mapi((i, n) => [i.to_string(), n.to_string()]),
      ),
      @base.titled("process", pt(s.process)),
    ])
  })
}
