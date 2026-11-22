///|
#derive(Eq)
pub(all) enum Target {
  RegionTarget(Int32)
  BlockTarget(Int32)
}

///|
#derive(Eq)
pub(all) enum Location[D] {
  Static(D)
  Stack(Int32)
  Heap(Int32, Int32)
}

///|
#derive(Eq)
pub(all) enum Value[L, D] {
  Num(@utils.Nat)
  Code_label(L)
  Key(Location[D])
}

///|
#derive(Eq)
pub(all) struct Resolution {
  registers : Array[(String, Int32)]
  data : Array[(String, Int32)]
  regions : Array[(String, Int32)]
  blocks : Array[Array[(String, Int32)]]
  label_names : Array[(Target, String)]
}

///|
type RuntimeValue = Value[Target, Int32]

///|
pub type FlowValue = Value[String, String]

///|
pub fn validate(c : Code) -> Unit raise @utils.Fault {
  @utils.unique(c.statics.map(s => s.label), "Duplicate static label: ")
  @utils.unique(c.regions.map(r => r.label), "Duplicate region label: ")
  for r in c.regions {
    @utils.require(
      !r.blocks.is_empty(),
      "Region :" + r.label + " has no blocks",
    )
    @utils.unique(
      r.blocks.map(b => b.label),
      "Duplicate block label in region :" + r.label + ": ",
    )
  }
  @utils.require(
    c.regions.any(r => r.label == "main"),
    "Missing entry region: :main",
  )
}

///|
pub fn[T] id_of_name(
  entries : Array[(String, T)],
  name : String,
) -> T raise @utils.Fault {
  match @utils.env_get(entries, name) {
    Some(id) => id
    None => @utils.error("Unknown name: " + name)
  }
}

///|
pub fn[T : Eq] name_of_id(
  entries : Array[(String, T)],
  id : T,
) -> String raise @utils.Fault {
  match entries.iter().find_first(p => p.1 == id) {
    Some(p) => p.0
    None => @utils.error("Unknown resolved ID")
  }
}

///|
fn register_values(v : ValueExpr, register : (String) -> Unit) -> Unit {
  match v {
    VReg(r) => register(r)
    Ref(p) => register_place(p, register)
    _ => ()
  }
}

///|
fn register_place(p : PlaceExpr, register : (String) -> Unit) -> Unit {
  match p {
    Deref(r) => register(r)
    SAcc(v) => register_values(v, register)
    HAcc(a, b) => {
      register_values(a, register)
      register_values(b, register)
    }
    _ => ()
  }
}

///|
pub fn build_resolution(c : Code) -> Resolution {
  let registers : Array[(String, Int32)] = []
  let label_names = []
  let labels = @utils.supply()
  fn register(name : String) -> Unit {
    if !registers.any(p => p.0 == name) {
      registers.push((name, registers.length()))
    }
  }
  fn value(v : ValueExpr) -> Unit {
    register_values(v, register)
  }
  fn place(p : PlaceExpr) -> Unit {
    register_place(p, register)
  }
  fn label(t : Target, hint : String) -> Unit {
    label_names.push((t, labels.fresh(hint)))
  }
  let regions = c.regions.mapi((id, r) => {
    label(RegionTarget(id), r.label)
    (r.label, id)
  })
  let count : Ref[Int32] = { val: 0, }
  let blocks = c.regions.map(r => {
    r.blocks.map(b => {
      for s in b.stmts {
        match s {
          Load(r, p) => {
            register(r)
            place(p)
          }
          Assign(r, v) | HAlloc(v, r) => {
            register(r)
            value(v)
          }
          BinOp(r, a, _, b) => {
            register(r)
            value(a)
            value(b)
          }
          Store(p, v) => {
            place(p)
            value(v)
          }
          Pop(r) | LGet(r) | Print(r) => register(r)
          Push(v) | HFree(v) => value(v)
          Input(p) => place(p)
          Nop => ()
        }
      }
      for j in ifs(b.cont) {
        match j.cond {
          Lt(a, b) | Eq(a, b) | Gt(a, b) => {
            value(a)
            value(b)
          }
        }
        value(j.target)
      }
      match b.cont {
        Go(_, v) | Enter(_, v) => value(v)
        Halt(_) => ()
      }
      let id = count.val
      count.val += 1
      label(BlockTarget(id), b.label)
      (b.label, id)
    })
  })
  {
    registers,
    data: c.statics.mapi((i, d) => (d.label, i)),
    regions,
    blocks,
    label_names,
  }
}

///|
pub fn local_target(
  r : Resolution,
  region : Int32,
  name : String,
) -> Target raise @utils.Fault {
  @utils.require(
    region >= 0 && region < r.blocks.length(),
    "Current region out of range",
  )
  BlockTarget(id_of_name(r.blocks[region], name))
}

///|
pub fn region_target(
  r : Resolution,
  name : String,
) -> Target raise @utils.Fault {
  RegionTarget(id_of_name(r.regions, name))
}

///|
pub fn resolve_target(
  r : Resolution,
  region : Int32,
  name : String,
) -> Target raise @utils.Fault {
  match @utils.env_get(r.regions, name) {
    Some(id) => RegionTarget(id)
    None => {
      @utils.require(
        region >= 0 && region < r.blocks.length(),
        "Current region out of range",
      )
      match @utils.env_get(r.blocks[region], name) {
        Some(id) => BlockTarget(id)
        None =>
          match r.label_names.iter().find_first(p => p.1 == name) {
            Some(p) => p.0
            None => local_target(r, region, name)
          }
      }
    }
  }
}

///|
pub fn target_name(
  r : Resolution,
  target : Target,
) -> String raise @utils.Fault {
  match r.label_names.iter().find_first(p => p.0 == target) {
    Some(p) => p.1
    None => @utils.error("Unknown resolved ID")
  }
}

///|
fn resolve_input(
  r : Resolution,
  region : Int32,
  v : FlowValue,
) -> RuntimeValue raise @utils.Fault {
  match v {
    Num(n) => Num(n)
    Code_label(n) => Code_label(resolve_target(r, region, n))
    Key(Static(n)) => Key(Static(id_of_name(r.data, n)))
    Key(Stack(i)) => Key(Stack(i))
    Key(Heap(h, i)) => Key(Heap(h, i))
  }
}

///|
fn named_value(
  r : Resolution,
  v : RuntimeValue,
) -> FlowValue raise @utils.Fault {
  match v {
    Num(n) => Num(n)
    Code_label(t) => Code_label(target_name(r, t))
    Key(Static(id)) => Key(Static(name_of_id(r.data, id)))
    Key(Stack(i)) => Key(Stack(i))
    Key(Heap(h, i)) => Key(Heap(h, i))
  }
}

///|
pub fn flow_text(v : FlowValue) -> String {
  match v {
    Num(n) => n.to_string()
    Code_label(l) => ":" + l
    Key(Static(l)) => "\\static @" + l
    Key(Stack(i)) => "\\stack[\{i}]"
    Key(Heap(h, i)) => "\\heap[\{h}][\{i}]"
  }
}

///|
fn input_value(text : String) -> FlowValue raise @utils.Fault {
  let p = @utils.create(text)
  fn index(p : @utils.Parser) -> Int32 raise @utils.Fault {
    let g = p.group('[')
    let i = g.integer()
    g.finish()
    i
  }
  let v = if p.eat(':') {
    Code_label(p.ident())
  } else if p.eat_keyword("static") {
    p.expect('@')
    Key(Static(p.ident()))
  } else if p.eat_keyword("stack") {
    Key(Stack(index(p)))
  } else if p.eat_keyword("heap") {
    let h = index(p)
    Key(Heap(h, index(p)))
  } else {
    Num(p.natural())
  }
  p.finish()
  v
}

///|
pub fn parse_input(
  text : String,
) -> Array[(String, FlowValue)] raise @utils.Fault {
  @utils.env_sorted(@utils.parse_env(text, input_value))
}
