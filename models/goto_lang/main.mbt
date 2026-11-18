///|
priv enum Command {
  Clr(String)
  Inc(String)
  Dec(String)
  Cpy(String, String)
  Ifnz(String, @base.Nat)
}

///|
priv struct State {
  commands : Array[Command]
  pc : @base.Nat
  names : Array[String]
  env : Array[(String, @base.Nat)]
}

///|
fn parse_code(code : String) -> Array[Command] raise @base.Fault {
  let p = @p.create(code)
  let commands = []
  while !p.eof() {
    let command = match p.keyword() {
      "clr" => Clr(p.ident())
      "inc" => Inc(p.ident())
      "dec" => Dec(p.ident())
      "cpy" => {
        let dest = p.ident()
        p.expect_text("<-")
        Cpy(dest, p.ident())
      }
      "ifnz" => {
        let name = p.ident()
        p.expect(':')
        Ifnz(name, p.natural())
      }
      _ => p.error("Expected goto command")
    }
    p.expect(';')
    commands.push(command)
  }
  commands
}

///|
fn parse_env(input : String) -> Array[(String, @base.Nat)] raise @base.Fault {
  let p = @p.create(input)
  let env = []
  while !p.eof() {
    let name = p.ident()
    p.expect('=')
    env.push((name, p.natural()))
    ignore(p.eat(';'))
  }
  env
}

///|
fn print_env(env : Array[(String, @base.Nat)]) -> String {
  @base.join(env.map(pair => pair.0 + " = " + pair.1.to_string()), "\n")
}

///|
fn make(code : String, input : String) -> State raise @base.Fault {
  let commands = parse_code(code)
  let env = parse_env(input)
  let names : Array[String] = []
  fn variable(name) {
    if !names.contains(name) {
      names.push(name)
    }
  }
  for command in commands {
    match command {
      Clr(name) | Inc(name) | Dec(name) | Ifnz(name, _) => variable(name)
      Cpy(a, b) => {
        variable(a)
        variable(b)
      }
    }
  }
  for pair in env {
    variable(pair.0)
  }
  { commands, env, names, pc: @base.zero(), }
}

///|
fn lookup(env : Array[(String, @base.Nat)], name : String) -> @base.Nat {
  for pair in env {
    if pair.0 == name {
      return pair.1
    }
  }
  @base.zero()
}

///|
fn set(
  env : Array[(String, @base.Nat)],
  name : String,
  value : @base.Nat,
) -> Array[(String, @base.Nat)] {
  if env.any(pair => pair.0 == name) {
    env.map(pair => if pair.0 == name { (name, value) } else { pair })
  } else {
    let env = env.copy()
    env.push((name, value))
    env
  }
}

///|
fn step(
  s : State,
  input : String,
) -> @runtime.Transition[State] raise @base.Fault {
  @base.unit_input(input)
  let pc = @base.index(s.pc)
  if pc >= s.commands.length() {
    return @runtime.Halt(print_env(s.env))
  }
  let next_pc = s.pc + @base.one()
  let (env, pc) = match s.commands[pc] {
    Clr(name) => (set(s.env, name, @base.zero()), next_pc)
    Inc(name) => (set(s.env, name, lookup(s.env, name) + @base.one()), next_pc)
    Dec(name) => (set(s.env, name, @base.pred(lookup(s.env, name))), next_pc)
    Cpy(a, b) => (set(s.env, a, lookup(s.env, b)), next_pc)
    Ifnz(name, target) =>
      (s.env, if lookup(s.env, name).is_zero() { next_pc } else { target })
  }
  @runtime.Continue({ ..s, env, pc, }, "")
}

///|
fn command_json(command : Command) -> Json {
  match command {
    Clr(name) => @base.tag("Clr", [@base.jstr(name)])
    Inc(name) => @base.tag("Inc", [@base.jstr(name)])
    Dec(name) => @base.tag("Dec", [@base.jstr(name)])
    Cpy(a, b) => @base.tag("Cpy", [@base.jstr(a), @base.jstr(b)])
    Ifnz(name, n) =>
      @base.tag("Ifnz", [@base.jstr(name), @base.jstr(n.to_string())])
  }
}

///|
fn decode_command(value : Json) -> Command raise @base.Fault {
  let name = @base.identifier(@base.string(@base.tag_arg(value, 0)))
  match @base.tag_name(value) {
    "Clr" => Clr(name)
    "Inc" => Inc(name)
    "Dec" => Dec(name)
    "Cpy" => Cpy(name, @base.identifier(@base.string(@base.tag_arg(value, 1))))
    "Ifnz" => Ifnz(name, @base.json_nat(@base.tag_arg(value, 1)))
    _ => @base.error("Invalid goto command")
  }
}

///|
fn encode(s : State) -> Json {
  fn id(name) {
    s.names.search(name).unwrap()
  }
  @base.object([
    ("commands", @base.jarray(s.commands.map(command_json))),
    ("pc", @base.jstr(s.pc.to_string())),
    (
      "variable_names",
      @base.jarray(
        s.names.mapi((i, name) => {
          @base.jarray([@base.jstr(name), @base.jint(i)])
        }),
      ),
    ),
    (
      "env",
      @base.jarray(
        s.env.map(pair => {
          @base.jarray([@base.jint(id(pair.0)), @base.jstr(pair.1.to_string())])
        }),
      ),
    ),
  ])
}

///|
fn decode(value : Json) -> State raise @base.Fault {
  let names = @base.array(@base.field(value, "variable_names")).mapi((i, pair) => {
    let pair = @base.array(pair)
    @base.require(
      pair.length() == 2 && @base.int(pair[1]) == i,
      "Invalid variable id",
    )
    @base.identifier(@base.string(pair[0]))
  })
  let seen : Map[String, Bool] = Map([])
  for name in names {
    @base.require(!seen.contains(name), "Duplicate variable name")
    seen[name] = true
  }
  let commands = @base.array(@base.field(value, "commands")).map(decode_command)
  for command in commands {
    let used = match command {
      Clr(n) | Inc(n) | Dec(n) | Ifnz(n, _) => [n]
      Cpy(a, b) => [a, b]
    }
    @base.require(
      used.all(n => names.contains(n)),
      "Unknown variable in snapshot",
    )
  }
  let env = @base.array(@base.field(value, "env")).map(pair => {
    let pair = @base.array(pair)
    @base.require(pair.length() == 2, "Invalid environment entry")
    let id = @base.int(pair[0])
    @base.require(id >= 0 && id < names.length(), "Invalid variable id")
    (names[id], @base.json_nat(pair[1]))
  })
  { names, commands, env, pc: @base.json_nat(@base.field(value, "pc")), }
}

///|
fn print_command(command : Command) -> String {
  match command {
    Clr(name) => "\\clr " + name + ";"
    Inc(name) => "\\inc " + name + ";"
    Dec(name) => "\\dec " + name + ";"
    Cpy(a, b) => "\\cpy " + a + " <- " + b + ";"
    Ifnz(name, n) => "\\ifnz " + name + " : " + n.to_string() + ";"
  }
}

///|
fn render(s : State) -> Json {
  let next = if s.pc >= @base.nint(s.commands.length()) {
    "halt"
  } else {
    print_command(s.commands[s.pc.to_int()])
  }
  @base.jarray([
    @base.table("status", ["field", "value"], [
      ["pc", s.pc.to_string()],
      ["next", next],
    ]),
    @base.table(
      "env",
      ["var", "value"],
      s.env.map(pair => [pair.0, pair.1.to_string()]),
    ),
  ])
}

///|
pub fn model() -> @runtime.Model {
  @runtime.model(make, step, encode, decode, render)
}

///|
test "goto arbitrary precision and stopping step" {
  let m = model()
  m.make("\\inc x;", "x = 999999999999999999999999999999")
  ignore(m.step(""))
  let snapshot = m.snapshot()
  let result = @base.parse_json(m.step(""))
  assert_eq(
    @base.string(@base.field(result, "foutput")),
    "x = 1000000000000000000000000000000",
  )
  m.restore(snapshot)
  assert_eq(@base.parse_json(m.step("")), result)
}

///|
test "goto conditional jump uses zero based line and errors preserve state" {
  let m = model()
  m.make("\\ifnz x : 0;", "x = 1")
  for _ in 0..<5 {
    ignore(m.step(""))
  }
  let saved = m.snapshot()
  assert_true(
    try {
      ignore(m.step("invalid"))
      false
    } catch {
      _ => true
    },
  )
  assert_eq(m.snapshot(), saved)
}
