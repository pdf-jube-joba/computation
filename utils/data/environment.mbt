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
pub fn[T] env_lookup(
  env : Array[(String, T)],
  name : String,
) -> T raise @base.Fault {
  match env_get(env, name) {
    Some(value) => value
    None => @base.error("Unknown variable: " + name)
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
  parse : (String) -> T raise @base.Fault,
) -> Array[(String, T)] raise @base.Fault {
  let mut env = []
  for line in text.split("\n") {
    let line = line.trim().to_owned()
    if line.is_empty() {
      continue
    }
    let parts = line.split_once("=")
    match parts {
      None => @base.error("Invalid env line: " + line)
      Some((name, value)) =>
        env = env_set(
          env,
          @base.identifier(name.trim().to_owned()),
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
  @base.join(env.map(pair => pair.0 + " = " + print(pair.1)), "\n") +
  (if trailing_newline && !env.is_empty() { "\n" } else { "" })
}

///|
pub fn[T] env_json(env : Array[(String, T)], encode : (T) -> Json) -> Json {
  @base.jarray(
    env.map(pair => @base.jarray([@base.jstr(pair.0), encode(pair.1)])),
  )
}

///|
pub fn[T] decode_env(
  j : Json,
  decode : (Json) -> T raise @base.Fault,
) -> Array[(String, T)] raise @base.Fault {
  @base.array(j).map(j => {
    let a = @base.array(j)
    @base.require(a.length() == 2, "Invalid environment entry")
    (@base.identifier(@base.string(a[0])), decode(a[1]))
  })
}

///|
pub fn unique(names : Array[String], prefix : String) -> Unit raise @base.Fault {
  let seen : Map[String, Bool] = Map([])
  for name in names {
    @base.require(!seen.contains(name), prefix + name)
    seen[name] = true
  }
}

///|
pub fn scalar(value : String) -> String raise @base.Fault {
  @base.require(value.to_array().length() == 1, "Expected one Unicode scalar")
  value
}

///|
pub fn decode_scalar(j : Json) -> String raise @base.Fault {
  scalar(@base.string(j))
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
pub fn read_char(p : @p.Parser) -> String raise @base.Fault {
  if p.peek() == @p.Symbol('(') {
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
  p : @p.Parser,
  grouped_characters~ : Bool = true,
) -> Array[String] raise @base.Fault {
  if p.peek() == @p.Symbol('(') {
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
    @base.require(!g.eof(), "Expected character after comma")
  }
  chars
}

///|
pub fn parse_chars(text : String) -> Array[String] raise @base.Fault {
  let p = @p.create(text, character_literals=true)
  let chars = read_chars(p)
  p.finish()
  chars
}

///|
pub fn chars_text(chars : Array[String]) -> String {
  "[" + @base.join(chars.map(char_text), ", ") + "]"
}

///|
#derive(Eq)
pub(all) enum CharTree {
  AtomValue(String)
  ConsValue(CharTree, CharTree)
}

///|
pub fn read_tree(p : @p.Parser) -> CharTree raise @base.Fault {
  if p.peek() == @p.Symbol('(') {
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
pub fn parse_tree(text : String) -> CharTree raise @base.Fault {
  let p = @p.create(text, character_literals=true)
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
    AtomValue(c) => @base.tag("AtomValue", [@base.jstr(c)])
    ConsValue(a, b) => @base.tag("ConsValue", [tree_json(a), tree_json(b)])
  }
}

///|
pub fn decode_tree(j : Json) -> CharTree raise @base.Fault {
  match @base.tag_name(j) {
    "AtomValue" => AtomValue(decode_scalar(@base.tag_arg(j, 0)))
    "ConsValue" =>
      ConsValue(
        decode_tree(@base.tag_arg(j, 0)),
        decode_tree(@base.tag_arg(j, 1)),
      )
    _ => @base.error("Invalid character tree")
  }
}

///|
pub fn natural_list(text : String) -> Array[@base.Nat] raise @base.Fault {
  let p = @p.create(text)
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
pub fn natural_list_text(ns : Array[@base.Nat]) -> String {
  "(" + @base.join(ns.map(n => n.to_string()), ", ") + ")"
}
