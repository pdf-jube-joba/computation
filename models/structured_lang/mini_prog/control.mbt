///|
fn allocate_scope(
  s : State,
  bindings : Array[(String, Ty)],
  values : Array[Value],
) -> (Array[(String, Int32)], Array[(Int32, Cell)], Int32) raise @utils.Fault {
  @utils.require(bindings.length() == values.length(), "binding count mismatch")
  let mut next = s.next_local_id
  let mut env = []
  let mut cells = []
  for i, binding in bindings {
    @utils.require(
      type_match(values[i], binding.1),
      "type mismatch for " + binding.0,
    )
    @utils.require(next >= 0 && next < 2147483647, "Local ID exhausted")
    env = assoc_set(env, binding.0, next)
    cells = assoc_set(cells, next, { ty: binding.1, value: values[i], })
    next += 1
  }
  (env, cells, next)
}

///|
fn enter_function(
  s : State,
  name : String,
  args : Array[Value],
) -> State raise @utils.Fault {
  let f = function_lookup(s.code, name)
  @utils.require(
    f.block.bindings.length() == args.length(),
    "argument count mismatch for " + name,
  )
  let (env, cells, next_local_id) = allocate_scope(s, f.block.bindings, args)
  {
    ..s,
    current_function: name,
    env: [env],
    store: { ..s.store, locals: [[cells]] + s.store.locals, },
    inner_k: f.block.stmts.map(st => Stmt_frame(st)),
    next_local_id,
  }
}

///|
fn push_scope(
  s : State,
  bindings : Array[(String, Ty)],
  values : Array[Value],
) -> State raise @utils.Fault {
  @utils.require(
    !s.store.locals.is_empty(),
    "push-scope without current function frame",
  )
  let (env, cells, next_local_id) = allocate_scope(s, bindings, values)
  {
    ..s,
    env: [env] + s.env,
    next_local_id,
    store: {
      ..s.store,
      locals: [[cells] + s.store.locals[0]] + s.store.locals[1:].to_owned(),
    },
  }
}

///|
fn pop_scope(s : State) -> State raise @utils.Fault {
  @utils.require(
    !s.env.is_empty() &&
    !s.store.locals.is_empty() &&
    !s.store.locals[0].is_empty(),
    "scope underflow",
  )
  {
    ..s,
    env: s.env[1:].to_owned(),
    store: {
      ..s.store,
      locals: [s.store.locals[0][1:].to_owned()] + s.store.locals[1:].to_owned(),
    },
  }
}

///|
fn step_eval(s : State, stmt : Stmt) -> State raise @utils.Fault {
  match stmt {
    Assign(p, v) => {
      let loc = eval_place(s, p)
      let value = eval_value(s, v)
      { ..store_assign(s, loc, value), exec: Ctrl(Next), }
    }
    If(c, st) =>
      match eval_value(s, c) {
        Bool_value(b) => { ..s, exec: if b { Eval(st) } else { Ctrl(Next) }, }
        _ => @utils.error("if expects a bool")
      }
    Case(tag, v, st) =>
      match eval_value(s, v) {
        Sum_value(found, _) =>
          { ..s, exec: if tag == found { Eval(st) } else { Ctrl(Next) }, }
        _ => @utils.error("case expects a tagged sum")
      }
    HAlloc(ty, p) => {
      let id = s.next_heap_id
      @utils.require(id >= 0 && id < 2147483647, "Heap ID exhausted")
      let s = {
        ..s,
        next_heap_id: id + 1,
        store: {
          ..s.store,
          heap: assoc_set(s.store.heap, id, { ty, value: default_value(ty), }),
        },
      }
      let loc = eval_place(s, p)
      {
        ..store_assign(
          s,
          loc,
          Ptr_value(Some({ key: Heap(id), projections: [], })),
        ),
        exec: Ctrl(Next),
      }
    }
    HFree(v) => {
      let loc = match eval_value(s, v) {
        Ptr_value(Some(loc)) => loc
        _ => @utils.error("hfree expects a heap pointer")
      }
      match loc.key {
        Heap(id) if loc.projections.is_empty() => {
          @utils.require(
            assoc_get(s.store.heap, id) is Some(_),
            "hfree on unknown heap location",
          )
          {
            ..s,
            store: {
              ..s.store,
              heap: s.store.heap.filter(entry => entry.0 != id),
            },
            exec: Ctrl(Next),
          }
        }
        _ => @utils.error("hfree expects a top-level heap pointer")
      }
    }
    Loop(label, st) =>
      { ..s, inner_k: [Loop_frame(label, st)] + s.inner_k, exec: Ctrl(Next), }
    Break(label) => { ..s, exec: Ctrl(Break_ctrl(label)), }
    Continue(label) => { ..s, exec: Ctrl(Continue_ctrl(label)), }
    Block(b) => {
      let values = b.bindings.map(binding => default_value(binding.1))
      let s = push_scope(s, b.bindings, values)
      {
        ..s,
        inner_k: b.stmts.map(st => Stmt_frame(st)) + [Scope] + s.inner_k,
        exec: Ctrl(Next),
      }
    }
    Call(callee, args, rets) => {
      let ret_locs = rets.map(p => eval_place(s, p))
      let name = match eval_value(s, callee) {
        FnPtr(Some(n)) => n
        _ => @utils.error("call expects a function pointer")
      }
      let values = args.map(v => eval_value(s, v))
      let s = {
        ..s,
        fn_k: [
          {
            function_name: s.current_function,
            env: s.env,
            ret_locs,
            inner_k: s.inner_k,
          },
        ] +
        s.fn_k,
      }
      { ..enter_function(s, name, values), exec: Ctrl(Next), }
    }
    Return(vs) => { ..s, exec: Ret(vs.map(v => eval_value(s, v))), }
  }
}

///|
fn step_ctrl(s : State, control : Control) -> State raise @utils.Fault {
  @utils.require(!s.inner_k.is_empty(), "control state has no continuation")
  let frame = s.inner_k[0]
  let s = { ..s, inner_k: s.inner_k[1:].to_owned(), }
  match frame {
    Scope => { ..pop_scope(s), exec: Ctrl(control), }
    Stmt_frame(st) =>
      {
        ..s,
        exec: match control {
          Next => Eval(st)
          _ => Ctrl(control)
        },
      }
    Loop_frame(label, st) =>
      match control {
        Next => { ..s, inner_k: [frame] + s.inner_k, exec: Eval(st), }
        Break_ctrl(target) =>
          {
            ..s,
            exec: Ctrl(if target == label { Next } else { Break_ctrl(target) }),
          }
        Continue_ctrl(target) =>
          if target == label {
            { ..s, inner_k: [frame] + s.inner_k, exec: Ctrl(Next), }
          } else {
            // Preserve the original machine's outward propagation rule.
            { ..s, exec: Ctrl(Break_ctrl(target)), }
          }
      }
  }
}

///|
fn step_ret(s : State, values : Array[Value]) -> State raise @utils.Fault {
  @utils.require(!s.fn_k.is_empty(), "return without caller")
  let frame = s.fn_k[0]
  @utils.require(
    values.length() == frame.ret_locs.length(),
    "return count mismatch",
  )
  @utils.require(!s.store.locals.is_empty(), "local store underflow on return")
  let mut next = {
    ..s,
    fn_k: s.fn_k[1:].to_owned(),
    store: { ..s.store, locals: s.store.locals[1:].to_owned(), },
  }
  for i, loc in frame.ret_locs {
    next = store_assign(next, loc, values[i])
  }
  {
    ..next,
    current_function: frame.function_name,
    env: frame.env,
    inner_k: frame.inner_k,
    exec: Ctrl(Next),
  }
}
