///|
pub(all) enum Command {
  Clr(String)
  Inc(String)
  Dec(String)
  Cpy(String, String)
  Ifnz(String, @utils.Nat)
}

///|
struct State {
  commands : Array[Command]
  pc : @utils.Nat
  names : Array[String]
  env : Array[(String, @utils.Nat)]
}

///|
fn parse_code(code : String) -> Array[Command] raise @utils.Fault {
  let p = @utils.create(code)
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
fn parse_env(input : String) -> Array[(String, @utils.Nat)] raise @utils.Fault {
  let p = @utils.create(input)
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
fn print_env(env : Array[(String, @utils.Nat)]) -> String {
  @utils.join(env.map(pair => pair.0 + " = " + pair.1.to_string()), "\n")
}

///|
fn make(code : Code, input : Environment) -> State {
  let commands = code.commands
  let env = input.values
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
  { commands, env, names, pc: @utils.zero(), }
}

///|
fn lookup(env : Array[(String, @utils.Nat)], name : String) -> @utils.Nat {
  for pair in env {
    if pair.0 == name {
      return pair.1
    }
  }
  @utils.zero()
}

///|
fn set(
  env : Array[(String, @utils.Nat)],
  name : String,
  value : @utils.Nat,
) -> Array[(String, @utils.Nat)] {
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
  _input : Rinput,
) -> @definition.Transition[State, Routput, Foutput] raise @utils.Fault {
  let pc = @utils.index(s.pc)
  if pc >= s.commands.length() {
    return @definition.Halt(Environment::{ values: s.env, })
  }
  let next_pc = s.pc + @utils.one()
  let (env, pc) = match s.commands[pc] {
    Clr(name) => (set(s.env, name, @utils.zero()), next_pc)
    Inc(name) => (set(s.env, name, lookup(s.env, name) + @utils.one()), next_pc)
    Dec(name) => (set(s.env, name, @utils.pred(lookup(s.env, name))), next_pc)
    Cpy(a, b) => (set(s.env, a, lookup(s.env, b)), next_pc)
    Ifnz(name, target) =>
      (s.env, if lookup(s.env, name).is_zero() { next_pc } else { target })
  }
  @definition.Continue({ ..s, env, pc, }, ())
}

///|
fn command_json(command : Command) -> Json {
  match command {
    Clr(name) => @utils.tag("Clr", [@utils.jstr(name)])
    Inc(name) => @utils.tag("Inc", [@utils.jstr(name)])
    Dec(name) => @utils.tag("Dec", [@utils.jstr(name)])
    Cpy(a, b) => @utils.tag("Cpy", [@utils.jstr(a), @utils.jstr(b)])
    Ifnz(name, n) =>
      @utils.tag("Ifnz", [@utils.jstr(name), @utils.jstr(n.to_string())])
  }
}

///|
fn decode_command(value : Json) -> Command raise @utils.Fault {
  let name = @utils.identifier(@utils.string(@utils.tag_arg(value, 0)))
  match @utils.tag_name(value) {
    "Clr" => Clr(name)
    "Inc" => Inc(name)
    "Dec" => Dec(name)
    "Cpy" =>
      Cpy(name, @utils.identifier(@utils.string(@utils.tag_arg(value, 1))))
    "Ifnz" => Ifnz(name, @utils.json_nat(@utils.tag_arg(value, 1)))
    _ => @utils.error("Invalid goto command")
  }
}

///|
fn encode(s : State) -> Json {
  fn id(name) {
    s.names.search(name).unwrap()
  }
  @utils.object([
    ("commands", @utils.jarray(s.commands.map(command_json))),
    ("pc", @utils.jstr(s.pc.to_string())),
    (
      "variable_names",
      @utils.jarray(
        s.names.mapi((i, name) => {
          @utils.jarray([@utils.jstr(name), @utils.jint(i)])
        }),
      ),
    ),
    (
      "env",
      @utils.jarray(
        s.env.map(pair => {
          @utils.jarray([
            @utils.jint(id(pair.0)),
            @utils.jstr(pair.1.to_string()),
          ])
        }),
      ),
    ),
  ])
}

///|
fn decode(value : Json) -> State raise @utils.Fault {
  let names = @utils.array(@utils.field(value, "variable_names")).mapi((i, pair) => {
    let pair = @utils.array(pair)
    @utils.require(
      pair.length() == 2 && @utils.int(pair[1]) == i,
      "Invalid variable id",
    )
    @utils.identifier(@utils.string(pair[0]))
  })
  let seen : Map[String, Bool] = Map([])
  for name in names {
    @utils.require(!seen.contains(name), "Duplicate variable name")
    seen[name] = true
  }
  let commands = @utils.array(@utils.field(value, "commands")).map(
    decode_command,
  )
  for command in commands {
    let used = match command {
      Clr(n) | Inc(n) | Dec(n) | Ifnz(n, _) => [n]
      Cpy(a, b) => [a, b]
    }
    @utils.require(
      used.all(n => names.contains(n)),
      "Unknown variable in snapshot",
    )
  }
  let env = @utils.array(@utils.field(value, "env")).map(pair => {
    let pair = @utils.array(pair)
    @utils.require(pair.length() == 2, "Invalid environment entry")
    let id = @utils.int(pair[0])
    @utils.require(id >= 0 && id < names.length(), "Invalid variable id")
    (names[id], @utils.json_nat(pair[1]))
  })
  { names, commands, env, pc: @utils.json_nat(@utils.field(value, "pc")), }
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
fn render(s : State) -> Array[@definition.Block] {
  let next = if s.pc >= @utils.nint(s.commands.length().reinterpret_as_uint()) {
    "halt"
  } else {
    print_command(s.commands[s.pc.to_int()])
  }
  [
    @definition.text_table("status", ["field", "value"], [
      ["pc", s.pc.to_string()],
      ["next", next],
    ]),
    @definition.text_table(
      "env",
      ["var", "value"],
      s.env.map(pair => [pair.0, pair.1.to_string()]),
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
