///|
priv struct State {
  code : Code
  current_region : Int32
  current_block : Int32
  current_line : Int32
  resolution : Resolution
  vregs : Array[(Int32, RuntimeValue)]
  static_mem : Array[(Int32, RuntimeValue)]
  stack : Array[RuntimeValue]
  heap : Array[(Int32, Array[RuntimeValue])]
  next_handle : Int32
  halted : Bool
}

///|
fn[T] assoc_get(entries : Array[(Int32, T)], id : Int32) -> T? {
  for entry in entries {
    if entry.0 == id {
      return Some(entry.1)
    }
  }
  None
}

///|
fn[T] assoc_set(
  entries : Array[(Int32, T)],
  id : Int32,
  value : T,
) -> Array[(Int32, T)] {
  let updated = entries.map(entry => {
    (entry.0, if entry.0 == id { value } else { entry.1 })
  })
  if !entries.any(entry => entry.0 == id) {
    updated.push((id, value))
  }
  updated
}

///|
fn[T] checked_at(
  items : Array[T],
  index : Int32,
  message : String,
) -> T raise @base.Fault {
  @base.require(index >= 0 && index < items.length(), message)
  items[index]
}

///|
fn[T] replace_at(
  items : Array[T],
  index : Int32,
  value : T,
  message : String,
) -> Array[T] raise @base.Fault {
  ignore(checked_at(items, index, message))
  let updated = items.copy()
  updated[index] = value
  updated
}

///|
fn runtime_text(state : State, value : RuntimeValue) -> String {
  flow_text(named_value(state.resolution, value)) catch {
    @base.Fault(message) => "<invalid: " + message + ">"
  }
}

///|
fn register_id(state : State, name : String) -> Int32 raise @base.Fault {
  id_of_name(state.resolution.registers, name)
}

///|
fn reg(state : State, name : String) -> RuntimeValue raise @base.Fault {
  assoc_get(state.vregs, register_id(state, name)).unwrap_or(Num(@base.zero()))
}

///|
fn put_register(
  state : State,
  name : String,
  value : RuntimeValue,
) -> State raise @base.Fault {
  { ..state, vregs: assoc_set(state.vregs, register_id(state, name), value), }
}

///|
fn number(state : State, value : RuntimeValue) -> @base.Nat raise @base.Fault {
  match value {
    Num(n) => n
    _ => @base.error("Expected number, got " + runtime_text(state, value))
  }
}

///|
fn index(state : State, value : RuntimeValue) -> Int32 raise @base.Fault {
  @base.index(number(state, value))
}

///|
fn load_key(
  state : State,
  key : Location[Int32],
) -> RuntimeValue raise @base.Fault {
  match key {
    Static(id) =>
      match assoc_get(state.static_mem, id) {
        Some(value) => value
        None =>
          @base.error(
            "Unknown data label: @" + name_of_id(state.resolution.data, id),
          )
      }
    Stack(i) => checked_at(state.stack, i, "Invalid stack key: s[\{i}]")
    Heap(handle, i) =>
      checked_at(
        assoc_get(state.heap, handle).unwrap_or([]),
        i,
        "Invalid heap key: h[\{handle}][\{i}]",
      )
  }
}

///|
fn write_key(
  state : State,
  key : Location[Int32],
  value : RuntimeValue,
) -> State raise @base.Fault {
  match key {
    Static(id) => {
      ignore(load_key(state, key))
      { ..state, static_mem: assoc_set(state.static_mem, id, value), }
    }
    Stack(i) =>
      {
        ..state,
        stack: replace_at(state.stack, i, value, "Invalid stack key: s[\{i}]"),
      }
    Heap(handle, i) => {
      let area = match assoc_get(state.heap, handle) {
        Some(area) => area
        None => @base.error("Unknown heap handle: \{handle}")
      }
      let area = replace_at(
        area,
        i,
        value,
        "Invalid heap key: h[\{handle}][\{i}]",
      )
      { ..state, heap: assoc_set(state.heap, handle, area), }
    }
  }
}

///|
fn eval_value(
  state : State,
  expression : ValueExpr,
) -> RuntimeValue raise @base.Fault {
  match expression {
    VReg(name) => reg(state, name)
    Imm(n) => Num(n)
    CodeLabel(name) =>
      Code_label(resolve_target(state.resolution, state.current_region, name))
    Ref(place) => Key(eval_place(state, place))
  }
}

///|
fn eval_place(
  state : State,
  expression : PlaceExpr,
) -> Location[Int32] raise @base.Fault {
  match expression {
    Label(name) => {
      let id = id_of_name(state.resolution.data, name)
      @base.require(
        assoc_get(state.static_mem, id) is Some(_),
        "Place is not data label: @" + name,
      )
      Static(id)
    }
    Deref(name) =>
      match reg(state, name) {
        Key(key) => key
        value =>
          @base.error(
            "deref expects place key in vreg, got " + runtime_text(state, value),
          )
      }
    SAcc(value) => Stack(index(state, eval_value(state, value)))
    HAcc(handle, offset) =>
      Heap(
        index(state, eval_value(state, handle)),
        index(state, eval_value(state, offset)),
      )
  }
}

///|
fn eval_cond(state : State, condition : Cond) -> Bool raise @base.Fault {
  let (left, right, comparison) = match condition {
    Lt(a, b) => (a, b, -1)
    Eq(a, b) => (a, b, 0)
    Gt(a, b) => (a, b, 1)
  }
  let left = eval_value(state, left)
  let right = eval_value(state, right)
  match (left, right) {
    (Num(a), Num(b)) =>
      if comparison == 0 {
        a == b
      } else if comparison < 0 {
        a < b
      } else {
        a > b
      }
    _ if comparison == 0 => left == right
    _ =>
      @base.error(
        "Condition " +
        (if comparison < 0 { "lt" } else { "gt" }) +
        " supports only numbers, got " +
        runtime_text(state, left) +
        " and " +
        runtime_text(state, right),
      )
  }
}
