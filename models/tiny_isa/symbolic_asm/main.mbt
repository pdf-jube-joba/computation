///|
struct State {
  code : Array[Item]
  insts : Array[@machine.Instr[RawValue]]
  symbols : Symbols
  debug_info : DebugInfo
  regs : Array[@utils.Nat]
  flag : Bool
  pc : Int32
  static_data_len : Int32
  static_data : Array[@utils.Nat]
  data_memory : Array[@utils.Nat]
  halted : Bool
}

///|
fn make(program : Code, image : @machine.Image) -> State raise @utils.Fault {
  let code = program.items
  let symbols = symbols(code)
  let insts = []
  let static_data = []
  let mut data = false
  for item in code {
    match item {
      Directive(Text) => data = false
      Directive(Data) => data = true
      Inst(i) => {
        @utils.require(!data, "Instruction found outside \\text section")
        insts.push(lower(i))
      }
      DataValue(v) => {
        @utils.require(data, "Data value found outside \\data section")
        static_data.push(resolve(symbols, true, static_data.length(), v))
      }
      _ => ()
    }
  }
  let data_memory = static_data.copy()
  data_memory.append(image.memory)
  {
    code,
    insts,
    symbols,
    debug_info: {
      text_labels_by_addr: debug_labels(symbols.text_labels),
      data_labels_by_addr: debug_labels(symbols.data_labels),
    },
    regs: image.register,
    flag: false,
    pc: @utils.index(image.register[0]),
    static_data_len: static_data.length(),
    static_data,
    data_memory,
    halted: false,
  }
}

///|
fn output(s : State) -> @machine.Image {
  { register: s.regs, memory: s.data_memory, }
}

///|
fn step(
  s : State,
  _input : Rinput,
) -> @definition.Transition[State, Routput, Foutput] raise @utils.Fault {
  if s.halted {
    return @definition.Halt(output(s))
  }
  if s.pc >= s.insts.length() {
    let regs = s.regs.copy()
    regs[0] = @utils.nint(s.pc.reinterpret_as_uint())
    return @definition.Halt(output({ ..s, regs, halted: true, }))
  }
  @utils.require(s.pc >= 0, "Negative program counter")
  let (regs, flag, data_memory, halted) = @machine.execute(
    s.insts[s.pc],
    v => resolve(s.symbols, false, s.pc, v),
    s.regs,
    s.flag,
    s.data_memory,
  )
  let pc = @utils.index(regs[0])
  let next = { ..s, regs, flag, data_memory, halted, pc, }
  if halted {
    @definition.Halt(output(next))
  } else {
    @definition.Continue(next, ())
  }
}

///|
fn encode(s : State) -> Json {
  @utils.object([
    ("code", @utils.jarray(s.code.map(item_json))),
    (
      "insts",
      @utils.jarray(s.insts.map(i => @machine.instr_json(i, rawvalue_json))),
    ),
    ("symbols", symbols_json(s.symbols)),
    ("debug_info", debug_json(s.debug_info)),
    ("regs", @machine.nats_json(s.regs)),
    ("flag", @utils.jbool(s.flag)),
    ("pc", @utils.jint(s.pc)),
    ("static_data_len", @utils.jint(s.static_data_len)),
    ("static_data", @machine.nats_json(s.static_data)),
    ("data_memory", @machine.nats_json(s.data_memory)),
    ("halted", @utils.jbool(s.halted)),
  ])
}

///|
fn decode(j : Json) -> State raise @utils.Fault {
  let regs = @machine.decode_nats(@utils.field(j, "regs"))
  @machine.validate_regs(regs)
  let pc = @utils.int(@utils.field(j, "pc"))
  let static_data_len = @utils.int(@utils.field(j, "static_data_len"))
  let data_memory = @machine.decode_nats(@utils.field(j, "data_memory"))
  @utils.require(
    pc >= 0 && static_data_len >= 0 && static_data_len <= data_memory.length(),
    "Invalid assembly snapshot",
  )
  {
    regs,
    pc,
    static_data_len,
    data_memory,
    code: @utils.array(@utils.field(j, "code")).map(decode_item),
    insts: @utils.array(@utils.field(j, "insts")).map(j => {
      @machine.decode_instr(j, decode_rawvalue)
    }),
    symbols: decode_symbols(@utils.field(j, "symbols")),
    debug_info: decode_debug(@utils.field(j, "debug_info")),
    flag: @utils.boolean(@utils.field(j, "flag")),
    static_data: @machine.decode_nats(@utils.field(j, "static_data")),
    halted: @utils.boolean(@utils.field(j, "halted")),
  }
}

///|
fn render(s : State) -> Array[@definition.Block] {
  let next = if s.pc >= 0 && s.pc < s.insts.length() {
    @machine.text(s.insts[s.pc], raw_text)
  } else {
    "halt"
  }
  [
    @definition.text_table("status", ["field", "value"], [
      ["pc", s.pc.to_string()],
      ["flag", s.flag.to_string()],
      ["next", next],
      ["halted", s.halted.to_string()],
    ]),
    @machine.registers(s.regs),
    @definition.text_table(
      "data",
      ["addr", "label", "value"],
      s.data_memory.mapi((addr, v) => {
        let names = match
          s.debug_info.data_labels_by_addr.iter().find_first(p => p.0 == addr) {
          Some(p) => p.1
          None => []
        }
        [
          addr.to_string(),
          if names.is_empty() {
            "-"
          } else {
            @utils.join(names, ", ")
          },
          v.to_string(),
        ]
      }),
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
