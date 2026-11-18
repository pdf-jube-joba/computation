///|
fn validate_state(s : State) -> Unit raise @base.Fault {
  validate(s.code)
  let globals = s.code.map(p => p.0)
  let expected = globals.mapi((a, x) => (x, Addr(a)))
  @base.require(s.globals == expected, "Invalid global environment")
  let addresses = s.heap.map(p => p.0)
  addresses.sort()
  @base.require(
    s.next_addr >= 0 &&
    addresses.length() == s.next_addr &&
    addresses.mapi((i, a) => i == a).all(b => b),
    "Invalid heap address sequence",
  )
  fn address(a : Int32) -> Unit raise @base.Fault {
    ignore(heap(s, a))
  }
  fn value(v : Value) -> Unit raise @base.Fault {
    match v {
      Addr(a) => address(a)
      _ => ()
    }
  }
  fn values(xs : Array[Value]) -> Unit raise @base.Fault {
    for v in xs {
      value(v)
    }
  }
  fn environment(env : Env) -> Unit raise @base.Fault {
    unique("environment binding", env.map(p => p.0))
    for p in env {
      value(p.1)
    }
  }
  fn function_address(a : Int32) -> Unit raise @base.Fault {
    match heap(s, a) {
      Closure(lf, _) if lf.update == Reentrant && !lf.args.is_empty() => ()
      Pap(_, _) => ()
      _ => @base.error("Function must point to a function closure or PAP")
    }
  }
  fn result(v : ResultValue) -> Unit raise @base.Fault {
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
          _ => @base.error("Invalid partial application")
        }
      }
      Closure(lf, env) => {
        environment(env)
        @base.require(env.map(p => p.0) == lf.free, "Closure capture mismatch")
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
    @base.require(!used.contains(a), "Duplicate update frame")
    used[a] = true
  }
  for p in s.heap {
    @base.require(
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
pub fn model() -> @runtime.Model {
  @runtime.model(make, step, encode, decode, render)
}

///|
test "updatable thunks share evaluated results and black holes fail atomically" {
  let m = model()
  m.make(
    "main = \\fun [] \\u [] -> \\let { x = \\fun [] \\u [] -> 3 } \\in \\case x() \\of { \\default n -> \\case x() \\of { \\default m -> \\add(n,m) } }",
    "",
  )
  let mut output = None
  for _ in 0..<60 {
    let saved = m.snapshot()
    let result = m.step("")
    m.restore(saved)
    assert_eq(m.step(""), result)
    let j = @base.parse_json(result)
    if @base.string(@base.field(j, "kind")) == "halt" {
      output = Some(@base.string(@base.field(j, "foutput")))
      break
    }
  }
  assert_eq(output, Some("6"))
  m.make("main = \\fun [] \\u [] -> main()", "")
  ignore(m.step(""))
  ignore(m.step(""))
  let saved = m.snapshot()
  assert_true(
    try {
      ignore(m.step(""))
      false
    } catch {
      _ => true
    },
  )
  assert_eq(m.snapshot(), saved)
}
