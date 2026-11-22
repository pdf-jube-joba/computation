///|
struct State {
  regs : Array[@utils.Nat]
  flag : Bool
  code_len : Int32
  memory : Array[@utils.Nat]
  halted : Bool
}

///|
fn make(program : Code, image : @machine.Image) -> State {
  let code = program.words
  let memory = code.copy()
  memory.append(image.memory)
  {
    regs: image.register,
    flag: false,
    code_len: code.length(),
    memory,
    halted: false,
  }
}

///|
fn output(s : State) -> @machine.Image {
  { register: s.regs, memory: s.memory[s.code_len:].to_owned(), }
}

///|
fn step(
  s : State,
  _input : Rinput,
) -> @definition.Transition[State, Routput, Foutput] raise @utils.Fault {
  if s.halted {
    return @definition.Halt(output(s))
  }
  let pc = @utils.index(s.regs[0])
  let inst = @machine.decode(@machine.read(s.memory, pc))
  let (regs, flag, memory, halted) = @machine.execute(
    inst,
    n => n,
    s.regs,
    s.flag,
    s.memory,
  )
  let next = { ..s, regs, flag, memory, halted, }
  if halted {
    @definition.Halt(output(next))
  } else {
    @definition.Continue(next, ())
  }
}

///|
fn encode(s : State) -> Json {
  @utils.object([
    ("regs", @machine.nats_json(s.regs)),
    ("flag", @utils.jbool(s.flag)),
    ("code_len", @utils.jint(s.code_len)),
    ("memory", @machine.nats_json(s.memory)),
    ("halted", @utils.jbool(s.halted)),
  ])
}

///|
fn decode(j : Json) -> State raise @utils.Fault {
  let regs = @machine.decode_nats(@utils.field(j, "regs"))
  @machine.validate_regs(regs)
  let memory = @machine.decode_nats(@utils.field(j, "memory"))
  let code_len = @utils.int(@utils.field(j, "code_len"))
  @utils.require(
    code_len >= 0 && code_len <= memory.length(),
    "Invalid code length",
  )
  {
    regs,
    memory,
    code_len,
    flag: @utils.boolean(@utils.field(j, "flag")),
    halted: @utils.boolean(@utils.field(j, "halted")),
  }
}

///|
fn render(s : State) -> Array[@definition.Block] {
  let pc = Some(@utils.index(s.regs[0])) catch { _ => None }
  let next = match pc {
    Some(i) if i < s.memory.length() =>
      @machine.text(@machine.decode(s.memory[i]), n => n.to_string()) catch {
        _ => "halt"
      }
    _ => "halt"
  }
  [
    @definition.text_table("status", ["field", "value"], [
      [
        "pc",
        match pc {
          Some(n) => n.to_string()
          None => "?"
        },
      ],
      ["flag", s.flag.to_string()],
      ["next", next],
      ["halted", s.halted.to_string()],
    ]),
    @machine.registers(s.regs),
    @definition.text_table(
      "memory",
      ["addr", "value"],
      s.memory.mapi((i, n) => [i.to_string(), n.to_string()]),
      highlight=pc.unwrap_or(-1),
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
