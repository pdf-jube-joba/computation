///|
#derive(Eq)
pub(all) enum Stmt {
  Lt
  Rt
  Assign(@s.Lvalue, @s.Rvalue)
  Break(@s.Condition?)
  Continue(@s.Condition?)
  Jump(String, @s.Condition?)
  Return(@s.Condition?)
  Call(String)
}

///|
#derive(Eq)
pub(all) struct Block {
  label : String
  body : Array[Stmt]
}

///|
#derive(Eq)
pub(all) struct FunctionDef {
  name : String
  blocks : Array[Block]
}

///|
#derive(Eq)
pub(all) struct Code {
  alphabet : Array[String]
  functions : Array[FunctionDef]
}

///|
priv struct Frame {
  function_ : String
  env : @s.Environment
  blocks : Array[Block]
  pc : (Int32, Int32)
}

///|
struct State {
  program : Code
  frame : Frame
  stack : Array[Frame]
  tape : @utils.Tape[String]
}

///|
pub fn stmt_vars(stmt : Stmt) -> Array[String] {
  match stmt {
    Assign(dst, src) => {
      let names = @s.lvars(dst)
      names.append(@s.rvars(src))
      names
    }
    Break(c) | Continue(c) | Jump(_, c) | Return(c) => @s.cvars(c)
    _ => []
  }
}

///|
pub fn collect_vars(blocks : Array[Block]) -> Array[String] {
  let names = []
  for b in blocks {
    for stmt in b.body {
      names.append(stmt_vars(stmt))
    }
  }
  @s.env_create(names).names.map(pair => pair.0)
}

///|
pub fn find_function(
  code : Code,
  name : String,
) -> FunctionDef raise @utils.Fault {
  match code.functions.iter().find_first(f => f.name == name) {
    Some(f) => f
    None => @utils.error("Undefined function '" + name + "'")
  }
}

///|
pub fn validate(code : Code) -> Unit raise @utils.Fault {
  ignore(find_function(code, "main"))
  let visited : Map[String, Int32] = Map([])
  fn visit(name : String, path : Array[String]) -> Unit raise @utils.Fault {
    match visited.get(name) {
      Some(2) => ()
      Some(_) => {
        let path = path.copy()
        path.push(name)
        @utils.error(
          "Recursive call is not allowed: " + @utils.join(path, " -> "),
        )
      }
      None => {
        visited[name] = 1
        let f = find_function(code, name)
        let path = path.copy()
        path.push(name)
        for b in f.blocks {
          for stmt in b.body {
            match stmt {
              Call(n) => visit(n, path)
              _ => ()
            }
          }
        }
        visited[name] = 2
      }
    }
  }
  for f in code.functions {
    visit(f.name, [])
  }
  let allowed = @s.normalized_alphabet(code.alphabet)
  let seen : Map[String, Bool] = Map([])
  for f in code.functions {
    @utils.require(
      !seen.contains(f.name),
      "Function '" + f.name + "' is defined twice",
    )
    seen[f.name] = true
    let labels : Map[String, Bool] = Map([])
    for b in f.blocks {
      @utils.require(
        !labels.contains(b.label),
        "label '" + b.label + "' is duplicated",
      )
      labels[b.label] = true
    }
    for b in f.blocks {
      for stmt in b.body {
        match stmt {
          Assign(_, src) => @s.validate_value(allowed, src)
          Break(c) | Continue(c) | Return(c) =>
            @s.validate_condition(allowed, c)
          Jump(label, c) => {
            @s.validate_condition(allowed, c)
            @utils.require(
              labels.contains(label),
              "jump target '" + label + "' not found",
            )
          }
          _ => ()
        }
      }
    }
  }
}

///|
fn read_stmt(p : @utils.Parser) -> Stmt raise @utils.Fault {
  if p.eat_keyword("LT") {
    Lt
  } else if p.eat_keyword("RT") {
    Rt
  } else if p.eat_keyword("break") {
    Break(@s.condition(p))
  } else if p.eat_keyword("continue") {
    Continue(@s.condition(p))
  } else if p.eat_keyword("return") {
    Return(@s.condition(p))
  } else if p.eat_keyword("jump") {
    let label = p.ident()
    Jump(label, @s.condition(p))
  } else if p.eat_keyword("call") {
    let name = p.ident()
    if p.peek() == @utils.Symbol('(') {
      p.group('(').finish()
    }
    Call(name)
  } else {
    let dst = @s.lvalue(p)
    p.expect_text(":=")
    Assign(dst, @s.rvalue(p))
  }
}

///|
pub fn parse_code(text : String) -> Code raise @utils.Fault {
  let p = @utils.create(text)
  let alphabet = @s.alphabet(p)
  let functions : Array[FunctionDef] = []
  while !p.eof() {
    p.expect_keyword("fn")
    let name = p.ident()
    if p.peek() == @utils.Symbol('(') {
      p.group('(').finish()
    }
    let g = p.group('{')
    let blocks = []
    while !g.eof() {
      g.expect_keyword("label")
      let label = g.ident()
      g.expect(':')
      let h = g.group('{')
      let body = []
      while !h.eof() {
        body.push(read_stmt(h))
      }
      blocks.push({ label, body, })
    }
    let f = { name, blocks, }
    match functions.iter().find_first(old => old.name == name) {
      Some(old) =>
        @utils.require(old == f, "Function '" + name + "' is defined twice")
      None => functions.push(f)
    }
  }
  { alphabet, functions, }
}

///|
pub fn stmt_text(stmt : Stmt) -> String {
  match stmt {
    Lt => "\\LT"
    Rt => "\\RT"
    Assign(dst, src) => @s.ltext(dst) + " := " + @s.rtext(src)
    Break(c) => "\\break" + @s.ctext(c)
    Continue(c) => "\\continue" + @s.ctext(c)
    Return(c) => "\\return" + @s.ctext(c)
    Jump(label, c) => "\\jump " + label + @s.ctext(c)
    Call(n) => "\\call " + n
  }
}

///|
pub fn print_code(code : Code) -> String {
  let seen : Map[String, Bool] = Map([])
  let ordered = []
  fn visit(f : FunctionDef) -> Unit {
    if seen.contains(f.name) {
      return
    }
    seen[f.name] = true
    for b in f.blocks {
      for stmt in b.body {
        match stmt {
          Call(n) =>
            match code.functions.iter().find_first(f => f.name == n) {
              Some(callee) => visit(callee)
              None => ()
            }
          _ => ()
        }
      }
    }
    ordered.push(f)
  }
  for f in code.functions {
    visit(f)
  }
  "\\alphabet: (" +
  @utils.join(code.alphabet, ", ") +
  ")\n" +
  @utils.join(
    ordered.map(f => {
      "\\fn " +
      f.name +
      "() {\n" +
      @utils.join(
        f.blocks.map(b => {
          "  \\label " +
          b.label +
          ": {\n" +
          @utils.join(b.body.map(s => "    " + stmt_text(s) + "\n"), "") +
          "  }\n"
        }),
        "",
      ) +
      "}\n"
    }),
    "\n",
  )
}

///|
fn build_frame(f : FunctionDef) -> Frame {
  {
    function_: f.name,
    env: @s.env_create(collect_vars(f.blocks)),
    blocks: f.blocks,
    pc: (0, 0),
  }
}

///|
fn make(program : Code, input : @s.Tape) -> State raise @utils.Fault {
  validate(program)
  let tape = input.value
  @s.validate_tape(@s.normalized_alphabet(program.alphabet), tape)
  {
    program,
    frame: build_frame(find_function(program, "main")),
    stack: [],
    tape,
  }
}

///|
fn normalized_pc(f : Frame) -> (Int32, Int32) {
  let mut b = f.pc.0
  let mut i = f.pc.1
  while b < f.blocks.length() && i >= f.blocks[b].body.length() {
    b += 1
    i = 0
  }
  (b, i)
}

///|
fn returning(s : State) -> @definition.Transition[State, Routput, Foutput] {
  if s.stack.is_empty() {
    @definition.Halt(@s.Tape::{ value: s.tape, })
  } else {
    @definition.Continue(
      { ..s, frame: s.stack[0], stack: s.stack[1:].to_owned(), },
      (),
    )
  }
}

///|
fn step(
  s : State,
  _input : Rinput,
) -> @definition.Transition[State, Routput, Foutput] raise @utils.Fault {
  let (b, i) = normalized_pc(s.frame)
  if b >= s.frame.blocks.length() {
    return returning(s)
  }
  let stmt = s.frame.blocks[b].body[i]
  let next = { ..s, frame: { ..s.frame, pc: (b, i + 1), }, }
  fn cond(c : @s.Condition?) -> Bool raise @utils.Fault {
    @s.eval_condition(next.frame.env, next.tape, c)
  }
  let next = match stmt {
    Lt => { ..next, tape: next.tape.shift(-1, "-"), }
    Rt => { ..next, tape: next.tape.shift(1, "-"), }
    Assign(dst, src) => {
      let (env, tape) = @s.assign(
        next.frame.env,
        next.tape,
        dst,
        @s.eval(next.frame.env, next.tape, src),
      )
      { ..next, tape, frame: { ..next.frame, env, }, }
    }
    Break(c) if cond(c) => { ..next, frame: { ..next.frame, pc: (b + 1, 0), }, }
    Continue(c) if cond(c) => { ..next, frame: { ..next.frame, pc: (b, 0), }, }
    Jump(label, c) if cond(c) => {
      let target = match next.frame.blocks.search_by(b => b.label == label) {
        Some(i) => i
        None => @utils.error("jump target '" + label + "' not found")
      }
      { ..next, frame: { ..next.frame, pc: (target, 0), }, }
    }
    Return(c) if cond(c) => return returning(next)
    Call(name) => {
      let stack = [next.frame]
      stack.append(next.stack)
      { ..next, stack, frame: build_frame(find_function(next.program, name)), }
    }
    _ => next
  }
  @definition.Continue(next, ())
}

///|
pub fn sj(s : Stmt) -> Json {
  match s {
    Lt => @utils.tag("Lt", [])
    Rt => @utils.tag("Rt", [])
    Assign(dst, src) => @utils.tag("Assign", [@s.lj(dst), @s.rj(src)])
    Break(c) => @utils.tag("Break", [@s.cj(c)])
    Continue(c) => @utils.tag("Continue", [@s.cj(c)])
    Return(c) => @utils.tag("Return", [@s.cj(c)])
    Jump(l, c) => @utils.tag("Jump", [@utils.jstr(l), @s.cj(c)])
    Call(n) => @utils.tag("Call", [@utils.jstr(n)])
  }
}

///|
pub fn sd(j : Json) -> Stmt raise @utils.Fault {
  fn a(i : Int32) -> Json raise @utils.Fault {
    @utils.tag_arg(j, i)
  }
  match @utils.tag_name(j) {
    "Lt" => Lt
    "Rt" => Rt
    "Assign" => Assign(@s.ld(a(0)), @s.rd(a(1)))
    "Break" => Break(@s.cd(a(0)))
    "Continue" => Continue(@s.cd(a(0)))
    "Return" => Return(@s.cd(a(0)))
    "Jump" => Jump(@utils.identifier(@utils.string(a(0))), @s.cd(a(1)))
    "Call" => Call(@utils.identifier(@utils.string(a(0))))
    _ => @utils.error("Invalid tape IR statement")
  }
}

///|
pub fn blocks_json(blocks : Array[Block]) -> Json {
  @utils.jarray(
    blocks.map(b => {
      @utils.object([
        ("label", @utils.jstr(b.label)),
        ("body", @utils.jarray(b.body.map(sj))),
      ])
    }),
  )
}

///|
pub fn decode_blocks(j : Json) -> Array[Block] raise @utils.Fault {
  @utils.array(j).map(j => {
    label: @utils.identifier(@utils.string(@utils.field(j, "label"))),
    body: @utils.array(@utils.field(j, "body")).map(sd),
  })
}

///|
pub fn code_json(code : Code) -> Json {
  @utils.object([
    ("alphabet", @utils.jarray(code.alphabet.map(@s.sign_json))),
    (
      "functions",
      @utils.jarray(
        code.functions.map(f => {
          @utils.object([
            ("name", @utils.jstr(f.name)),
            ("blocks", blocks_json(f.blocks)),
          ])
        }),
      ),
    ),
  ])
}

///|
pub fn decode_code(j : Json) -> Code raise @utils.Fault {
  {
    alphabet: @utils.array(@utils.field(j, "alphabet")).map(@s.decode_sign),
    functions: @utils.array(@utils.field(j, "functions")).map(j => {
      name: @utils.identifier(@utils.string(@utils.field(j, "name"))),
      blocks: decode_blocks(@utils.field(j, "blocks")),
    }),
  }
}

///|
fn fj(f : Frame) -> Json {
  @utils.object([
    ("function_", @utils.jstr(f.function_)),
    ("env", @s.ej(f.env)),
    ("blocks", blocks_json(f.blocks)),
    ("pc", @utils.jarray([@utils.jint(f.pc.0), @utils.jint(f.pc.1)])),
  ])
}

///|
fn fd(j : Json, code : Code) -> Frame raise @utils.Fault {
  let function_ = @utils.identifier(@utils.string(@utils.field(j, "function_")))
  let definition = find_function(code, function_)
  let blocks = decode_blocks(@utils.field(j, "blocks"))
  @utils.require(
    blocks == definition.blocks,
    "Frame differs from function definition",
  )
  let pc = @utils.array(@utils.field(j, "pc"))
  @utils.require(pc.length() == 2, "Invalid frame pc")
  let pc = (@utils.int(pc[0]), @utils.int(pc[1]))
  @utils.require(pc.0 >= 0 && pc.1 >= 0, "Invalid frame pc")
  {
    function_,
    blocks,
    pc,
    env: @s.ed(@utils.field(j, "env"), collect_vars(blocks)),
  }
}

///|
fn encode(s : State) -> Json {
  @utils.object([
    ("program", code_json(s.program)),
    ("frame", fj(s.frame)),
    ("stack", @utils.jarray(s.stack.map(fj))),
    ("tape", s.tape.json(@s.sign_json)),
  ])
}

///|
fn decode(j : Json) -> State raise @utils.Fault {
  let program = decode_code(@utils.field(j, "program"))
  validate(program)
  let tape = @utils.decode_tape(@utils.field(j, "tape"), @s.decode_sign)
  @s.validate_tape(@s.normalized_alphabet(program.alphabet), tape)
  {
    program,
    frame: fd(@utils.field(j, "frame"), program),
    stack: @utils.array(@utils.field(j, "stack")).map(j => fd(j, program)),
    tape,
  }
}

///|
fn render(s : State) -> Array[@definition.Block] {
  let (b, i) = normalized_pc(s.frame)
  let stack = s.stack.rev()
  stack.push(s.frame)
  [
    @definition.titled("function", s.frame.function_),
    @definition.titled("pc", "\{b}:\{i}"),
    @definition.titled(
      "next",
      if b >= s.frame.blocks.length() {
        "halt"
      } else {
        stmt_text(s.frame.blocks[b].body[i])
      },
    ),
    @definition.highlight(
      "stack",
      s.stack.length(),
      stack.map(f => f.function_),
    ),
    @definition.text_table(
      "env",
      ["var", "value"],
      @s.env_named(s.frame.env).map(v => [v.0, v.1]),
    ),
    @definition.highlight("tape", s.tape.left.length(), s.tape.values()),
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
