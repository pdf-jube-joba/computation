///|
fn[K : Eq, V] assoc_get(entries : Array[(K, V)], key : K) -> V? {
  for entry in entries {
    if entry.0 == key {
      return Some(entry.1)
    }
  }
  None
}

///|
fn[K : Eq, V] assoc_set(
  entries : Array[(K, V)],
  key : K,
  value : V,
) -> Array[(K, V)] {
  let mut found = false
  let result = entries.map(entry => {
    if entry.0 == key {
      found = true
      (key, value)
    } else {
      entry
    }
  })
  if !found {
    result.push((key, value))
  }
  result
}

///|
fn[T] at(values : Array[T], i : Int32) -> T raise @utils.Fault {
  @utils.require(i >= 0 && i < values.length(), "projection out of bounds")
  values[i]
}

///|
fn type_match(v : Value, ty : Ty) -> Bool {
  match (v, ty) {
    (Number_value(_), Num) => true
    (Number_value(n), U8) => n <= @utils.nint(255)
    (Bool_value(_), Bool)
    | (Unit_v, Unit)
    | (Ptr_value(_), Ptr)
    | (FnPtr(_), Fn) => true
    (Aggregate(vs), Product(ts)) => {
      if vs.length() != ts.length() {
        return false
      }
      for i, v in vs {
        if !type_match(v, ts[i]) {
          return false
        }
      }
      true
    }
    (Aggregate(vs), Array(t, n)) =>
      vs.length() == n && vs.all(v => type_match(v, t))
    (Sum_value(tag, v), Sum(ts)) =>
      tag >= 0 && tag < ts.length() && type_match(v, ts[tag])
    _ => false
  }
}

///|
fn default_value(ty : Ty) -> Value raise @utils.Fault {
  match ty {
    Num | U8 => Number_value(@utils.zero())
    Bool => Bool_value(false)
    Unit => Unit_v
    Ptr => Ptr_value(None)
    Fn => FnPtr(None)
    Product(ts) => Aggregate(ts.map(default_value))
    Array(t, n) => {
      @utils.require(n >= 0, "negative array length")
      Aggregate(Array::makei(n, _ => default_value(t)))
    }
    Sum(ts) => {
      @utils.require(!ts.is_empty(), "sum type must have at least one variant")
      Sum_value(0, default_value(ts[0]))
    }
  }
}

///|
fn project(
  ty : Ty,
  value : Value,
  projections : Array[Projection],
) -> (Ty, Value) raise @utils.Fault {
  if projections.is_empty() {
    return (ty, value)
  }
  let rest = projections[1:].to_owned()
  match (projections[0], ty, value) {
    (Field_proj(i), Product(ts), Aggregate(vs)) =>
      project(at(ts, i), at(vs, i), rest)
    (Index_proj(i), Array(t, n), Aggregate(vs)) => {
      @utils.require(i < n, "projection out of bounds")
      project(t, at(vs, i), rest)
    }
    (Tag_proj(i), Sum(ts), Sum_value(tag, v)) => {
      @utils.require(i == tag, "sum tag mismatch")
      project(at(ts, i), v, rest)
    }
    _ => @utils.error("invalid place projection")
  }
}

///|
fn update_projection(
  ty : Ty,
  current : Value,
  projections : Array[Projection],
  value : Value,
) -> Value raise @utils.Fault {
  if projections.is_empty() {
    @utils.require(
      type_match(value, ty),
      "assigned value does not match location type",
    )
    return value
  }
  let rest = projections[1:].to_owned()
  match (projections[0], ty, current) {
    (Field_proj(i), Product(ts), Aggregate(vs)) => {
      let t = at(ts, i)
      let old = at(vs, i)
      let vs = vs.copy()
      vs[i] = update_projection(t, old, rest, value)
      Aggregate(vs)
    }
    (Index_proj(i), Array(t, n), Aggregate(vs)) => {
      @utils.require(i < n, "array index out of bounds")
      let old = at(vs, i)
      let vs = vs.copy()
      vs[i] = update_projection(t, old, rest, value)
      Aggregate(vs)
    }
    (Tag_proj(i), Sum(ts), Sum_value(tag, v)) => {
      @utils.require(i == tag, "sum tag mismatch")
      Sum_value(tag, update_projection(at(ts, i), v, rest, value))
    }
    _ => @utils.error("invalid assignment target")
  }
}

///|
fn get_cell(s : State, key : Key) -> Cell raise @utils.Fault {
  let cell = match key {
    Heap(id) => assoc_get(s.store.heap, id)
    Static_key(name) => assoc_get(s.store.statics, name)
    Local_key(id) => {
      let mut found = None
      for scopes in s.store.locals {
        for cells in scopes {
          if assoc_get(cells, id) is Some(c) {
            found = Some(c)
            break
          }
        }
        if found is Some(_) {
          break
        }
      }
      found
    }
  }
  match cell {
    Some(c) => c
    None => @utils.error("unknown location: " + key_text(key))
  }
}

///|
fn set_cell(s : State, key : Key, cell : Cell) -> State raise @utils.Fault {
  ignore(get_cell(s, key))
  let store = match key {
    Heap(id) => { ..s.store, heap: assoc_set(s.store.heap, id, cell), }
    Static_key(name) =>
      { ..s.store, statics: assoc_set(s.store.statics, name, cell), }
    Local_key(id) => {
      let mut updated = false
      let locals = s.store.locals.map(scopes => {
        scopes.map(cells => {
          if !updated && assoc_get(cells, id) is Some(_) {
            updated = true
            assoc_set(cells, id, cell)
          } else {
            cells
          }
        })
      })
      { ..s.store, locals, }
    }
  }
  { ..s, store, }
}

///|
fn store_assign(
  s : State,
  loc : Location,
  value : Value,
) -> State raise @utils.Fault {
  let cell = get_cell(s, loc.key)
  set_cell(s, loc.key, {
    ..cell,
    value: update_projection(cell.ty, cell.value, loc.projections, value),
  })
}

///|
fn function_lookup(
  code : Code,
  name : String,
) -> FunctionDecl raise @utils.Fault {
  for f in code.functions {
    if f.name == name {
      return f
    }
  }
  @utils.error("unknown function: " + name)
}

///|
fn eval_binop(op : Binop, a : Value, b : Value) -> Value raise @utils.Fault {
  match (op, a, b) {
    (Add, Number_value(a), Number_value(b)) => Number_value(a + b)
    (Sub, Number_value(a), Number_value(b)) => {
      @utils.require(a >= b, "operator '-' underflowed")
      Number_value(a - b)
    }
    (Eq, Number_value(a), Number_value(b)) => Bool_value(a == b)
    (Eq, Char_value(a), Char_value(b)) => Bool_value(a == b)
    (Eq, Bool_value(a), Bool_value(b)) => Bool_value(a == b)
    (Eq, Unit_v, Unit_v) => Bool_value(true)
    (Eq, Ptr_value(a), Ptr_value(b)) => Bool_value(a == b)
    (Eq, FnPtr(a), FnPtr(b)) => Bool_value(a == b)
    (Lt, Number_value(a), Number_value(b)) => Bool_value(a < b)
    (And, Bool_value(a), Bool_value(b)) => Bool_value(a && b)
    (Or, Bool_value(a), Bool_value(b)) => Bool_value(a || b)
    _ => @utils.error("operator applied to incompatible values")
  }
}

///|
fn eval_place(s : State, p : Place) -> Location raise @utils.Fault {
  match p {
    Static(n) => {
      @utils.require(
        assoc_get(s.store.statics, n) is Some(_),
        "unknown static: " + n,
      )
      { key: Static_key(n), projections: [], }
    }
    Local(n) => {
      for scope in s.env {
        if assoc_get(scope, n) is Some(id) {
          return { key: Local_key(id), projections: [], }
        }
      }
      @utils.error("unknown local: " + n)
    }
    Field(p, i) => {
      let loc = eval_place(s, p)
      { ..loc, projections: loc.projections + [Field_proj(i)], }
    }
    Tag_place(p, i) => {
      let loc = eval_place(s, p)
      { ..loc, projections: loc.projections + [Tag_proj(i)], }
    }
    Index(p, v) => {
      let i = match eval_value(s, v) {
        Number_value(n) => @utils.index(n)
        _ => @utils.error("array index must be a number")
      }
      let loc = eval_place(s, p)
      { ..loc, projections: loc.projections + [Index_proj(i)], }
    }
    Deref(v) =>
      match eval_value(s, v) {
        Ptr_value(Some(loc)) => loc
        Ptr_value(None) => @utils.error("cannot dereference \\null_ptr")
        _ => @utils.error("dereference expects a pointer")
      }
  }
}

///|
fn eval_value(s : State, v : Expr) -> Value raise @utils.Fault {
  match v {
    Number(n) => Number_value(n)
    Char(c) => Char_value(c)
    Boolean(b) => Bool_value(b)
    Unit_value => Unit_v
    NullPtr => Ptr_value(None)
    NullFn => FnPtr(None)
    BinOp(a, op, b) => {
      let a = eval_value(s, a)
      let b = eval_value(s, b)
      eval_binop(op, a, b)
    }
    UnOp(Not, v) =>
      match eval_value(s, v) {
        Bool_value(b) => Bool_value(!b)
        _ => @utils.error("operator '!' expects a bool")
      }
    Pair(vs) => Aggregate(vs.map(v => eval_value(s, v)))
    Tag(t, v) => Sum_value(t, eval_value(s, v))
    Load(p) => {
      let loc = eval_place(s, p)
      let cell = get_cell(s, loc.key)
      project(cell.ty, cell.value, loc.projections).1
    }
    Addr(p) => Ptr_value(Some(eval_place(s, p)))
    Function(n) => {
      ignore(function_lookup(s.code, n))
      FnPtr(Some(n))
    }
  }
}
