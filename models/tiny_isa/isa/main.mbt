///|
priv struct State {
  regs : Array[@base.Nat]
  flag : Bool
  code_len : Int32
  memory : Array[@base.Nat]
  halted : Bool
}

///|
fn make(code : String, input : String) -> State raise @base.Fault {
  let code = @machine.natural_lines(code)
  let image = @machine.image(input)
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
fn output(s : State) -> String {
  @machine.image_text(s.regs, s.memory[s.code_len:].to_owned())
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
  let pc = @base.index(s.regs[0])
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
    @runtime.Halt(output(next))
  } else {
    @runtime.Continue(next, "")
  }
}

///|
fn encode(s : State) -> Json {
  @base.object([
    ("regs", @machine.nats_json(s.regs)),
    ("flag", @base.jbool(s.flag)),
    ("code_len", @base.jint(s.code_len)),
    ("memory", @machine.nats_json(s.memory)),
    ("halted", @base.jbool(s.halted)),
  ])
}

///|
fn decode(j : Json) -> State raise @base.Fault {
  let regs = @machine.decode_nats(@base.field(j, "regs"))
  @machine.validate_regs(regs)
  let memory = @machine.decode_nats(@base.field(j, "memory"))
  let code_len = @base.int(@base.field(j, "code_len"))
  @base.require(
    code_len >= 0 && code_len <= memory.length(),
    "Invalid code length",
  )
  {
    regs,
    memory,
    code_len,
    flag: @base.boolean(@base.field(j, "flag")),
    halted: @base.boolean(@base.field(j, "halted")),
  }
}

///|
fn render(s : State) -> Json {
  let pc = Some(@base.index(s.regs[0])) catch { _ => None }
  let next = match pc {
    Some(i) if i < s.memory.length() =>
      @machine.text(@machine.decode(s.memory[i]), n => n.to_string()) catch {
        _ => "halt"
      }
    _ => "halt"
  }
  @base.jarray([
    @base.table("status", ["field", "value"], [
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
    @base.table(
      "memory",
      ["addr", "value"],
      s.memory.mapi((i, n) => [i.to_string(), n.to_string()]),
      highlight=pc.unwrap_or(-1),
    ),
  ])
}

///|
pub fn model() -> @runtime.Model {
  @runtime.model(make, step, encode, decode, render)
}

///|
test "instruction encoding and conditional updates preserve registers" {
  for
    inst in [
      @machine.Nop,
      @machine.Halt,
      @machine.Rst,
      @machine.Ldi(false, 3, @base.nat("999999999999999999999")),
      @machine.Mov(true, 0, 2),
    ] {
    let word = @machine.encode(inst)
    assert_eq(@machine.encode(@machine.decode(word)), word)
  }
}
