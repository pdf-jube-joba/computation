///|
priv struct Compiled {
  blocks : Array[Block]
  entry_block : Int32
  block_index : Array[(String, Int32)]
  static_labels : Array[(String, Int32)]
}

///|
struct State {
  code : Code
  compiled : Compiled
  current_block : Int32
  vregs : Array[@utils.Nat]
  memory : Array[@utils.Nat]
  halted : Bool
}

///|
fn link(code : Code) -> Compiled raise @utils.Fault {
  let block_index = code.blocks.mapi((i, b) => (b.label, i))
  let static_labels = code.statics.mapi((i, s) => (s.label, i))
  @utils.unique(block_index.map(p => p.0), "Duplicate block label: @")
  @utils.unique(static_labels.map(p => p.0), "Duplicate static label: @")
  let entry_block = match @utils.env_get(block_index, "main") {
    Some(i) => i
    None => @utils.error("Missing entry block: @main")
  }
  { blocks: code.blocks, entry_block, block_index, static_labels, }
}

///|
fn make(code : Code, input : Memory) -> State raise @utils.Fault {
  let compiled = link(code)
  let memory = code.statics.map(s => s.value)
  memory.append(input.values)
  {
    code,
    compiled,
    current_block: compiled.entry_block,
    vregs: [],
    memory,
    halted: false,
  }
}

///|
fn eval_addr(s : State, a : Addr) -> @utils.Nat raise @utils.Fault {
  match a {
    VReg(i) => @machine.read(s.vregs, i)
    Imm(n) => n
    Label(n) =>
      match @utils.env_get(s.compiled.static_labels, n) {
        Some(i) => @utils.nint(i.reinterpret_as_uint())
        None =>
          match @utils.env_get(s.compiled.block_index, n) {
            Some(i) => @utils.nint(i.reinterpret_as_uint())
            None => @utils.error("Unknown label: @" + n)
          }
      }
  }
}

///|
fn eval_value(s : State, v : Value) -> @utils.Nat raise @utils.Fault {
  match v {
    VRegValue(i) => @machine.read(s.vregs, i)
    ImmValue(n) => n
    LabelValue(n) => eval_addr(s, Label(n))
    Deref(p) => @machine.read(s.memory, @utils.index(eval_addr(s, p)))
  }
}

///|
fn eval_cond(s : State, c : Cond) -> Bool raise @utils.Fault {
  match c {
    Lt(a, b) => {
      let a = eval_value(s, a)
      a < eval_value(s, b)
    }
    Eq(a, b) => {
      let a = eval_value(s, a)
      a == eval_value(s, b)
    }
    Gt(a, b) => {
      let a = eval_value(s, a)
      a > eval_value(s, b)
    }
  }
}

///|
fn step(
  s : State,
  _input : Rinput,
) -> @definition.Transition[State, Routput, Foutput] raise @utils.Fault {
  if s.halted {
    return @definition.Halt(Memory::{ values: s.memory, })
  }
  @utils.require(
    s.current_block >= 0 && s.current_block < s.compiled.blocks.length(),
    "Current block out of range: " + s.current_block.to_string(),
  )
  let block = s.compiled.blocks[s.current_block]
  let mut s = s
  for stmt in block.stmts {
    s = match stmt {
      Assign(dst, src) => {
        let n = eval_value(s, src)
        { ..s, vregs: @machine.write(s.vregs, dst, n), }
      }
      BinOp(dst, a, o, b) => {
        let a = eval_value(s, a)
        let b = eval_value(s, b)
        {
          ..s,
          vregs: @machine.write(
            s.vregs,
            dst,
            match o {
              Add => a + b
              Sub => @utils.sub(a, b)
            },
          ),
        }
      }
      Store(p, src) => {
        let addr = @utils.index(eval_addr(s, p))
        let n = eval_value(s, src)
        { ..s, memory: @machine.write(s.memory, addr, n), }
      }
    }
  }
  let target = match
    block.cont.ifs.iter().find_first(j => eval_cond(s, j.cond)) {
    Some(j) => j.target
    None => block.cont.jump
  }
  let current_block = @utils.index(eval_addr(s, target))
  @utils.require(
    current_block < s.compiled.blocks.length(),
    "Block address out of range: " + current_block.to_string(),
  )
  @definition.Continue({ ..s, current_block, }, ())
}

///|
fn encode(s : State) -> Json {
  @utils.object([
    ("code", code_json(s.code)),
    ("compiled", compiled_json(s.compiled)),
    ("current_block", @utils.jint(s.current_block)),
    ("vregs", @machine.nats_json(s.vregs)),
    ("memory", @machine.nats_json(s.memory)),
    ("halted", @utils.jbool(s.halted)),
  ])
}

///|
fn decode(j : Json) -> State raise @utils.Fault {
  {
    code: decode_code(@utils.field(j, "code")),
    compiled: decode_compiled(@utils.field(j, "compiled")),
    current_block: @utils.int(@utils.field(j, "current_block")),
    vregs: @machine.decode_nats(@utils.field(j, "vregs")),
    memory: @machine.decode_nats(@utils.field(j, "memory")),
    halted: @utils.boolean(@utils.field(j, "halted")),
  }
}

///|
fn render(s : State) -> Array[@definition.Block] {
  let b = if s.current_block >= 0 &&
    s.current_block < s.compiled.blocks.length() {
    Some(s.compiled.blocks[s.current_block])
  } else {
    None
  }
  [
    @definition.text_table("status", ["field", "value"], [
      ["block", s.current_block.to_string()],
      [
        "label",
        match b {
          Some(b) => b.label
          None => "<invalid>"
        },
      ],
      ["halted", s.halted.to_string()],
    ]),
    @definition.text_table(
      "block",
      ["index", "stmt"],
      (match b {
        Some(b) => b.stmts
        None => []
      }).mapi((i, s) => [i.to_string(), stmt_text(s)]),
    ),
    @definition.text_table(
      "vregs",
      ["vreg", "value"],
      s.vregs.mapi((i, n) => ["v" + i.to_string(), n.to_string()]),
    ),
    @definition.text_table(
      "memory",
      ["addr", "value"],
      s.memory.mapi((i, n) => [i.to_string(), n.to_string()]),
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
