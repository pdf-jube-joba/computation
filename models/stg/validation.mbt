///|
fn validate_state(s : State) -> Unit raise @utils.Fault {
  validate(s.code)
  let globals = s.code.map(p => p.0)
  let expected = globals.mapi((a, x) => (x, Addr(a)))
  @utils.require(s.globals == expected, "Invalid global environment")
  let addresses = s.heap.map(p => p.0)
  addresses.sort()
  @utils.require(
    s.next_addr >= 0 &&
    addresses.length() == s.next_addr &&
    addresses.mapi((i, a) => i == a).all(b => b),
    "Invalid heap address sequence",
  )
  fn address(a : Int32) -> Unit raise @utils.Fault {
    ignore(heap(s, a))
  }
  fn value(v : Value) -> Unit raise @utils.Fault {
    match v {
      Addr(a) => address(a)
      _ => ()
    }
  }
  fn values(xs : Array[Value]) -> Unit raise @utils.Fault {
    for v in xs {
      value(v)
    }
  }
  fn environment(env : Env) -> Unit raise @utils.Fault {
    unique("environment binding", env.map(p => p.0))
    for p in env {
      value(p.1)
    }
  }
  fn function_address(a : Int32) -> Unit raise @utils.Fault {
    match heap(s, a) {
      Closure(lf, _) if lf.update == Reentrant && !lf.args.is_empty() => ()
      Pap(_, _) => ()
      _ => @utils.error("Function must point to a function closure or PAP")
    }
  }
  fn result(v : ResultValue) -> Unit raise @utils.Fault {
    match v {
      Number(_) => ()
      Data(_, xs) => values(xs)
      Function(a) => function_address(a)
    }
  }
  for p in s.globals {
    value(p.1)
  }
  for p in s.heap {
    match p.1 {
      Blackhole => ()
      Evaluated(v) => result(v)
      Pap(f, xs) => {
        values(xs)
        match heap(s, f) {
          Closure(lf, _) if lf.update == Reentrant &&
            !xs.is_empty() &&
            xs.length() < lf.args.length() => ()
          _ => @utils.error("Invalid partial application")
        }
      }
      Closure(lf, env) => {
        environment(env)
        @utils.require(env.map(p => p.0) == lf.free, "Closure capture mismatch")
        check_lambda(globals, append(env.map(p => p.0), globals), lf)
      }
    }
  }
  let updates = []
  for f in s.stack {
    match f {
      Update(a, _) => updates.push(a)
      _ => ()
    }
  }
  let used : Map[Int32, Bool] = Map([])
  for a in updates {
    @utils.require(!used.contains(a), "Duplicate update frame")
    used[a] = true
  }
  for p in s.heap {
    @utils.require(
      (p.1 is Blackhole) == updates.contains(p.0),
      "Black hole/update frame mismatch",
    )
  }
  for f in s.stack {
    match f {
      Update(a, xs) => {
        address(a)
        values(xs)
      }
      Match(alts, env, xs) => {
        environment(env)
        values(xs)
        check_alternatives(globals, append(env.map(p => p.0), globals), alts)
      }
    }
  }
  values(s.arguments)
  match s.control {
    Enter(v) => value(v)
    Return(v) => result(v)
    Eval(e, env) => {
      environment(env)
      check_expr(globals, append(env.map(p => p.0), globals), e)
    }
  }
}

///|
pub fn model() -> @definition.Model[Code, Unit, Unit, Unit, ResultValue, State] {
  @definition.Model(make=(code, input) => make(code, input), step=(state, input) => {
    step(state, input)
  })
}
