///|
pub fn[T] env_get(env : Array[(String, T)], name : String) -> T? {
  for pair in env {
    if pair.0 == name {
      return Some(pair.1)
    }
  }
  None
}

///|
pub fn[T] env_lookup(env : Array[(String, T)], name : String) -> T raise Fault {
  match env_get(env, name) {
    Some(value) => value
    None => error("Unknown variable: " + name)
  }
}

///|
pub fn[T] env_set(
  env : Array[(String, T)],
  name : String,
  value : T,
) -> Array[(String, T)] {
  let result = env.map(pair => if pair.0 == name { (name, value) } else { pair })
  if !env.any(pair => pair.0 == name) {
    result.push((name, value))
  }
  result
}

///|
pub fn[T] env_sorted(env : Array[(String, T)]) -> Array[(String, T)] {
  let env = env.copy()
  env.sort_by((a, b) => a.0.compare(b.0))
  env
}

///|
pub fn[T] parse_env(
  text : String,
  parse : (String) -> T raise Fault,
) -> Array[(String, T)] raise Fault {
  let mut env = []
  for line in text.split("\n") {
    let line = line.trim().to_owned()
    if line.is_empty() {
      continue
    }
    let parts = line.split_once("=")
    match parts {
      None => error("Invalid env line: " + line)
      Some((name, value)) =>
        env = env_set(
          env,
          identifier(name.trim().to_owned()),
          parse(value.trim().to_owned()),
        )
    }
  }
  env
}

///|
pub fn[T] print_env(
  env : Array[(String, T)],
  print : (T) -> String,
  trailing_newline~ : Bool = true,
) -> String {
  join(env.map(pair => pair.0 + " = " + print(pair.1)), "\n") +
  (if trailing_newline && !env.is_empty() { "\n" } else { "" })
}

///|
pub fn[T] env_json(env : Array[(String, T)], encode : (T) -> Json) -> Json {
  jarray(env.map(pair => jarray([jstr(pair.0), encode(pair.1)])))
}

///|
pub fn[T] decode_env(
  j : Json,
  decode : (Json) -> T raise Fault,
) -> Array[(String, T)] raise Fault {
  array(j).map(j => {
    let a = array(j)
    require(a.length() == 2, "Invalid environment entry")
    (identifier(string(a[0])), decode(a[1]))
  })
}

///|
pub fn unique(names : Array[String], prefix : String) -> Unit raise Fault {
  let seen : Map[String, Bool] = Map([])
  for name in names {
    require(!seen.contains(name), prefix + name)
    seen[name] = true
  }
}

///|
pub fn scalar(value : String) -> String raise Fault {
  require(value.to_array().length() == 1, "Expected one Unicode scalar")
  value
}

///|
pub fn decode_scalar(j : Json) -> String raise Fault {
  scalar(string(j))
}

///|
pub fn char_text(value : String) -> String {
  "'" +
  (match value {
    "\n" => "\\n"
    "\r" => "\\r"
    "\t" => "\\t"
    "\\" => "\\\\"
    "'" => "\\'"
    _ => value
  }) +
  "'"
}

///|
pub fn read_char(p : Parser) -> String raise Fault {
  if p.peek() == Symbol('(') {
    let g = p.group('(')
    let c = read_char(g)
    g.finish()
    c
  } else {
    p.character()
  }
}

///|
pub fn read_chars(
  p : Parser,
  grouped_characters~ : Bool = true,
) -> Array[String] raise Fault {
  if p.peek() == Symbol('(') {
    let g = p.group('(')
    let cs = read_chars(g, grouped_characters~)
    g.finish()
    return cs
  }
  let g = p.group('[')
  let chars = []
  while !g.eof() {
    chars.push(if grouped_characters { read_char(g) } else { g.character() })
    if g.eof() {
      break
    }
    g.expect(',')
    require(!g.eof(), "Expected character after comma")
  }
  chars
}

///|
pub fn parse_chars(text : String) -> Array[String] raise Fault {
  let p = create(text, character_literals=true)
  let chars = read_chars(p)
  p.finish()
  chars
}

///|
pub fn chars_text(chars : Array[String]) -> String {
  "[" + join(chars.map(char_text), ", ") + "]"
}

///|
#derive(Eq)
pub(all) enum CharTree {
  AtomValue(String)
  ConsValue(CharTree, CharTree)
}

///|
pub fn read_tree(p : Parser) -> CharTree raise Fault {
  if p.peek() == Symbol('(') {
    let g = p.group('(')
    let t = read_tree(g)
    g.finish()
    t
  } else if p.eat_keyword("atom") {
    AtomValue(read_char(p))
  } else {
    p.expect_keyword("cons")
    let a = read_tree(p)
    ConsValue(a, read_tree(p))
  }
}

///|
pub fn parse_tree(text : String) -> CharTree raise Fault {
  let p = create(text, character_literals=true)
  let t = read_tree(p)
  p.finish()
  t
}

///|
pub fn tree_text(tree : CharTree) -> String {
  match tree {
    AtomValue(c) => "\\atom " + char_text(c)
    ConsValue(a, b) => "\\cons " + tree_text(a) + " " + tree_text(b)
  }
}

///|
pub fn tree_json(tree : CharTree) -> Json {
  match tree {
    AtomValue(c) => tag("AtomValue", [jstr(c)])
    ConsValue(a, b) => tag("ConsValue", [tree_json(a), tree_json(b)])
  }
}

///|
pub fn decode_tree(j : Json) -> CharTree raise Fault {
  match tag_name(j) {
    "AtomValue" => AtomValue(decode_scalar(tag_arg(j, 0)))
    "ConsValue" =>
      ConsValue(decode_tree(tag_arg(j, 0)), decode_tree(tag_arg(j, 1)))
    _ => error("Invalid character tree")
  }
}

///|
pub fn natural_list(text : String) -> Array[Nat] raise Fault {
  let p = create(text)
  let g = p.group('(')
  p.finish()
  let ns = []
  while !g.eof() {
    ignore(g.eat('+'))
    ns.push(g.natural())
    if g.eof() {
      break
    }
    g.expect(',')
  }
  ns
}

///|
pub fn natural_list_text(ns : Array[Nat]) -> String {
  "(" + join(ns.map(n => n.to_string()), ", ") + ")"
}
