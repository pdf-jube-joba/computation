///|
struct Environment {
  vars : Array[(String, @utils.Nat)]
  task_ids : Array[(String, Int32?)]
}

///|
priv struct Frame {
  function_name : String
  pc : Int32
}

///|
struct State {
  code : Array[FunctionDef]
  env : Environment
  workers : Array[Int32?]
  queue : Array[Int32]
  tasks : Array[Array[Frame]]
}

///|
fn get_id(env : Environment, name : String) -> Int32 raise @utils.Fault {
  match @utils.env_get(env.task_ids, name) {
    Some(Some(id)) => id
    Some(None) => @utils.error("task id $" + name + " is not initialized")
    None => @utils.error("unknown task id $" + name)
  }
}

///|
fn eval_a(env : Environment, a : Aexp) -> @utils.Nat {
  match a {
    Atom(Var(n)) => @utils.env_get(env.vars, n).unwrap_or(@utils.zero())
    Atom(Imm(n)) => n
    Bin(a, o, b) => {
      let a = eval_a(env, a)
      let b = eval_a(env, b)
      match o {
        Add => a + b
        Sub => @utils.sub(a, b)
        Mul => a * b
      }
    }
  }
}

///|
fn eval_b(s : State, b : Bexp) -> Bool raise @utils.Fault {
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
) -> FunctionDef raise @utils.Fault {
  match code.iter().find_first(f => f.name == name) {
    Some(f) => f
    None => @utils.error("unknown function: " + name)
  }
}

///|
fn make(program : Program, agents : UInt32) -> State raise @utils.Fault {
  let code = program.functions
  validate(code)
  @utils.require(
    agents > 0 && agents <= 2147483647U,
    "agent count must fit in a positive index",
  )
  let agents = agents.reinterpret_as_int()
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
  @utils.print_env(env.vars, n => n.to_string()) +
  @utils.join(
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
  input : Rinput,
) -> @definition.Transition[State, Routput, Foutput] raise @utils.Fault {
  @utils.require(
    input < s.workers.length().reinterpret_as_uint(),
    "agent " + input.to_string() + " out of range",
  )
  let agent = input.reinterpret_as_int()
  let workers = s.workers.copy()
  let tasks = s.tasks.copy()
  let s = { ..s, workers, tasks, }
  match workers[agent] {
    None =>
      if s.queue.is_empty() {
        @definition.Halt(s.env)
      } else {
        let id = s.queue[0]
        workers[agent] = Some(id)
        @definition.Continue(
          { ..s, queue: s.queue[1:].to_owned(), },
          "agent \{agent}: dequeue task \{id}",
        )
      }
    Some(id) => {
      @utils.require(
        id >= 0 && id < tasks.length(),
        "unknown task " + id.to_string(),
      )
      let frames = tasks[id]
      if frames.is_empty() {
        workers[agent] = None
        return @definition.Continue(s, "agent \{agent}: release task \{id}")
      }
      let frame = frames[0]
      let rest = frames[1:].to_owned()
      let f = find_function(s.code, frame.function_name)
      if frame.pc == f.body.length() {
        tasks[id] = rest
        return @definition.Continue(
          s,
          "agent \{agent}: return from " + frame.function_name,
        )
      }
      @utils.require(
        frame.pc >= 0 && frame.pc < f.body.length(),
        "pc \{frame.pc} out of bounds in " + frame.function_name,
      )
      let stmt = f.body[frame.pc]
      fn shift(offset : Int32) -> Unit raise @utils.Fault {
        let wide = frame.pc.to_int64() + offset.to_int64()
        @utils.require(
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
              vars: @utils.env_sorted(@utils.env_set(s.env.vars, n, value)),
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
              task_ids: @utils.env_sorted(
                @utils.env_set(s.env.task_ids, id_var, Some(new_id)),
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
      @definition.Continue(next, "agent \{agent}: " + stext(stmt))
    }
  }
}

///|
fn function_json(f : FunctionDef) -> Json {
  @utils.object([
    ("name", @utils.jstr(f.name)),
    ("body", @utils.jarray(f.body.map(stmt_json))),
  ])
}

///|
fn decode_function(j : Json) -> FunctionDef raise @utils.Fault {
  {
    name: @utils.identifier(@utils.string(@utils.field(j, "name"))),
    body: @utils.array(@utils.field(j, "body")).map(decode_stmt),
  }
}

///|
fn env_json(env : Environment) -> Json {
  @utils.object([
    ("vars", @utils.env_json(env.vars, n => @utils.jstr(n.to_string()))),
    (
      "task_ids",
      @utils.env_json(env.task_ids, id => @utils.optional_json(id, @utils.jint)),
    ),
  ])
}

///|
fn optional_id(j : Json) -> Int32? raise @utils.Fault {
  match j {
    Null => None
    _ => {
      let id = @utils.int(j)
      @utils.require(id >= 0, "Invalid task ID")
      Some(id)
    }
  }
}

///|
fn decode_env(j : Json) -> Environment raise @utils.Fault {
  {
    vars: @utils.decode_env(@utils.field(j, "vars"), @utils.json_nat),
    task_ids: @utils.decode_env(@utils.field(j, "task_ids"), optional_id),
  }
}

///|
fn frame_json(f : Frame) -> Json {
  @utils.object([
    ("function_name", @utils.jstr(f.function_name)),
    ("pc", @utils.jint(f.pc)),
  ])
}

///|
fn decode_frame(j : Json) -> Frame raise @utils.Fault {
  {
    function_name: @utils.identifier(
      @utils.string(@utils.field(j, "function_name")),
    ),
    pc: @utils.int(@utils.field(j, "pc")),
  }
}

///|
fn encode(s : State) -> Json {
  @utils.object([
    ("code", @utils.jarray(s.code.map(function_json))),
    ("env", env_json(s.env)),
    (
      "workers",
      @utils.jarray(s.workers.map(id => @utils.optional_json(id, @utils.jint))),
    ),
    ("queue", @utils.jarray(s.queue.map(@utils.jint))),
    (
      "tasks",
      @utils.jarray(s.tasks.map(fs => @utils.jarray(fs.map(frame_json)))),
    ),
  ])
}

///|
fn decode(j : Json) -> State raise @utils.Fault {
  let code = @utils.array(@utils.field(j, "code")).map(decode_function)
  validate(code)
  let workers = @utils.array(@utils.field(j, "workers")).map(optional_id)
  let queue = @utils.array(@utils.field(j, "queue")).map(@utils.int)
  let tasks = @utils.array(@utils.field(j, "tasks")).map(j => {
    @utils.array(j).map(decode_frame)
  })
  for id in queue {
    @utils.require(
      id >= 0 && id < tasks.length(),
      "Unknown task ID in snapshot",
    )
  }
  for id in workers {
    match id {
      Some(id) =>
        @utils.require(id < tasks.length(), "Unknown task ID in snapshot")
      _ => ()
    }
  }
  { code, env: decode_env(@utils.field(j, "env")), workers, queue, tasks, }
}

///|
fn render(s : State) -> Array[@definition.Block] {
  [
    @definition.text_table("status", ["field", "value"], [
      ["agents", s.workers.length().to_string()],
      ["queue_len", s.queue.length().to_string()],
      ["task_count", s.tasks.length().to_string()],
    ]),
    @definition.container(
      "env",
      [
        @definition.text_table(
          "vars",
          ["var", "value"],
          s.env.vars.map(p => [p.0, p.1.to_string()]),
        ),
        @definition.text_table(
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
      orientation=@definition.Horizontal,
    ),
    @definition.text_table(
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
    @definition.text_table(
      "queue",
      ["slot", "task"],
      s.queue.mapi((i, id) => [i.to_string(), id.to_string()]),
    ),
    @definition.container(
      "tasks",
      s.tasks.mapi((id, fs) => {
        @definition.text_table(
          "task \{id}",
          ["depth", "function", "pc"],
          fs
          .rev()
          .mapi((i, f) => [i.to_string(), f.function_name, f.pc.to_string()]),
        )
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
