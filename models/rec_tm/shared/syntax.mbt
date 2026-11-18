///|
#derive(Eq)
pub(all) enum Lvalue {
  PutVar(String)
  PutHead
}

///|
#derive(Eq)
pub(all) enum Rvalue {
  Var(String)
  Head
  Const(String)
}

///|
#derive(Eq)
pub(all) struct Condition {
  left : Rvalue
  right : Rvalue
}

///|
#derive(Eq)
pub(all) struct Environment {
  names : Array[(String, Int32)]
  values : Array[(Int32, String)]
}

///|
pub fn sign(text : String) -> String raise @base.Fault {
  let text = text.trim().to_owned()
  if text == "-" {
    text
  } else {
    @base.identifier(text)
  }
}

///|
pub fn read_sign(p : @p.Parser) -> String raise @base.Fault {
  if p.eat('-') {
    "-"
  } else {
    p.ident()
  }
}

///|
pub fn alphabet(p : @p.Parser) -> Array[String] raise @base.Fault {
  p.expect_text("\\alphabet:")
  let g = p.group('(')
  let signs = []
  while !g.eof() {
    signs.push(read_sign(g))
    if g.eof() {
      break
    }
    g.expect(',')
  }
  signs
}

///|
pub fn normalized_alphabet(
  signs : Array[String],
) -> Array[String] raise @base.Fault {
  @base.require(!signs.is_empty(), "\\alphabet must not be empty")
  let result = []
  for s in signs {
    ignore(sign(s))
    if !result.contains(s) {
      result.push(s)
    }
  }
  if !result.contains("-") {
    result.push("-")
  }
  result
}

///|
pub fn lvalue(p : @p.Parser) -> Lvalue raise @base.Fault {
  if p.eat('@') {
    PutHead
  } else {
    PutVar(p.ident())
  }
}

///|
pub fn rvalue(p : @p.Parser) -> Rvalue raise @base.Fault {
  if p.eat('@') {
    Head
  } else if p.eat_keyword("const") {
    Const(read_sign(p))
  } else {
    Var(p.ident())
  }
}

///|
pub fn condition(p : @p.Parser) -> Condition? raise @base.Fault {
  if p.eat_keyword("if") {
    let left = rvalue(p)
    p.expect_text("==")
    Some({ left, right: rvalue(p), })
  } else {
    None
  }
}

///|
pub fn ltext(v : Lvalue) -> String {
  match v {
    PutVar(n) => n
    PutHead => "@"
  }
}

///|
pub fn rtext(v : Rvalue) -> String {
  match v {
    Var(n) => n
    Head => "@"
    Const(s) => "\\const " + s
  }
}

///|
pub fn ctext(c : Condition?) -> String {
  match c {
    None => ""
    Some(c) => " \\if " + rtext(c.left) + " == " + rtext(c.right)
  }
}

///|
pub fn lvars(v : Lvalue) -> Array[String] {
  match v {
    PutVar(n) => [n]
    PutHead => []
  }
}

///|
pub fn rvars(v : Rvalue) -> Array[String] {
  match v {
    Var(n) => [n]
    _ => []
  }
}

///|
pub fn cvars(c : Condition?) -> Array[String] {
  match c {
    None => []
    Some(c) => {
      let names = rvars(c.left)
      names.append(rvars(c.right))
      names
    }
  }
}

///|
pub fn validate_value(
  allowed : Array[String],
  v : Rvalue,
) -> Unit raise @base.Fault {
  match v {
    Const(s) =>
      @base.require(allowed.contains(s), "Unknown sign in \\const: " + s)
    _ => ()
  }
}

///|
pub fn validate_condition(
  allowed : Array[String],
  c : Condition?,
) -> Unit raise @base.Fault {
  match c {
    None => ()
    Some(c) => {
      validate_value(allowed, c.left)
      validate_value(allowed, c.right)
    }
  }
}

///|
pub fn validate_tape(
  allowed : Array[String],
  tape : @data.Tape[String],
) -> Unit raise @base.Fault {
  for s in tape.values() {
    @base.require(allowed.contains(s), "Unknown sign on tape: " + s)
  }
}

///|
pub fn env_create(vars : Array[String]) -> Environment {
  let names : Array[String] = []
  for n in vars {
    if !names.contains(n) {
      names.push(n)
    }
  }
  names.sort()
  {
    names: names.mapi((i, n) => (n, i)),
    values: names.mapi((i, _) => (i, "-")),
  }
}

///|
pub fn variable(env : Environment, name : String) -> Int32 raise @base.Fault {
  for pair in env.names {
    if pair.0 == name {
      return pair.1
    }
  }
  @base.error("Unknown variable: " + name)
}

///|
pub fn env_named(env : Environment) -> Array[(String, String)] {
  env.names.map(pair => {
    let value = env.values.iter().find_first(v => v.0 == pair.1)
    (
      pair.0,
      match value {
        Some(v) => v.1
        None => "-"
      },
    )
  })
}

///|
pub fn eval(
  env : Environment,
  tape : @data.Tape[String],
  v : Rvalue,
) -> String raise @base.Fault {
  match v {
    Const(s) => s
    Head => tape.head
    Var(n) => {
      let id = variable(env, n)
      match env.values.iter().find_first(v => v.0 == id) {
        Some(v) => v.1
        None => "-"
      }
    }
  }
}

///|
pub fn eval_condition(
  env : Environment,
  tape : @data.Tape[String],
  c : Condition?,
) -> Bool raise @base.Fault {
  match c {
    None => true
    Some(c) => {
      let a = eval(env, tape, c.left)
      a == eval(env, tape, c.right)
    }
  }
}

///|
pub fn assign(
  env : Environment,
  tape : @data.Tape[String],
  dst : Lvalue,
  value : String,
) -> (Environment, @data.Tape[String]) raise @base.Fault {
  match dst {
    PutHead => (env, tape.write(value))
    PutVar(n) => {
      let id = variable(env, n)
      let values = env.values.filter(v => v.0 != id)
      values.push((id, value))
      values.sort_by((a, b) => a.0.compare(b.0))
      ({ ..env, values, }, tape)
    }
  }
}

///|
pub fn sign_json(s : String) -> Json {
  if s == "-" {
    Json::null()
  } else {
    @base.jstr(s)
  }
}

///|
pub fn decode_sign(j : Json) -> String raise @base.Fault {
  match j {
    Null => "-"
    _ => @base.identifier(@base.string(j))
  }
}

///|
pub fn lj(v : Lvalue) -> Json {
  match v {
    PutHead => @base.tag("Head", [])
    PutVar(n) => @base.tag("Var", [@base.jstr(n)])
  }
}

///|
pub fn ld(j : Json) -> Lvalue raise @base.Fault {
  match @base.tag_name(j) {
    "Head" => PutHead
    "Var" => PutVar(@base.identifier(@base.string(@base.tag_arg(j, 0))))
    _ => @base.error("Invalid tape lvalue")
  }
}

///|
pub fn rj(v : Rvalue) -> Json {
  match v {
    Head => @base.tag("Head", [])
    Var(n) => @base.tag("Var", [@base.jstr(n)])
    Const(s) => @base.tag("Const", [sign_json(s)])
  }
}

///|
pub fn rd(j : Json) -> Rvalue raise @base.Fault {
  match @base.tag_name(j) {
    "Head" => Head
    "Var" => Var(@base.identifier(@base.string(@base.tag_arg(j, 0))))
    "Const" => Const(decode_sign(@base.tag_arg(j, 0)))
    _ => @base.error("Invalid tape rvalue")
  }
}

///|
pub fn cj(c : Condition?) -> Json {
  @base.optional_json(c, c => {
    @base.object([("left", rj(c.left)), ("right", rj(c.right))])
  })
}

///|
pub fn cd(j : Json) -> Condition? raise @base.Fault {
  match j {
    Null => None
    _ =>
      Some({
        left: rd(@base.field(j, "left")),
        right: rd(@base.field(j, "right")),
      })
  }
}

///|
pub fn ej(env : Environment) -> Json {
  @base.object([
    (
      "names",
      @base.jarray(
        env.names.map(v => @base.jarray([@base.jstr(v.0), @base.jint(v.1)])),
      ),
    ),
    (
      "values",
      @base.jarray(
        env.values.map(v => @base.jarray([@base.jint(v.0), sign_json(v.1)])),
      ),
    ),
  ])
}

///|
pub fn ed(j : Json, vars : Array[String]) -> Environment raise @base.Fault {
  let names = @base.array(@base.field(j, "names")).map(j => {
    let a = @base.array(j)
    @base.require(a.length() == 2, "Invalid environment name")
    (@base.identifier(@base.string(a[0])), @base.int(a[1]))
  })
  let expected = env_create(vars)
  @base.require(
    names == expected.names,
    "Inconsistent variable IDs in environment",
  )
  let values = @base.array(@base.field(j, "values")).map(j => {
    let a = @base.array(j)
    @base.require(a.length() == 2, "Invalid environment value")
    (@base.int(a[0]), decode_sign(a[1]))
  })
  let ids = values.map(v => v.0)
  @base.require(
    ids.length() == names.length() && names.all(v => ids.contains(v.1)),
    "Inconsistent environment values",
  )
  { names, values, }
}
