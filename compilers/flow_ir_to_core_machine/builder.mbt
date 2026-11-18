///|
type Triple = (@c.ValueExpr, @c.ValueExpr, @c.ValueExpr)

///|
priv struct Builder {
  mut serial : Int32
  mut label : String
  mut stmts : Array[@c.Stmt]
  blocks : Array[@c.Block]
}

///|
fn imm(n : Int32) -> @c.ValueExpr {
  @c.Imm(@base.nint(n))
}

///|
fn fresh(c : Builder, prefix : String) -> String {
  c.serial += 1
  prefix + c.serial.to_string()
}

///|
fn emit(c : Builder, stmt : @c.Stmt) -> Unit {
  c.stmts.push(stmt)
}

///|
fn finish(c : Builder, ifs : Array[@c.JumpIf], ending : @c.Ending) -> Unit {
  c.blocks.push({ label: c.label, stmts: c.stmts, ifs, ending, })
  c.stmts = []
}

///|
fn go(c : Builder, label : String) -> Unit {
  finish(c, [], @c.Goto(@c.Label(label)))
}

///|
fn start(c : Builder, label : String) -> Unit {
  c.label = label
  c.stmts = []
}

///|
fn bind(c : Builder, rhs : @c.Rhs) -> @c.ValueExpr {
  let r = fresh(c, "tmp")
  emit(c, @c.Assign(r, rhs))
  @c.Reg(r)
}

///|
fn set(c : Builder, register : String, value : @c.ValueExpr) -> Unit {
  emit(c, @c.Assign(register, @c.Value(value)))
}

///|
fn binary(
  c : Builder,
  a : @c.ValueExpr,
  op : @c.Binop,
  b : @c.ValueExpr,
) -> @c.ValueExpr {
  bind(c, @c.Binop(a, op, b))
}

///|
fn branch(c : Builder, condition : @c.Cond, yes : String, no : String) -> Unit {
  finish(
    c,
    [{ cond: condition, target: @c.Label(yes), }],
    @c.Goto(@c.Label(no)),
  )
}

///|
fn require(c : Builder, condition : @c.Cond) -> Unit {
  let ok = fresh(c, "checked")
  branch(c, condition, ok, "runtime_error")
  start(c, ok)
}

///|
fn times3(c : Builder, n : @c.ValueExpr) -> @c.ValueExpr {
  let twice = binary(c, n, @c.Add, n)
  binary(c, twice, @c.Add, n)
}

///|
fn addr(c : Builder, memory : Int32, index : @c.ValueExpr) -> @c.ValueExpr {
  bind(c, @c.Addr(@c.Memory(memory), index))
}

///|
fn load(c : Builder, memory : Int32, index : @c.ValueExpr) -> @c.ValueExpr {
  let p = addr(c, memory, index)
  bind(c, @c.Load(@c.Deref(p)))
}

///|
fn store(
  c : Builder,
  memory : Int32,
  index : @c.ValueExpr,
  value : @c.ValueExpr,
) -> Unit {
  let p = addr(c, memory, index)
  emit(c, @c.Store(@c.Deref(p), value))
}

///|
fn offset(c : Builder, i : @c.ValueExpr, n : Int32) -> @c.ValueExpr {
  if n == 0 {
    i
  } else {
    binary(c, i, @c.Add, imm(n))
  }
}

///|
fn read3(c : Builder, memory : Int32, index : @c.ValueExpr) -> Triple {
  let t = load(c, memory, index)
  let a = load(c, memory, offset(c, index, 1))
  let b = load(c, memory, offset(c, index, 2))
  (t, a, b)
}

///|
fn write3(
  c : Builder,
  memory : Int32,
  index : @c.ValueExpr,
  value : Triple,
) -> Unit {
  store(c, memory, index, value.0)
  store(c, memory, offset(c, index, 1), value.1)
  store(c, memory, offset(c, index, 2), value.2)
}

///|
fn regs(register : String) -> (String, String, String) {
  let n = @data.of_bytes(register).to_string()
  ("v" + n + "t", "v" + n + "a", "v" + n + "b")
}

///|
fn reg_value(register : String) -> Triple {
  let (t, a, b) = regs(register)
  (@c.Reg(t), @c.Reg(a), @c.Reg(b))
}

///|
fn copy(c : Builder, register : String, value : Triple) -> Unit {
  let (t, a, b) = regs(register)
  set(c, t, value.0)
  set(c, a, value.1)
  set(c, b, value.2)
}

///|
fn numeric(c : Builder, value : Triple) -> @c.ValueExpr {
  require(c, (value.0, @c.Eq, imm(0)))
  value.1
}

///|
fn index(c : Builder, value : Triple) -> @c.ValueExpr {
  let n = numeric(c, value)
  require(c, (n, @c.Lt, @c.Imm(@base.nint(2147483647) + @base.one())))
  n
}

///|
fn[T] choose(
  c : Builder,
  key : @c.ValueExpr,
  cases : Array[(@c.ValueExpr, T)],
  action : (T) -> Unit raise @base.Fault,
) -> Unit raise @base.Fault {
  let join = fresh(c, "join")
  let cases = cases.map(entry => (entry.0, entry.1, fresh(c, "case")))
  finish(
    c,
    cases.map(entry => {
      cond: (key, @c.Eq, entry.0),
      target: @c.Label(entry.2),
    }),
    @c.Goto(@c.Label("runtime_error")),
  )
  for entry in cases {
    start(c, entry.2)
    action(entry.1)
    go(c, join)
  }
  start(c, join)
}

///|
fn block_label(region : Int32, block : Int32) -> String {
  "block_\{region}_\{block}"
}

///|
fn region_label(region : Int32) -> String {
  "region_\{region}"
}
