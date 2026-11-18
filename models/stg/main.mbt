///|
#derive(Eq)
priv enum Value {
  Addr(Int32)
  Integer(@base.Nat)
}

///|
type Env = Array[(String, Value)]

///|
priv enum ResultValue {
  Number(@base.Nat)
  Data(String, Array[Value])
  Function(Int32)
}

///|
priv enum Closure {
  Closure(Lambda, Env)
  Blackhole
  Evaluated(ResultValue)
  Pap(Int32, Array[Value])
}

///|
priv enum Control {
  Eval(Expr, Env)
  Enter(Value)
  Return(ResultValue)
}

///|
priv enum Frame {
  Match(Array[(Pattern, Expr)], Env, Array[Value])
  Update(Int32, Array[Value])
}

///|
priv struct State {
  code : Code
  globals : Env
  heap : Array[(Int32, Closure)]
  next_addr : Int32
  control : Control
  arguments : Array[Value]
  stack : Array[Frame]
}

///|
fn value_text(v : Value) -> String {
  match v {
    Addr(a) => "@" + a.to_string()
    Integer(n) => n.to_string()
  }
}

///|
fn values_text(xs : Array[Value]) -> String {
  @base.join(xs.map(value_text), ", ")
}

///|
fn result_text(v : ResultValue) -> String {
  match v {
    Number(n) => n.to_string()
    Data(c, xs) => "\\con " + c + "(" + values_text(xs) + ")"
    Function(a) => "\\function @" + a.to_string()
  }
}

///|
fn lookup(env : Env, x : String) -> Value raise @base.Fault {
  match @data.env_get(env, x) {
    Some(v) => v
    None => @base.error("Unbound variable: " + x)
  }
}

///|
fn heap(s : State, a : Int32) -> Closure raise @base.Fault {
  match s.heap.iter().find_first(p => p.0 == a) {
    Some(p) => p.1
    None => @base.error("Invalid heap address: @" + a.to_string())
  }
}

///|
fn bind(bindings : Env, env : Env) -> Env {
  append(bindings, env.filter(p => !bindings.any(b => b.0 == p.0)))
}

///|
fn atom(env : Env, a : Atom) -> Value raise @base.Fault {
  match a {
    Var(x) => lookup(env, x)
    Lit(n) => Integer(n)
  }
}

///|
fn set_heap(s : State, a : Int32, c : Closure) -> State {
  { ..s, heap: s.heap.map(p => (p.0, if p.0 == a { c } else { p.1 })), }
}

///|
fn allocate(s : State, c : Closure) -> (Int32, State) raise @base.Fault {
  @base.require(s.next_addr < 2147483647, "Heap address space exhausted")
  let a = s.next_addr
  (a, { ..s, heap: append([(a, c)], s.heap), next_addr: a + 1, })
}

///|
fn allocate_bindings(
  s : State,
  env : Env,
  rec : Bool,
  bindings : Code,
) -> (State, Env) raise @base.Fault {
  let count = bindings.length()
  @base.require(
    count <= 2147483647 - s.next_addr,
    "Heap address space exhausted",
  )
  let addresses = bindings.mapi((i, b) => (b.0, Addr(s.next_addr + i)))
  let inner = bind(addresses, env)
  let captures = append(if rec { inner } else { env }, s.globals)
  let heap = bindings.mapi((i, b) => {
    (s.next_addr + i, Closure(b.1, b.1.free.map(x => (x, lookup(captures, x)))))
  })
  ({ ..s, heap: append(heap, s.heap), next_addr: s.next_addr + count, }, inner)
}

///|
fn make(text : String, input : String) -> State raise @base.Fault {
  @base.unit_input(input)
  let code = parse(text)
  let globals = code.mapi((a, b) => (b.0, Addr(a)))
  {
    code,
    globals,
    heap: code.mapi((a, b) => (a, Closure(b.1, []))),
    next_addr: code.length(),
    control: Enter(lookup(globals, "main")),
    arguments: [],
    stack: [],
  }
}

///|
fn primitive(o : Prim, a : Value, b : Value) -> ResultValue raise @base.Fault {
  match (a, b) {
    (Integer(a), Integer(b)) =>
      Number(
        match o {
          Add => a + b
          Sub => @base.sub(a, b)
          Mul => a * b
          Eq => if a == b { @base.one() } else { @base.zero() }
          Lt => if a < b { @base.one() } else { @base.zero() }
        },
      )
    _ =>
      @base.error(
        "Primitive operands must be unboxed naturals; force them with case",
      )
  }
}

///|
fn match_result(
  s : State,
  v : ResultValue,
  alts : Array[(Pattern, Expr)],
  env : Env,
) -> (State, Expr, Env) raise @base.Fault {
  let chosen = alts
    .iter()
    .find_first(a => {
      match (a.0, v) {
        (Constructor(c, _), Data(d, _)) => c == d
        (Literal(n), Number(m)) => n == m
        _ => false
      }
    })
  let chosen = match chosen {
    Some(a) => a
    None =>
      match alts.iter().find_first(a => a.0 is Default(_)) {
        Some(a) => a
        None => @base.error("No matching case alternative")
      }
  }
  let (p, body) = chosen
  match (p, v) {
    (Constructor(_, names), Data(_, fields)) => {
      @base.require(
        names.length() == fields.length(),
        "Constructor pattern arity mismatch",
      )
      (s, body, bind(names.mapi((i, n) => (n, fields[i])), env))
    }
    (Constructor(_, _), _) => @base.error("Constructor pattern arity mismatch")
    (Default(Some(x)), v) => {
      let (v, s) = match v {
        Number(n) => (Integer(n), s)
        Function(a) => (Addr(a), s)
        Data(_, _) => {
          let (a, s) = allocate(s, Evaluated(v))
          (Addr(a), s)
        }
      }
      (s, body, bind([(x, v)], env))
    }
    _ => (s, body, env)
  }
}

///|
fn transition(s : State) -> State raise @base.Fault {
  match s.control {
    Eval(e, env) => {
      let scope = append(env, s.globals)
      match e {
        Nat(n) => { ..s, control: Return(Number(n)), }
        App(f, xs) =>
          {
            ..s,
            control: Enter(lookup(scope, f)),
            arguments: append(xs.map(a => atom(scope, a)), s.arguments),
          }
        Con(c, xs) =>
          { ..s, control: Return(Data(c, xs.map(a => atom(scope, a)))), }
        Prim(o, a, b) =>
          {
            ..s,
            control: Return(primitive(o, atom(scope, a), atom(scope, b))),
          }
        Let(rec, bs, body) => {
          let (s, env) = allocate_bindings(s, env, rec, bs)
          { ..s, control: Eval(body, env), }
        }
        Case(e, alts) =>
          {
            ..s,
            control: Eval(e, env),
            arguments: [],
            stack: append([Match(alts, env, s.arguments)], s.stack),
          }
      }
    }
    Enter(Integer(n)) => { ..s, control: Return(Number(n)), }
    Enter(Addr(a)) =>
      match heap(s, a) {
        Blackhole =>
          @base.error("Black hole: recursive thunk entry at @" + a.to_string())
        Evaluated(v) => { ..s, control: Return(v), }
        Pap(f, xs) =>
          if s.arguments.is_empty() {
            { ..s, control: Return(Function(a)), }
          } else {
            {
              ..s,
              control: Enter(Addr(f)),
              arguments: append(xs, s.arguments),
            }
          }
        Closure(lf, env) =>
          if lf.update == Updatable {
            let next = set_heap(s, a, Blackhole)
            {
              ..next,
              control: Eval(lf.body, env),
              arguments: [],
              stack: append([Update(a, s.arguments)], s.stack),
            }
          } else {
            let arity = lf.args.length()
            if s.arguments.length() < arity {
              let (a, s) = if s.arguments.is_empty() {
                (a, s)
              } else {
                allocate(s, Pap(a, s.arguments))
              }
              { ..s, control: Return(Function(a)), arguments: [], }
            } else {
              let supplied = s.arguments[:arity]
              let arguments = s.arguments[arity:].to_owned()
              {
                ..s,
                control: Eval(
                  lf.body,
                  bind(lf.args.mapi((i, n) => (n, supplied[i])), env),
                ),
                arguments,
              }
            }
          }
      }
    Return(v) =>
      if !s.arguments.is_empty() {
        match v {
          Function(a) => { ..s, control: Enter(Addr(a)), }
          _ => @base.error("Cannot apply a number or constructor")
        }
      } else {
        @base.require(!s.stack.is_empty(), "Evaluation already finished")
        let stack = s.stack[1:].to_owned()
        match s.stack[0] {
          Update(a, arguments) => {
            let s = set_heap(s, a, Evaluated(v))
            { ..s, arguments, stack, }
          }
          Match(alts, env, arguments) => {
            let (s, body, env) = match_result(s, v, alts, env)
            { ..s, control: Eval(body, env), arguments, stack, }
          }
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
  match s.control {
    Return(v) if s.arguments.is_empty() && s.stack.is_empty() =>
      @runtime.Halt(result_text(v))
    _ => @runtime.Continue(transition(s), "")
  }
}

///|
fn env_text(env : Env) -> String {
  @base.join(env.map(p => p.0 + " = " + value_text(p.1)), ", ")
}

///|
fn closure_text(c : Closure) -> String {
  match c {
    Closure(lf, env) => lambda_text(lf) + " {" + env_text(env) + "}"
    Blackhole => "black hole"
    Evaluated(v) => "updated: " + result_text(v)
    Pap(a, xs) => "PAP @" + a.to_string() + "(" + values_text(xs) + ")"
  }
}

///|
fn frame_text(f : Frame) -> String {
  match f {
    Update(a, xs) =>
      "update @" + a.to_string() + "; saved arguments: " + values_text(xs)
    Match(alts, env, xs) =>
      @base.join(
        alts.map(a => pattern_text(a.0) + " -> " + expr_text(a.1)),
        "; ",
      ) +
      " {" +
      env_text(env) +
      "}; saved arguments: " +
      values_text(xs)
  }
}

///|
fn render(s : State) -> Json {
  let (mode, focus, env) = match s.control {
    Eval(e, env) => ("eval", expr_text(e), env)
    Enter(v) => ("enter", value_text(v), [])
    Return(v) => ("return", result_text(v), [])
  }
  let heap = s.heap.copy()
  heap.sort_by((a, b) => a.0.compare(b.0))
  @base.jarray([
    @base.titled("code", code_text(s.code)),
    @base.table("control", ["mode", "focus"], [[mode, focus]]),
    @base.table(
      "environment",
      ["name", "value"],
      env.map(p => [p.0, value_text(p.1)]),
    ),
    @base.table(
      "globals",
      ["name", "value"],
      s.globals.map(p => [p.0, value_text(p.1)]),
    ),
    @base.table(
      "arguments (top first)",
      ["index", "value"],
      s.arguments.mapi((i, v) => [i.to_string(), value_text(v)]),
    ),
    @base.table(
      "return / update stack (top first)",
      ["index", "value"],
      s.stack.mapi((i, f) => [i.to_string(), frame_text(f)]),
    ),
    @base.table(
      "heap",
      ["address", "closure"],
      heap.map(p => ["@" + p.0.to_string(), closure_text(p.1)]),
    ),
  ])
}
