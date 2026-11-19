///|
priv struct Compiled {
  blocks : Array[Block]
  entry_block : Int32
  block_index : Array[(String, Int32)]
  static_labels : Array[(String, Int32)]
}

///|
priv struct State {
  code : Code
  compiled : Compiled
  current_block : Int32
  vregs : Array[@base.Nat]
  memory : Array[@base.Nat]
  halted : Bool
}

///|
fn link(code : Code) -> Compiled raise @base.Fault {
  let block_index = code.blocks.mapi((i, b) => (b.label, i))
  let static_labels = code.statics.mapi((i, s) => (s.label, i))
  @data.unique(block_index.map(p => p.0), "Duplicate block label: @")
  @data.unique(static_labels.map(p => p.0), "Duplicate static label: @")
  let entry_block = match @data.env_get(block_index, "main") {
    Some(i) => i
    None => @base.error("Missing entry block: @main")
  }
  { blocks: code.blocks, entry_block, block_index, static_labels, }
}

///|
fn make(text : String, input : String) -> State raise @base.Fault {
  let code = parse(text)
  let compiled = link(code)
  let memory = code.statics.map(s => s.value)
  memory.append(@data.natural_list(input))
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
fn eval_addr(s : State, a : Addr) -> @base.Nat raise @base.Fault {
  match a {
    VReg(i) => @machine.read(s.vregs, i)
    Imm(n) => n
    Label(n) =>
      match @data.env_get(s.compiled.static_labels, n) {
        Some(i) => @base.nint(i)
        None =>
          match @data.env_get(s.compiled.block_index, n) {
            Some(i) => @base.nint(i)
            None => @base.error("Unknown label: @" + n)
          }
      }
  }
}

///|
fn eval_value(s : State, v : Value) -> @base.Nat raise @base.Fault {
  match v {
    VRegValue(i) => @machine.read(s.vregs, i)
    ImmValue(n) => n
    LabelValue(n) => eval_addr(s, Label(n))
    Deref(p) => @machine.read(s.memory, @base.index(eval_addr(s, p)))
  }
}

///|
fn eval_cond(s : State, c : Cond) -> Bool raise @base.Fault {
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
  input : String,
) -> @runtime.Transition[State] raise @base.Fault {
  @base.unit_input(input)
  if s.halted {
    return @runtime.Halt(@data.natural_list_text(s.memory))
  }
  @base.require(
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
              Sub => @base.sub(a, b)
            },
          ),
        }
      }
      Store(p, src) => {
        let addr = @base.index(eval_addr(s, p))
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
  let current_block = @base.index(eval_addr(s, target))
  @base.require(
    current_block < s.compiled.blocks.length(),
    "Block address out of range: " + current_block.to_string(),
  )
  @runtime.Continue({ ..s, current_block, }, "")
}

///|
fn encode(s : State) -> Json {
  @base.object([
    ("code", code_json(s.code)),
    ("compiled", compiled_json(s.compiled)),
    ("current_block", @base.jint(s.current_block)),
    ("vregs", @machine.nats_json(s.vregs)),
    ("memory", @machine.nats_json(s.memory)),
    ("halted", @base.jbool(s.halted)),
  ])
}

///|
fn decode(j : Json) -> State raise @base.Fault {
  {
    code: decode_code(@base.field(j, "code")),
    compiled: decode_compiled(@base.field(j, "compiled")),
    current_block: @base.int(@base.field(j, "current_block")),
    vregs: @machine.decode_nats(@base.field(j, "vregs")),
    memory: @machine.decode_nats(@base.field(j, "memory")),
    halted: @base.boolean(@base.field(j, "halted")),
  }
}

///|
fn render(s : State) -> Json {
  let b = if s.current_block >= 0 &&
    s.current_block < s.compiled.blocks.length() {
    Some(s.compiled.blocks[s.current_block])
  } else {
    None
  }
  @base.jarray([
    @base.table("status", ["field", "value"], [
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
    @base.table(
      "block",
      ["index", "stmt"],
      (match b {
        Some(b) => b.stmts
        None => []
      }).mapi((i, s) => [i.to_string(), stmt_text(s)]),
    ),
    @base.table(
      "vregs",
      ["vreg", "value"],
      s.vregs.mapi((i, n) => ["v" + i.to_string(), n.to_string()]),
    ),
    @base.table(
      "memory",
      ["addr", "value"],
      s.memory.mapi((i, n) => [i.to_string(), n.to_string()]),
    ),
  ])
}

///|
pub fn model() -> @runtime.Model {
  @runtime.model(make, step, encode, decode, render)
}
