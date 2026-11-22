///|
#derive(Eq)
priv struct Location {
  frame_id : Int32
  slot : Int32
}

///|
priv enum Value {
  Num(Int64)
  Location(Location)
  Null_ptr
}

///|
priv struct FrameStore {
  frame_id : Int32
  values : Array[(Location, Value)]
}

///|
priv struct Frame {
  env : Array[(String, Location)]
  inner_k : Array[Stmt]
  function_name : String
}

///|
struct State {
  code : Code
  current_function : String
  current_stmt : Stmt?
  env : Array[(String, Location)]
  store : Array[FrameStore]
  inner_k : Array[Stmt]
  fn_k : Array[Frame]
  next_frame_id : Int32
}

///|
fn location_text(l : Location) -> String {
  "\\loc(\{l.frame_id}, \{l.slot})"
}

///|
fn value_text(v : Value) -> String {
  match v {
    Num(n) => n.to_string()
    Location(l) => location_text(l)
    Null_ptr => "\\null_ptr"
  }
}

///|
fn validate(code : Code) -> Unit raise @utils.Fault {
  let names : Map[String, Bool] = Map([])
  for f in code.functions {
    @utils.require(!names.contains(f.name), "duplicate function: " + f.name)
    names[f.name] = true
  }
}

///|
fn lookup_function(code : Code, name : String) -> Function raise @utils.Fault {
  for f in code.functions {
    if f.name == name {
      return f
    }
  }
  @utils.error("unknown function: " + name)
}

///|
fn bindings(
  id : Int32,
  params : Array[String],
  values : Array[Value],
) -> (Array[(String, Location)], Array[(Location, Value)]) {
  let mut env = []
  let store = []
  for slot, name in params {
    let loc = { frame_id: id, slot, }
    env = @utils.env_set(env, name, loc)
    store.insert(0, (loc, values[slot]))
  }
  (@utils.env_sorted(env), store)
}

///|
fn store_lookup(s : State, loc : Location) -> Value raise @utils.Fault {
  for frame in s.store {
    for pair in frame.values {
      if pair.0 == loc {
        return pair.1
      }
    }
  }
  @utils.error("unknown location: " + location_text(loc))
}

///|
fn store_assign(
  s : State,
  loc : Location,
  value : Value,
) -> State raise @utils.Fault {
  for index, frame in s.store {
    if frame.values.any(pair => pair.0 == loc) {
      let store = s.store.copy()
      store[index] = {
        ..frame,
        values: frame.values.map(pair => {
          if pair.0 == loc {
            (loc, value)
          } else {
            pair
          }
        }),
      }
      return { ..s, store, }
    }
  }
  @utils.error("unknown location: " + location_text(loc))
}

///|
fn eval_place(s : State, p : Place) -> Location raise @utils.Fault {
  match p {
    Var(n) => @utils.env_lookup(s.env, n)
    Deref(v) =>
      match eval_value(s, v) {
        Location(l) => l
        Null_ptr => @utils.error("cannot dereference \\null_ptr")
        Num(_) => @utils.error("dereference expects a location value")
      }
  }
}

///|
fn number(v : Value) -> Int64 raise @utils.Fault {
  match v {
    Num(n) => n
    _ => @utils.error("expected a number, found " + value_text(v))
  }
}

///|
fn eval_value(s : State, v : Expr) -> Value raise @utils.Fault {
  match v {
    Number(n) => Num(n)
    NullPtr => Null_ptr
    BinOp(a, op, b) => {
      let a = number(eval_value(s, a))
      let b = number(eval_value(s, b))
      Num(
        match op {
          Add => a + b
          Sub => a - b
        },
      )
    }
    Load(p) => store_lookup(s, eval_place(s, p))
    Addr(p) => Location(eval_place(s, p))
  }
}

///|
fn make(code : Code, _input : Unit) -> State raise @utils.Fault {
  validate(code)
  let f = lookup_function(code, "main")
  @utils.require(!f.body.is_empty(), "function body must not be empty")
  let (env, values) = bindings(0, f.params, f.params.map(_ => Num(0L)))
  {
    code,
    current_function: f.name,
    current_stmt: Some(f.body[0]),
    env,
    store: [{ frame_id: 0, values, }],
    inner_k: f.body[1:].to_owned(),
    fn_k: [],
    next_frame_id: 1,
  }
}

///|
fn sorted_values(frame : FrameStore) -> Array[(Location, Value)] {
  let values = frame.values.copy()
  values.sort_by((a, b) => {
    let cmp = a.0.frame_id.compare(b.0.frame_id)
    if cmp == 0 {
      a.0.slot.compare(b.0.slot)
    } else {
      cmp
    }
  })
  values
}

///|
fn final_output(report : Final) -> String {
  fn entries(indent : String, frame : FrameStore) -> String {
    @utils.join(
      sorted_values(frame).map(pair => {
        indent + location_text(pair.0) + " = " + value_text(pair.1) + "\n"
      }),
      "",
    )
  }
  "halted\ncurrent_function: " +
  report.current_function +
  "\nfinished_frame: " +
  report.finished.frame_id.to_string() +
  "\n" +
  entries("  ", report.finished) +
  "remaining_store:\n" +
  (if report.remaining_store.is_empty() {
    "  <empty>\n"
  } else {
    @utils.join(
      report.remaining_store
      .rev()
      .map(f => "  frame " + f.frame_id.to_string() + ":\n" + entries("    ", f)),
      "",
    )
  })
}

///|
fn step(
  s : State,
  _input : Rinput,
) -> @definition.Transition[State, Routput, Foutput] raise @utils.Fault {
  match s.current_stmt {
    None => {
      @utils.require(
        !s.inner_k.is_empty(),
        "control reached end of function body without return",
      )
      @definition.Continue(
        {
          ..s,
          current_stmt: Some(s.inner_k[0]),
          inner_k: s.inner_k[1:].to_owned(),
        },
        (),
      )
    }
    Some(Assign(p, v)) => {
      let loc = eval_place(s, p)
      let value = eval_value(s, v)
      @definition.Continue(
        { ..store_assign(s, loc, value), current_stmt: None, },
        (),
      )
    }
    Some(Ifz(c, a, b)) => {
      let selected = match eval_value(s, c) {
        Num(n) => if n == 0L { a } else { b }
        v => @utils.error("ifz expects a number, found " + value_text(v))
      }
      @definition.Continue({ ..s, current_stmt: Some(selected), }, ())
    }
    Some(Call(name, args)) => {
      let values = args.map(v => eval_value(s, v))
      let f = lookup_function(s.code, name)
      @utils.require(
        values.length() == f.params.length(),
        "argument count mismatch for " + name,
      )
      @utils.require(!f.body.is_empty(), "function body must not be empty")
      @utils.require(s.next_frame_id < 2147483647, "Frame ID exhausted")
      let (env, values) = bindings(s.next_frame_id, f.params, values)
      @definition.Continue(
        {
          ..s,
          current_function: name,
          current_stmt: Some(f.body[0]),
          env,
          store: [{ frame_id: s.next_frame_id, values, }] + s.store,
          inner_k: f.body[1:].to_owned(),
          fn_k: [
            {
              env: s.env,
              inner_k: s.inner_k,
              function_name: s.current_function,
            },
          ] +
          s.fn_k,
          next_frame_id: s.next_frame_id + 1,
        },
        (),
      )
    }
    Some(Return) => {
      @utils.require(!s.store.is_empty(), "return on empty store stack")
      let finished = s.store[0]
      let s = { ..s, store: s.store[1:].to_owned(), }
      if s.fn_k.is_empty() {
        @definition.Halt(Final::{
          current_function: s.current_function,
          finished,
          remaining_store: s.store,
        })
      } else {
        let f = s.fn_k[0]
        @definition.Continue(
          {
            ..s,
            current_function: f.function_name,
            env: f.env,
            inner_k: f.inner_k,
            fn_k: s.fn_k[1:].to_owned(),
            current_stmt: None,
          },
          (),
        )
      }
    }
  }
}

///|
fn render(s : State) -> Array[@definition.Block] {
  [
    @definition.text_table("status", ["field", "value"], [
      ["control", if s.current_stmt is None { "next" } else { "eval" }],
      ["function", s.current_function],
      [
        "current",
        match s.current_stmt {
          Some(st) => stmt_text(st)
          None => "<none>"
        },
      ],
      ["next_frame_id", s.next_frame_id.to_string()],
    ]),
    @definition.text_table(
      "env",
      ["var", "location", "value"],
      @utils.env_sorted(s.env).map(pair => {
        [
          pair.0,
          location_text(pair.1),
          value_text(store_lookup(s, pair.1) catch { _ => Null_ptr }),
        ]
      }),
    ),
    @definition.container(
      "store",
      s.store
      .rev()
      .map(f => {
        @definition.text_table(
          "frame " + f.frame_id.to_string(),
          ["location", "value"],
          sorted_values(f).map(pair => {
            [location_text(pair.0), value_text(pair.1)]
          }),
        )
      }),
    ),
    @definition.container(
      "inner_k",
      s.inner_k.map(st => @definition.text(stmt_text(st))),
    ),
    @definition.text_table(
      "fn_k",
      ["depth", "function", "resume_len"],
      s.fn_k.mapi((i, f) => {
        [i.to_string(), f.function_name, f.inner_k.length().to_string()]
      }),
    ),
  ]
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
