///|
priv struct Environment {
  vars : Array[(String, @base.Nat)]
  task_ids : Array[(String, Int32?)]
}

///|
priv struct Frame {
  function_name : String
  pc : Int32
}

///|
priv struct State {
  code : Array[FunctionDef]
  env : Environment
  workers : Array[Int32?]
  queue : Array[Int32]
  tasks : Array[Array[Frame]]
}

///|
fn get_id(env : Environment, name : String) -> Int32 raise @base.Fault {
  match @data.env_get(env.task_ids, name) {
    Some(Some(id)) => id
    Some(None) => @base.error("task id $" + name + " is not initialized")
    None => @base.error("unknown task id $" + name)
  }
}

///|
fn eval_a(env : Environment, a : Aexp) -> @base.Nat {
  match a {
    Atom(Var(n)) => @data.env_get(env.vars, n).unwrap_or(@base.zero())
    Atom(Imm(n)) => n
    Bin(a, o, b) => {
      let a = eval_a(env, a)
      let b = eval_a(env, b)
      match o {
        Add => a + b
        Sub => @base.sub(a, b)
        Mul => a * b
      }
    }
  }
}

///|
fn eval_b(s : State, b : Bexp) -> Bool raise @base.Fault {
  match b {
    Not(b) => !eval_b(s, b)
    Rel(a, o, b) => {
      let a = eval_a(s.env, a)
      let b = eval_a(s.env, b)
      match o {
        Eq => a == b
        Ne => a != b
        Lt => a < b
      }
    }
    And(a, b) => eval_b(s, a) && eval_b(s, b)
    Or(a, b) => eval_b(s, a) || eval_b(s, b)
    Done(n) => {
      let id = get_id(s.env, n)
      id < s.tasks.length() && s.tasks[id].is_empty()
    }
  }
}

///|
fn find_function(
  code : Array[FunctionDef],
  name : String,
) -> FunctionDef raise @base.Fault {
  match code.iter().find_first(f => f.name == name) {
    Some(f) => f
    None => @base.error("unknown function: " + name)
  }
}

///|
fn make(text : String, input : String) -> State raise @base.Fault {
  let code = parse(text)
  validate(code)
  let agents = @base.index(@base.nat(input))
  @base.require(agents > 0, "agent count must be positive")
  {
    code,
    env: { vars: [], task_ids: [], },
    workers: Array::make(agents, None),
    queue: [0],
    tasks: [[{ function_name: "main", pc: 0, }]],
  }
}

///|
fn output(env : Environment) -> String {
  @data.print_env(env.vars, n => n.to_string()) +
  @base.join(
    env.task_ids.map(p => {
      "$" +
      p.0 +
      " = " +
      (match p.1 {
        None => "\\none"
        Some(id) => id.to_string()
      }) +
      "\n"
    }),
    "",
  )
}

///|
fn step(
  s : State,
  input : String,
) -> @runtime.Transition[State] raise @base.Fault {
  let agent = @base.index(@base.nat(input))
  @base.require(
    agent < s.workers.length(),
    "agent " + agent.to_string() + " out of range",
  )
  let workers = s.workers.copy()
  let tasks = s.tasks.copy()
  let s = { ..s, workers, tasks, }
  match workers[agent] {
    None =>
      if s.queue.is_empty() {
        @runtime.Halt(output(s.env))
      } else {
        let id = s.queue[0]
        workers[agent] = Some(id)
        @runtime.Continue(
          { ..s, queue: s.queue[1:].to_owned(), },
          "agent \{agent}: dequeue task \{id}",
        )
      }
    Some(id) => {
      @base.require(
        id >= 0 && id < tasks.length(),
        "unknown task " + id.to_string(),
      )
      let frames = tasks[id]
      if frames.is_empty() {
        workers[agent] = None
        return @runtime.Continue(s, "agent \{agent}: release task \{id}")
      }
      let frame = frames[0]
      let rest = frames[1:].to_owned()
      let f = find_function(s.code, frame.function_name)
      if frame.pc == f.body.length() {
        tasks[id] = rest
        return @runtime.Continue(
          s,
          "agent \{agent}: return from " + frame.function_name,
        )
      }
      @base.require(
        frame.pc >= 0 && frame.pc < f.body.length(),
        "pc \{frame.pc} out of bounds in " + frame.function_name,
      )
      let stmt = f.body[frame.pc]
      fn shift(offset : Int32) -> Unit raise @base.Fault {
        let wide = frame.pc.to_int64() + offset.to_int64()
        @base.require(
          wide >= 0L && wide <= 2147483647L,
          "pc became negative or overflowed in " +
          frame.function_name +
          ": \{frame.pc} + \{offset}",
        )
        let next = [{ ..frame, pc: wide.to_int(), }]
        next.append(rest)
        tasks[id] = next
      }
      let next = match stmt {
        Assign(n, a) => {
          let value = eval_a(s.env, a)
          shift(1)
          {
            ..s,
            env: {
              ..s.env,
              vars: @data.env_sorted(@data.env_set(s.env.vars, n, value)),
            },
          }
        }
        IfGoto(b, offset) => {
          shift(if eval_b(s, b) { offset } else { 1 })
          s
        }
        Goto(offset) => {
          shift(offset)
          s
        }
        Call(n) => {
          ignore(find_function(s.code, n))
          shift(1)
          let frames = [{ function_name: n, pc: 0, }]
          frames.append(tasks[id])
          tasks[id] = frames
          s
        }
        Run(n, id_var) => {
          ignore(find_function(s.code, n))
          let new_id = tasks.length()
          shift(1)
          let expanded = tasks.copy()
          expanded.push([{ function_name: n, pc: 0, }])
          let queue = s.queue.copy()
          queue.push(new_id)
          {
            ..s,
            tasks: expanded,
            queue,
            env: {
              ..s.env,
              task_ids: @data.env_sorted(
                @data.env_set(s.env.task_ids, id_var, Some(new_id)),
              ),
            },
          }
        }
        Yield => {
          shift(1)
          workers[agent] = None
          let queue = s.queue.copy()
          queue.push(id)
          { ..s, queue, }
        }
      }
      @runtime.Continue(next, "agent \{agent}: " + stext(stmt))
    }
  }
}

///|
fn function_json(f : FunctionDef) -> Json {
  @base.object([
    ("name", @base.jstr(f.name)),
    ("body", @base.jarray(f.body.map(stmt_json))),
  ])
}

///|
fn decode_function(j : Json) -> FunctionDef raise @base.Fault {
  {
    name: @base.identifier(@base.string(@base.field(j, "name"))),
    body: @base.array(@base.field(j, "body")).map(decode_stmt),
  }
}

///|
fn env_json(env : Environment) -> Json {
  @base.object([
    ("vars", @data.env_json(env.vars, n => @base.jstr(n.to_string()))),
    (
      "task_ids",
      @data.env_json(env.task_ids, id => @base.optional_json(id, @base.jint)),
    ),
  ])
}

///|
fn optional_id(j : Json) -> Int32? raise @base.Fault {
  match j {
    Null => None
    _ => {
      let id = @base.int(j)
      @base.require(id >= 0, "Invalid task ID")
      Some(id)
    }
  }
}

///|
fn decode_env(j : Json) -> Environment raise @base.Fault {
  {
    vars: @data.decode_env(@base.field(j, "vars"), @base.json_nat),
    task_ids: @data.decode_env(@base.field(j, "task_ids"), optional_id),
  }
}

///|
fn frame_json(f : Frame) -> Json {
  @base.object([
    ("function_name", @base.jstr(f.function_name)),
    ("pc", @base.jint(f.pc)),
  ])
}

///|
fn decode_frame(j : Json) -> Frame raise @base.Fault {
  {
    function_name: @base.identifier(
      @base.string(@base.field(j, "function_name")),
    ),
    pc: @base.int(@base.field(j, "pc")),
  }
}

///|
fn encode(s : State) -> Json {
  @base.object([
    ("code", @base.jarray(s.code.map(function_json))),
    ("env", env_json(s.env)),
    (
      "workers",
      @base.jarray(s.workers.map(id => @base.optional_json(id, @base.jint))),
    ),
    ("queue", @base.jarray(s.queue.map(@base.jint))),
    ("tasks", @base.jarray(s.tasks.map(fs => @base.jarray(fs.map(frame_json))))),
  ])
}

///|
fn decode(j : Json) -> State raise @base.Fault {
  let code = @base.array(@base.field(j, "code")).map(decode_function)
  validate(code)
  let workers = @base.array(@base.field(j, "workers")).map(optional_id)
  let queue = @base.array(@base.field(j, "queue")).map(@base.int)
  let tasks = @base.array(@base.field(j, "tasks")).map(j => {
    @base.array(j).map(decode_frame)
  })
  for id in queue {
    @base.require(id >= 0 && id < tasks.length(), "Unknown task ID in snapshot")
  }
  for id in workers {
    match id {
      Some(id) =>
        @base.require(id < tasks.length(), "Unknown task ID in snapshot")
      _ => ()
    }
  }
  { code, env: decode_env(@base.field(j, "env")), workers, queue, tasks, }
}

///|
fn container(
  title : String,
  children : Array[Json],
  horizontal~ : Bool = false,
) -> Json {
  @base.object([
    ("kind", @base.jstr("container")),
    ("title", @base.jstr(title)),
    (
      "orientation",
      @base.jstr(if horizontal { "horizontal" } else { "vertical" }),
    ),
    ("display", @base.jstr("block")),
    ("children", @base.jarray(children)),
  ])
}

///|
fn render(s : State) -> Json {
  @base.jarray([
    @base.table("status", ["field", "value"], [
      ["agents", s.workers.length().to_string()],
      ["queue_len", s.queue.length().to_string()],
      ["task_count", s.tasks.length().to_string()],
    ]),
    container(
      "env",
      [
        @base.table(
          "vars",
          ["var", "value"],
          s.env.vars.map(p => [p.0, p.1.to_string()]),
        ),
        @base.table(
          "task_ids",
          ["id", "task"],
          s.env.task_ids.map(p => {
            [
              "$" + p.0,
              match p.1 {
                None => "none"
                Some(id) => id.to_string()
              },
            ]
          }),
        ),
      ],
      horizontal=true,
    ),
    @base.table(
      "workers",
      ["agent", "task"],
      s.workers.mapi((i, id) => {
        [
          i.to_string(),
          match id {
            None => "idle"
            Some(id) => id.to_string()
          },
        ]
      }),
    ),
    @base.table(
      "queue",
      ["slot", "task"],
      s.queue.mapi((i, id) => [i.to_string(), id.to_string()]),
    ),
    container(
      "tasks",
      s.tasks.mapi((id, fs) => {
        @base.table(
          "task \{id}",
          ["depth", "function", "pc"],
          fs
          .rev()
          .mapi((i, f) => [i.to_string(), f.function_name, f.pc.to_string()]),
        )
      }),
    ),
  ])
}

///|
pub fn model() -> @runtime.Model {
  @runtime.model(make, step, encode, decode, render)
}

///|
test "yielding scheduler and spawned task snapshots" {
  let m = model()
  m.make(
    "\\fn worker { x <- x + 1; \\yield; x <- x + 2 } \\fn main { \\run worker -> $w; \\yield; \\if !\\done $w \\goto -1 }",
    "1",
  )
  let mut output = None
  for _ in 0..<100 {
    let saved = m.snapshot()
    let result = m.step("0")
    m.restore(saved)
    assert_eq(m.step("0"), result)
    let j = @base.parse_json(result)
    if @base.string(@base.field(j, "kind")) == "halt" {
      output = Some(@base.string(@base.field(j, "foutput")))
      break
    }
  }
  assert_eq(output, Some("x = 3\n$w = 1\n"))
}
