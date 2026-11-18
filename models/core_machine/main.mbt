///|
#derive(Eq)
pub(all) enum SliceValue {
  Whole(Int32)
  Subslice(SliceValue, @base.Nat, @base.Nat)
}

///|
pub(all) enum RuntimeValue {
  Num(@base.Nat)
  CodeLabel(String)
  SliceValue(SliceValue)
  Address(SliceValue, @base.Nat)
}

///|
pub type Memory = Array[(@base.Nat, RuntimeValue)]

///|
priv struct State {
  code : Code
  block : String
  line : Int32
  registers : Array[(String, RuntimeValue)]
  memory : Array[Memory]
}

///|
fn origin(s : SliceValue) -> Int32 {
  match s {
    Whole(m) => m
    Subslice(s, _, _) => origin(s)
  }
}

///|
fn base(s : SliceValue) -> @base.Nat {
  match s {
    Whole(_) => @base.zero()
    Subslice(s, b, _) => base(s) + b
  }
}

///|
fn length(s : SliceValue) -> @base.Nat? {
  match s {
    Whole(_) => None
    Subslice(_, b, e) => Some(@base.sub(e, b))
  }
}

///|
fn within(s : SliceValue, i : @base.Nat) -> Bool {
  match length(s) {
    None => true
    Some(n) => i <= n
  }
}

///|
fn readable(s : SliceValue, i : @base.Nat) -> Bool {
  match length(s) {
    None => true
    Some(n) => i < n
  }
}

///|
fn validate_slice(c : Code, s : SliceValue) -> Unit raise @base.Fault {
  match s {
    Whole(m) => @base.require(m >= 0 && m < c.memories, "Unknown memory")
    Subslice(s, b, e) => {
      validate_slice(c, s)
      @base.require(b <= e && within(s, e), "Slice out of bounds")
    }
  }
}

///|
fn known(c : Code, l : String) -> Bool {
  c.blocks.any(b => b.label == l)
}

///|
fn validate_value(c : Code, v : RuntimeValue) -> Unit raise @base.Fault {
  match v {
    Num(_) => ()
    CodeLabel(l) => @base.require(known(c, l), "Unknown code label")
    SliceValue(s) => validate_slice(c, s)
    Address(s, i) => {
      validate_slice(c, s)
      @base.require(within(s, i), "Address out of bounds")
    }
  }
}

///|
fn validate_code(c : Code) -> Unit raise @base.Fault {
  @base.require(c.memories >= 1, "At least one memory is required")
  @data.unique(c.blocks.map(b => b.label), "Duplicate key: ")
  @base.require(known(c, "main"), "Missing entry block :main")
  fn validate_expr(v : ValueExpr) -> Unit raise @base.Fault {
    match v {
      Label(l) => validate_value(c, CodeLabel(l))
      Memory(m) => validate_slice(c, Whole(m))
      _ => ()
    }
  }
  fn place(p : Place) -> Unit raise @base.Fault {
    let Deref(v) = p
    validate_expr(v)
  }
  fn target(v : ValueExpr) -> Unit raise @base.Fault {
    match v {
      Reg(_) | Label(_) => validate_expr(v)
      _ => @base.error("Invalid jump target")
    }
  }
  for b in c.blocks {
    for s in b.stmts {
      match s {
        Print(v) => validate_expr(v)
        Input(p) => place(p)
        Store(p, v) => {
          place(p)
          validate_expr(v)
        }
        Assign(_, r) =>
          match r {
            Value(v) | Len(v) => validate_expr(v)
            Load(p) => place(p)
            Binop(a, _, b) | Addr(a, b) => {
              validate_expr(a)
              validate_expr(b)
            }
            Slice(s, b, e) => {
              validate_expr(s)
              validate_expr(b)
              validate_expr(e)
            }
          }
      }
    }
    for j in b.ifs {
      validate_expr(j.cond.0)
      validate_expr(j.cond.2)
      target(j.target)
    }
    match b.ending {
      Halt => ()
      Goto(v) => target(v)
    }
  }
}

///|
fn validate_memory(c : Code, mem : Memory) -> Unit raise @base.Fault {
  let keys = mem.map(p => p.0.to_string())
  @data.unique(keys, "Duplicate key: ")
  for p in mem {
    validate_value(c, p.1)
  }
}

///|
fn normalize(mem : Memory) -> Memory {
  let mem = mem.filter(p => {
    match p.1 {
      Num(n) => !n.is_zero()
      _ => true
    }
  })
  mem.sort_by((a, b) => a.0.compare(b.0))
  mem
}

///|
fn read_slice(p : @p.Parser) -> SliceValue raise @base.Fault {
  if p.eat_keyword("slice") {
    let s = read_slice(p)
    let b = p.natural()
    Subslice(s, b, p.natural())
  } else {
    match read_value(p) {
      Memory(m) => Whole(m)
      _ => p.error("Expected memory or slice")
    }
  }
}

///|
fn read_runtime_value(p : @p.Parser) -> RuntimeValue raise @base.Fault {
  if p.eat_keyword("addr") {
    let s = read_slice(p)
    Address(s, p.natural())
  } else if p.peek() == @p.Keyword("slice") {
    SliceValue(read_slice(p))
  } else {
    match read_value(p) {
      Imm(n) => Num(n)
      Label(l) => CodeLabel(l)
      Memory(m) => SliceValue(Whole(m))
      Reg(_) => p.error("Registers are not input values")
    }
  }
}

///|
pub fn parse_input(text : String) -> Memory raise @base.Fault {
  let p = @p.create(text)
  let cells = []
  while !p.eof() {
    let i = p.natural()
    p.expect_text(":=")
    let v = read_runtime_value(p)
    p.expect(';')
    cells.push((i, v))
  }
  cells
}

///|
fn slice_text(s : SliceValue) -> String {
  match s {
    Whole(m) => "MEM_" + m.to_string()
    Subslice(s, b, e) =>
      "\\slice " + slice_text(s) + " " + b.to_string() + " " + e.to_string()
  }
}

///|
fn value_text(v : RuntimeValue) -> String {
  match v {
    Num(n) => n.to_string()
    CodeLabel(l) => ":" + l
    SliceValue(s) => slice_text(s)
    Address(s, i) => "\\addr " + slice_text(s) + " " + i.to_string()
  }
}

///|
pub fn output(mem : Memory) -> String {
  @base.join(
    normalize(mem).map(p => p.0.to_string() + " := " + value_text(p.1) + ";\n"),
    "",
  )
}

///|
fn make(text : String, input : String) -> State raise @base.Fault {
  let code = parse(text)
  validate_code(code)
  let input = parse_input(input)
  validate_memory(code, input)
  {
    code,
    block: "main",
    line: 0,
    registers: [],
    memory: Array::makei(code.memories, i => {
      if i == 0 {
        normalize(input)
      } else {
        []
      }
    }),
  }
}

///|
fn eval(s : State, v : ValueExpr) -> RuntimeValue {
  match v {
    Reg(r) => @data.env_get(s.registers, r).unwrap_or(Num(@base.zero()))
    Imm(n) => Num(n)
    Label(l) => CodeLabel(l)
    Memory(m) => SliceValue(Whole(m))
  }
}

///|
fn num(v : RuntimeValue) -> @base.Nat raise @base.Fault {
  match v {
    Num(n) => n
    _ => @base.error("Expected natural number")
  }
}

///|
fn slice(v : RuntimeValue) -> SliceValue raise @base.Fault {
  match v {
    SliceValue(s) => s
    _ => @base.error("Expected slice")
  }
}

///|
fn address(v : RuntimeValue) -> (SliceValue, @base.Nat) raise @base.Fault {
  match v {
    Address(s, i) => (s, i)
    _ => @base.error("Expected address")
  }
}

///|
fn same_origin(s : SliceValue, t : SliceValue) -> Unit raise @base.Fault {
  @base.require(origin(s) == origin(t), "Different address origins")
}

///|
fn binop(
  o : Binop,
  a : RuntimeValue,
  b : RuntimeValue,
) -> RuntimeValue raise @base.Fault {
  match (o, a, b) {
    (Add, Num(x), Num(y)) => Num(x + y)
    (Sub, Num(x), Num(y)) => Num(@base.sub(x, y))
    (Add, Address(s, i), Num(n)) => {
      let j = i + n
      @base.require(within(s, j), "Address out of bounds")
      Address(s, j)
    }
    (Sub, Address(s, i), Num(n)) => {
      @base.require(n <= i, "Address out of bounds")
      Address(s, @base.sub(i, n))
    }
    (Sub, Address(s, i), Address(t, j)) => {
      same_origin(s, t)
      Num(@base.sub(base(s) + i, base(t) + j))
    }
    _ => @base.error("Invalid binary operand types")
  }
}

///|
fn place(s : State, p : Place) -> (Int32, @base.Nat) raise @base.Fault {
  let Deref(v) = p
  let (t, i) = address(eval(s, v))
  @base.require(readable(t, i), "Cannot access slice end")
  (origin(t), base(t) + i)
}

///|
fn load(s : State, m : Int32, i : @base.Nat) -> RuntimeValue raise @base.Fault {
  @base.require(m >= 0 && m < s.memory.length(), "Unknown memory")
  match s.memory[m].iter().find_first(p => p.0 == i) {
    Some(p) => p.1
    None => Num(@base.zero())
  }
}

///|
fn store(
  s : State,
  m : Int32,
  i : @base.Nat,
  v : RuntimeValue,
) -> State raise @base.Fault {
  @base.require(m >= 0 && m < s.memory.length(), "Unknown memory")
  let mem = s.memory[m].filter(p => p.0 != i)
  mem.push((i, v))
  let memory = s.memory.copy()
  memory[m] = normalize(mem)
  { ..s, memory, }
}

///|
fn rhs(s : State, r : Rhs) -> RuntimeValue raise @base.Fault {
  match r {
    Value(v) => eval(s, v)
    Binop(a, o, b) => binop(o, eval(s, a), eval(s, b))
    Load(p) => {
      let (m, i) = place(s, p)
      load(s, m, i)
    }
    Slice(v, b, e) => {
      let t = Subslice(slice(eval(s, v)), num(eval(s, b)), num(eval(s, e)))
      validate_slice(s.code, t)
      SliceValue(t)
    }
    Addr(v, i) => {
      let t = slice(eval(s, v))
      let i = num(eval(s, i))
      @base.require(within(t, i), "Address out of bounds")
      Address(t, i)
    }
    Len(v) =>
      match length(slice(eval(s, v))) {
        Some(n) => Num(n)
        None => @base.error("Infinite slice has no natural length")
      }
  }
}

///|
fn cond(s : State, c : Cond) -> Bool raise @base.Fault {
  let a = eval(s, c.0)
  let b = eval(s, c.2)
  let o = c.1
  let cmp = match (a, b) {
    (Num(x), Num(y)) => x.compare(y)
    (CodeLabel(x), CodeLabel(y)) if o == Eq => if x == y { 0 } else { 1 }
    (SliceValue(x), SliceValue(y)) if o == Eq => if x == y { 0 } else { 1 }
    (Address(t, i), Address(u, j)) =>
      if o == Eq && origin(t) != origin(u) {
        1
      } else {
        same_origin(t, u)
        (base(t) + i).compare(base(u) + j)
      }
    _ => @base.error("Invalid comparison operand types")
  }
  match o {
    Eq => cmp == 0
    Lt => cmp < 0
    Gt => cmp > 0
  }
}

///|
fn jump(s : State, v : ValueExpr) -> State raise @base.Fault {
  match eval(s, v) {
    CodeLabel(l) if known(s.code, l) => { ..s, block: l, line: 0, }
    _ => @base.error("Jump target is not a known code label")
  }
}

///|
fn step(
  s : State,
  input : String,
) -> @runtime.Transition[State] raise @base.Fault {
  let b = match s.code.blocks.iter().find_first(b => b.label == s.block) {
    Some(b) => b
    None => @base.error("Unknown current block")
  }
  let ns = b.stmts.length()
  let ni = b.ifs.length()
  @base.require(s.line >= 0, "Invalid instruction position")
  if s.line < ns {
    let (next, out) = match b.stmts[s.line] {
      Print(v) => (s, @data.to_bytes(num(eval(s, v))))
      Input(p) => {
        let (m, i) = place(s, p)
        (store(s, m, i, Num(@data.of_bytes(input))), "")
      }
      Assign(r, e) => {
        let v = rhs(s, e)
        ({ ..s, registers: @data.env_set(s.registers, r, v), }, "")
      }
      Store(p, v) => {
        let (m, i) = place(s, p)
        (store(s, m, i, eval(s, v)), "")
      }
    }
    @runtime.Continue({ ..next, line: s.line + 1, }, out)
  } else if s.line < ns + ni {
    let j = b.ifs[s.line - ns]
    @runtime.Continue(
      if cond(s, j.cond) {
        jump(s, j.target)
      } else {
        { ..s, line: s.line + 1, }
      },
      "",
    )
  } else if s.line == ns + ni {
    match b.ending {
      Halt => @runtime.Halt(output(s.memory[0]))
      Goto(v) => @runtime.Continue(jump(s, v), "")
    }
  } else {
    @base.error("Invalid instruction position")
  }
}

///|
fn encode(s : State) -> Json {
  @base.object([
    ("code", code_json(s.code)),
    ("block", @base.jstr(s.block)),
    ("line", @base.jint(s.line)),
    ("registers", @data.env_json(s.registers, runtimevalue_json)),
    ("memory", @base.jarray(s.memory.map(memory_json))),
  ])
}

///|
fn decode(j : Json) -> State raise @base.Fault {
  let code = decode_code(@base.field(j, "code"))
  validate_code(code)
  let block = @base.identifier(@base.string(@base.field(j, "block")))
  let b = match code.blocks.iter().find_first(b => b.label == block) {
    Some(b) => b
    None => @base.error("Unknown current block")
  }
  let line = @base.int(@base.field(j, "line"))
  @base.require(
    line >= 0 && line <= b.stmts.length() + b.ifs.length(),
    "Invalid instruction position",
  )
  let registers = @data.decode_env(
    @base.field(j, "registers"),
    decode_runtimevalue,
  )
  @data.unique(registers.map(p => p.0), "Duplicate key: ")
  for p in registers {
    validate_value(code, p.1)
  }
  let memory = @base.array(@base.field(j, "memory")).map(decode_memory)
  @base.require(
    memory.length() == code.memories,
    "Memory count does not match declaration",
  )
  for mem in memory {
    validate_memory(code, mem)
  }
  { code, block, line, registers, memory, }
}

///|
fn render(s : State) -> Json {
  let rows = []
  for m, cells in s.memory {
    for p in cells {
      rows.push(["MEM_" + m.to_string(), p.0.to_string(), value_text(p.1)])
    }
  }
  @base.jarray([
    @base.table("status", ["field", "value"], [
      ["block", ":" + s.block],
      ["line", s.line.to_string()],
    ]),
    @base.table(
      "registers",
      ["register", "value"],
      s.registers.map(p => ["%" + p.0, value_text(p.1)]),
    ),
    @base.table("memory (nonzero cells)", ["memory", "address", "value"], rows),
  ])
}

///|
pub fn model() -> @runtime.Model {
  @runtime.model(make, step, encode, decode, render)
}

///|
test "UTF-8 I/O and slice end cannot be dereferenced" {
  let m = model()
  m.make(
    "\\memories 1; :main { %p := \\addr MEM_0 0; \\input \\deref %p; %v := \\ld \\deref %p; \\print %v; \\halt; }",
    "",
  )
  ignore(m.step(""))
  ignore(m.step("日本語🌕"))
  ignore(m.step(""))
  assert_eq(
    @base.string(@base.field(@base.parse_json(m.step("")), "routput")),
    "日本語🌕",
  )
  m.make(
    "\\memories 1; :main { %s := \\slice MEM_0 0 1; %p := \\addr %s 1; %v := \\ld \\deref %p; \\halt; }",
    "",
  )
  ignore(m.step(""))
  ignore(m.step(""))
  let saved = m.snapshot()
  assert_true(
    try {
      ignore(m.step(""))
      false
    } catch {
      _ => true
    },
  )
  assert_eq(m.snapshot(), saved)
}
