///|
priv struct State {
  code : Array[Item]
  insts : Array[@machine.Instr[RawValue]]
  symbols : Symbols
  debug_info : DebugInfo
  regs : Array[@base.Nat]
  flag : Bool
  pc : Int32
  static_data_len : Int32
  static_data : Array[@base.Nat]
  data_memory : Array[@base.Nat]
  halted : Bool
}

///|
fn make(text : String, input : String) -> State raise @base.Fault {
  let code = parse(text)
  let image = @machine.image(input)
  let symbols = symbols(code)
  let insts = []
  let static_data = []
  let mut data = false
  for item in code {
    match item {
      Directive(Text) => data = false
      Directive(Data) => data = true
      Inst(i) => {
        @base.require(!data, "Instruction found outside \\text section")
        insts.push(lower(i))
      }
      DataValue(v) => {
        @base.require(data, "Data value found outside \\data section")
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
    pc: @base.index(image.register[0]),
    static_data_len: static_data.length(),
    static_data,
    data_memory,
    halted: false,
  }
}

///|
fn output(s : State) -> String {
  @machine.image_text(s.regs, s.data_memory)
}

///|
fn step(
  s : State,
  input : String,
) -> @runtime.Transition[State] raise @base.Fault {
  @base.unit_input(input)
  if s.halted {
    return @runtime.Halt(output(s))
  }
  if s.pc >= s.insts.length() {
    let regs = s.regs.copy()
    regs[0] = @base.nint(s.pc)
    return @runtime.Halt(output({ ..s, regs, halted: true, }))
  }
  @base.require(s.pc >= 0, "Negative program counter")
  let (regs, flag, data_memory, halted) = @machine.execute(
    s.insts[s.pc],
    v => resolve(s.symbols, false, s.pc, v),
    s.regs,
    s.flag,
    s.data_memory,
  )
  let pc = @base.index(regs[0])
  let next = { ..s, regs, flag, data_memory, halted, pc, }
  if halted {
    @runtime.Halt(output(next))
  } else {
    @runtime.Continue(next, "")
  }
}

///|
fn encode(s : State) -> Json {
  @base.object([
    ("code", @base.jarray(s.code.map(item_json))),
    (
      "insts",
      @base.jarray(s.insts.map(i => @machine.instr_json(i, rawvalue_json))),
    ),
    ("symbols", symbols_json(s.symbols)),
    ("debug_info", debug_json(s.debug_info)),
    ("regs", @machine.nats_json(s.regs)),
    ("flag", @base.jbool(s.flag)),
    ("pc", @base.jint(s.pc)),
    ("static_data_len", @base.jint(s.static_data_len)),
    ("static_data", @machine.nats_json(s.static_data)),
    ("data_memory", @machine.nats_json(s.data_memory)),
    ("halted", @base.jbool(s.halted)),
  ])
}

///|
fn decode(j : Json) -> State raise @base.Fault {
  let regs = @machine.decode_nats(@base.field(j, "regs"))
  @machine.validate_regs(regs)
  let pc = @base.int(@base.field(j, "pc"))
  let static_data_len = @base.int(@base.field(j, "static_data_len"))
  let data_memory = @machine.decode_nats(@base.field(j, "data_memory"))
  @base.require(
    pc >= 0 && static_data_len >= 0 && static_data_len <= data_memory.length(),
    "Invalid assembly snapshot",
  )
  {
    regs,
    pc,
    static_data_len,
    data_memory,
    code: @base.array(@base.field(j, "code")).map(decode_item),
    insts: @base.array(@base.field(j, "insts")).map(j => {
      @machine.decode_instr(j, decode_rawvalue)
    }),
    symbols: decode_symbols(@base.field(j, "symbols")),
    debug_info: decode_debug(@base.field(j, "debug_info")),
    flag: @base.boolean(@base.field(j, "flag")),
    static_data: @machine.decode_nats(@base.field(j, "static_data")),
    halted: @base.boolean(@base.field(j, "halted")),
  }
}

///|
fn render(s : State) -> Json {
  let next = if s.pc >= 0 && s.pc < s.insts.length() {
    @machine.text(s.insts[s.pc], raw_text)
  } else {
    "halt"
  }
  @base.jarray([
    @base.table("status", ["field", "value"], [
      ["pc", s.pc.to_string()],
      ["flag", s.flag.to_string()],
      ["next", next],
      ["halted", s.halted.to_string()],
    ]),
    @machine.registers(s.regs),
    @base.table(
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
            @base.join(names, ", ")
          },
          v.to_string(),
        ]
      }),
    ),
  ])
}

///|
pub fn model() -> @runtime.Model {
  @runtime.model(make, step, encode, decode, render)
}

///|
test "symbol resolution, data stores and skipped conditional immediates" {
  let m = model()
  m.make(
    "\\equ N #4\n\\text\n\\ldi %r1 N;\n\\ldi %r2 @v;\n\\st %r1 %r2;\n\\ldi \\f %r3 UNKNOWN;\n\\halt;\n\\data\n@v:\n0",
    "0,0,0,0,0,0,0,0",
  )
  for _ in 0..<4 {
    ignore(m.step(""))
  }
  assert_eq(
    @base.string(@base.field(@base.parse_json(m.step("")), "foutput")),
    "4, 4, 0, 0, 0, 0, 0, 0\n4",
  )
}
