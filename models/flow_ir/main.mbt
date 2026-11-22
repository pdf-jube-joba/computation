///|
fn make(code : Code, input : Environment) -> State raise @utils.Fault {
  validate(code)
  let resolution = build_resolution(code)
  let main = code.regions.search_by(region => region.label == "main").unwrap()
  let mut static_mem = code.statics.map(definition => {
    (id_of_name(resolution.data, definition.label), Num(definition.value))
  })
  for entry in input.values {
    let id = id_of_name(resolution.data, entry.0)
    static_mem = assoc_set(
      static_mem,
      id,
      resolve_input(resolution, main, entry.1),
    )
  }
  {
    code,
    resolution,
    static_mem,
    current_region: main,
    current_block: 0,
    current_line: 0,
    vregs: [],
    stack: [],
    heap: [],
    next_handle: 1,
    halted: false,
  }
}

///|
fn target(state : State, expression : ValueExpr) -> Target raise @utils.Fault {
  match eval_value(state, expression) {
    Code_label(target) => target
    value =>
      @utils.error(
        "jump target expects code label, got " + runtime_text(state, value),
      )
  }
}

///|
fn jump_go(state : State, expression : ValueExpr) -> State raise @utils.Fault {
  let destination = match expression {
    CodeLabel(name) =>
      local_target(state.resolution, state.current_region, name)
    _ => target(state, expression)
  }
  let id = match destination {
    BlockTarget(id) => id
    _ => @utils.error("goto expects a local block ID")
  }
  let blocks = checked_at(
    state.resolution.blocks,
    state.current_region,
    "Current region out of range",
  )
  let block = match blocks.search_by(entry => entry.1 == id) {
    Some(index) => index
    None => @utils.error("goto target is not in the current region")
  }
  { ..state, current_block: block, current_line: 0, }
}

///|
fn jump_enter(
  state : State,
  expression : ValueExpr,
) -> State raise @utils.Fault {
  let destination = match expression {
    CodeLabel(name) => region_target(state.resolution, name)
    _ => target(state, expression)
  }
  let id = match destination {
    RegionTarget(id) => id
    _ => @utils.error("enter expects a region ID")
  }
  let region = match
    state.resolution.regions.search_by(entry => entry.1 == id) {
    Some(index) => index
    None => @utils.error("unknown region ID")
  }
  // Enter changes the register scope. Stack, heap, and static memory survive.
  {
    ..state,
    current_region: region,
    current_block: 0,
    current_line: 0,
    vregs: [],
  }
}

///|
fn output(state : State) -> Environment raise @utils.Fault {
  let values = state.static_mem.map(entry => {
    (
      name_of_id(state.resolution.data, entry.0),
      named_value(state.resolution, entry.1),
    )
  })
  { values: @utils.env_sorted(values), }
}

///|
fn execute_statement(
  state : State,
  statement : Stmt,
  input : String,
) -> (State, String) raise @utils.Fault {
  match statement {
    Nop => (state, "")
    Assign(name, value) =>
      (put_register(state, name, eval_value(state, value)), "")
    Load(name, place) =>
      (put_register(state, name, load_key(state, eval_place(state, place))), "")
    BinOp(name, left, operation, right) => {
      let left = number(state, eval_value(state, left))
      let right = number(state, eval_value(state, right))
      let value = match operation {
        Add => left + right
        Sub => @utils.sub(left, right)
      }
      (put_register(state, name, Num(value)), "")
    }
    Store(place, value) =>
      (write_key(state, eval_place(state, place), eval_value(state, value)), "")
    Push(value) => {
      let stack = state.stack.copy()
      stack.push(eval_value(state, value))
      ({ ..state, stack, }, "")
    }
    Pop(name) => {
      @utils.require(!state.stack.is_empty(), "pop on empty stack")
      let last = state.stack.length() - 1
      let next = put_register(state, name, state.stack[last])
      ({ ..next, stack: state.stack[:last].to_owned(), }, "")
    }
    LGet(name) =>
      (
        put_register(
          state,
          name,
          Num(@utils.nint(state.stack.length().reinterpret_as_uint())),
        ),
        "",
      )
    HAlloc(size, name) => {
      let size = index(state, eval_value(state, size))
      @utils.require(state.next_handle < 2147483647, "ID supply exhausted")
      let handle = state.next_handle
      let next = put_register(
        state,
        name,
        Num(@utils.nint(handle.reinterpret_as_uint())),
      )
      (
        {
          ..next,
          next_handle: handle + 1,
          heap: assoc_set(
            state.heap,
            handle,
            Array::make(size, Num(@utils.zero())),
          ),
        },
        "",
      )
    }
    HFree(value) => {
      let handle = index(state, eval_value(state, value))
      @utils.require(
        assoc_get(state.heap, handle) is Some(_),
        "hfree unknown handle: \{handle}",
      )
      ({ ..state, heap: state.heap.filter(entry => entry.0 != handle), }, "")
    }
    Print(name) => (state, @utils.to_bytes(number(state, reg(state, name))))
    Input(place) =>
      (
        write_key(state, eval_place(state, place), Num(@utils.of_bytes(input))),
        "",
      )
  }
}

///|
fn step(
  state : State,
  input : Rinput,
) -> @definition.Transition[State, Routput, Foutput] raise @utils.Fault {
  if state.halted {
    return @definition.Halt(output(state))
  }
  let region = checked_at(
    state.code.regions,
    state.current_region,
    "Current region out of range",
  )
  let block = checked_at(
    region.blocks,
    state.current_block,
    "Current block out of range",
  )
  let statements = block.stmts.length()
  let jumps = ifs(block.cont)
  @utils.require(state.current_line >= 0, "Current line out of range")
  if state.current_line < statements {
    let statement = block.stmts[state.current_line]
    let next = { ..state, current_line: state.current_line + 1, }
    let (next, output) = execute_statement(next, statement, input)
    @definition.Continue(next, output)
  } else if state.current_line < statements + jumps.length() {
    let jump = jumps[state.current_line - statements]
    let next = { ..state, current_line: state.current_line + 1, }
    @definition.Continue(
      if eval_cond(next, jump.cond) {
        jump_go(next, jump.target)
      } else {
        next
      },
      "",
    )
  } else if state.current_line == statements + jumps.length() {
    match block.cont {
      Go(_, value) => @definition.Continue(jump_go(state, value), "")
      Enter(_, value) => @definition.Continue(jump_enter(state, value), "")
      Halt(_) => @definition.Halt(output(state))
    }
  } else {
    @utils.error("Current line out of range")
  }
}

///|
fn render(state : State) -> Array[@definition.Block] {
  let region = if state.current_region >= 0 &&
    state.current_region < state.code.regions.length() {
    Some(state.code.regions[state.current_region])
  } else {
    None
  }
  let block = match region {
    Some(region) if state.current_block >= 0 &&
      state.current_block < region.blocks.length() =>
      Some(region.blocks[state.current_block])
    _ => None
  }
  let vregs = state.vregs.copy()
  vregs.sort_by((a, b) => a.0.compare(b.0))
  let heap = state.heap.copy()
  heap.sort_by((a, b) => a.0.compare(b.0))
  let heap_rows = []
  for entry in heap {
    for index, value in entry.1 {
      heap_rows.push([
        entry.0.to_string(),
        index.to_string(),
        runtime_text(state, value),
      ])
    }
  }
  [
    @definition.titled(
      "current_region",
      ":" +
      (match region {
        Some(region) => region.label
        None => "<invalid>"
      }),
    ),
    @definition.titled(
      "current_block",
      ":" +
      (match block {
        Some(block) => block.label
        None => "<invalid>"
      }),
    ),
    @definition.titled("current_line", state.current_line.to_string()),
    @definition.titled("halted", state.halted.to_string()),
    @definition.text_table(
      "vregs",
      ["vreg", "value"],
      vregs.map(entry => {
        [
          "%" +
          (name_of_id(state.resolution.registers, entry.0) catch {
            _ => "<invalid>"
          }),
          runtime_text(state, entry.1),
        ]
      }),
    ),
    @definition.text_table(
      "static",
      ["label", "value"],
      state.code.statics.mapi((id, definition) => {
        [
          "@" + definition.label,
          runtime_text(
            state,
            assoc_get(state.static_mem, id).unwrap_or(Num(@utils.zero())),
          ),
        ]
      }),
    ),
    @definition.text_table(
      "stack",
      ["index", "value"],
      state.stack.mapi((index, value) => {
        [index.to_string(), runtime_text(state, value)]
      }),
    ),
    @definition.text_table("heap", ["handle", "index", "value"], heap_rows),
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
