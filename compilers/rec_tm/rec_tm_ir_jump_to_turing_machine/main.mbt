///|
/// Solve backwards liveness before enumerating valuations of live variables.
fn live_variables(body : Array[@source.Stmt]) -> Array[Array[String]] {
  let n = body.length()
  let live : Array[Array[String]] = Array::makei(n + 1, _ => [])
  let mut changed = true
  while changed {
    changed = false
    for pc = n - 1; pc >= 0; pc = pc - 1 {
      let (uses, defines, successors) = match body[pc] {
        @source.Assign(dst, src) =>
          (@syntax.rvars(src), @syntax.lvars(dst), [pc + 1])
        @source.Jump(target, cond) =>
          (
            @syntax.cvars(cond),
            [],
            if cond is None {
              [target]
            } else {
              [target, pc + 1]
            },
          )
        _ => ([], [], [pc + 1])
      }
      let names : Map[String, Bool] = Map([])
      for name in uses {
        names[name] = true
      }
      for next in successors {
        for name in live[next] {
          if !defines.contains(name) {
            names[name] = true
          }
        }
      }
      let next = names.keys().collect()
      next.sort()
      if live[pc] != next {
        live[pc] = next
        changed = true
      }
    }
  }
  live
}

///|
pub fn compile(program : @source.Code) -> @target.Definition raise @base.Fault {
  @source.validate(program)
  let alphabet = @syntax.normalized_alphabet(program.alphabet)
  let vars = @source.collect_vars(program.body)
  let live = live_variables(program.body)
  let states : Map[String, Int32] = Map([])
  let work : Array[(Int32, Array[String], Int32)] = []
  let names : Array[(Int32, String)] = []
  let supply = @names.supply()
  fn state_for(pc : Int32, values : Array[String]) -> Int32 {
    let values = values.mapi((i, v) => {
      if live[pc].contains(vars[i]) {
        v
      } else {
        "-"
      }
    })
    let key = pc.to_string() + ":" + @base.join(values, ",")
    match states.get(key) {
      Some(id) => id
      None => {
        let id = states.length()
        states[key] = id
        names.push((id, supply.fresh("q")))
        work.push((pc, values, id))
        id
      }
    }
  }
  let initial = state_for(0, vars.map(_ => "-"))
  let code : Array[@target.Entry] = []
  let accepted = []
  let mut cursor = 0
  while cursor < work.length() {
    let (pc, values, state) = work[cursor]
    cursor += 1
    if pc == program.body.length() {
      accepted.push(state)
      continue
    }
    let environment = @syntax.env_create(vars)
    let env = { ..environment, values: values.mapi((i, v) => (i, v)), }
    for head in alphabet {
      let tape = @data.blank(head)
      let (next_pc, next_values, write, dir) = match program.body[pc] {
        @source.Lt => (pc + 1, values, head, -1)
        @source.Rt => (pc + 1, values, head, 1)
        @source.Assign(dst, src) => {
          let value = @syntax.eval(env, tape, src)
          match dst {
            @syntax.PutHead => (pc + 1, values, value, 0)
            @syntax.PutVar(name) =>
              (
                pc + 1,
                values.mapi((i, old) => {
                  if vars[i] == name {
                    value
                  } else {
                    old
                  }
                }),
                head,
                0,
              )
          }
        }
        @source.Jump(target, cond) =>
          (
            if @syntax.eval_condition(env, tape, cond) {
              target
            } else {
              pc + 1
            },
            values,
            head,
            0,
          )
      }
      let target = state_for(next_pc, next_values)
      code.push({
        reads: [head],
        source: state,
        writes: [write],
        target,
        directions: [dir],
      })
    }
  }
  if accepted.is_empty() {
    accepted.push(state_for(program.body.length(), vars.map(_ => "-")))
  }
  { count: 1, alphabet: [], initial, accepted, code, names, }
}

///|
pub fn compiler() -> @runtime.Compiler {
  @encoding.compiler(text => {
    @target.print_code(compile(@source.parse_code(text)))
  })
}
