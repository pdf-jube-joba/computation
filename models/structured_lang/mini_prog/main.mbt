///|
fn validate(code : Code) -> Unit raise @utils.Fault {
  let names : Map[String, Bool] = Map([])
  for f in code.functions {
    @utils.require(!names.contains(f.name), "duplicate function: " + f.name)
    names[f.name] = true
  }
}

///|
fn make(code : Code, input : Ainput) -> State raise @utils.Fault {
  validate(code)
  let args = input.values.map(n => Number_value(n))
  let mut statics = []
  for definition in code.statics {
    statics = assoc_set(statics, definition.name, {
      ty: definition.ty,
      value: default_value(definition.ty),
    })
  }
  let s : State = {
    code,
    current_function: "main",
    exec: Ctrl(Next),
    env: [],
    store: { statics, heap: [], locals: [], },
    inner_k: [],
    fn_k: [],
    next_local_id: 0,
    next_heap_id: 0,
  }
  enter_function(s, "main", args)
}

///|
fn step(
  s : State,
  _input : Rinput,
) -> @definition.Transition[State, Routput, Foutput] raise @utils.Fault {
  match s.exec {
    Eval(st) => @definition.Continue(step_eval(s, st), ())
    Ctrl(c) => @definition.Continue(step_ctrl(s, c), ())
    Ret(values) =>
      if s.fn_k.is_empty() {
        @utils.require(
          values.length() == 1,
          "main must return exactly one value",
        )
        match values[0] {
          Number_value(n) => @definition.Halt(n)
          _ => @utils.error("main must return a natural number")
        }
      } else {
        @definition.Continue(step_ret(s, values), ())
      }
  }
}

///|
fn decode(j : Json) -> State raise @utils.Fault {
  let s = read_state(j)
  validate(s.code)
  @utils.require(
    s.next_local_id >= 0 && s.next_heap_id >= 0,
    "Invalid ID supply",
  )
  fn key(k : Key) -> Unit raise @utils.Fault {
    match k {
      Heap(id) =>
        @utils.require(id >= 0 && id < s.next_heap_id, "Invalid heap ID")
      Local_key(id) =>
        @utils.require(id >= 0 && id < s.next_local_id, "Invalid local ID")
      Static_key(_) => ()
    }
  }
  fn value(v : Value) -> Unit raise @utils.Fault {
    match v {
      Ptr_value(Some(loc)) => key(loc.key)
      Aggregate(vs) =>
        for v in vs {
          value(v)
        }
      Sum_value(_, v) => value(v)
      _ => ()
    }
  }
  for scope in s.env {
    for entry in scope {
      key(Local_key(entry.1))
    }
  }
  for frame in s.fn_k {
    for scope in frame.env {
      for entry in scope {
        key(Local_key(entry.1))
      }
    }
    for loc in frame.ret_locs {
      key(loc.key)
    }
  }
  for entry in s.store.heap {
    key(Heap(entry.0))
    value(entry.1.value)
  }
  for entry in s.store.statics {
    value(entry.1.value)
  }
  for scopes in s.store.locals {
    for cells in scopes {
      for entry in cells {
        key(Local_key(entry.0))
        value(entry.1.value)
      }
    }
  }
  s
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
