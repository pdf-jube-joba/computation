///|
pub suberror Fault {
  Fault(String)
}

///|
pub fn[T] error(message : String) -> T raise Fault {
  raise Fault(message)
}

///|
pub fn require(condition : Bool, message : String) -> Unit raise Fault {
  if !condition {
    error(message)
  }
}

///|
pub fn unit_input(text : String) -> Unit raise Fault {
  require(text.trim().is_empty(), "Expected empty input")
}

///|
pub type Nat = @bigint.BigInt

///|
pub fn nat(text : String) -> Nat raise Fault {
  let text = text.trim().to_owned()
  let digits = if text.has_prefix("+") {
    text.unsafe_substring(start=1, end=text.length())
  } else {
    text
  }
  require(
    !digits.is_empty() && digits.all(c => c >= '0' && c <= '9'),
    "Expected a natural number",
  )
  @string.parse_bigint(digits) catch {
    _ => error("Expected a natural number")
  }
}

///|
pub fn zero() -> Nat {
  @bigint.BigInt::from_int(0)
}

///|
pub fn one() -> Nat {
  @bigint.BigInt::from_int(1)
}

///|
pub fn nint(n : Int32) -> Nat {
  @bigint.BigInt::from_int(n)
}

///|
pub fn pred(n : Nat) -> Nat {
  if n.is_zero() {
    n
  } else {
    n - one()
  }
}

///|
pub fn sub(a : Nat, b : Nat) -> Nat {
  if a < b {
    zero()
  } else {
    a - b
  }
}

///|
pub fn integer(text : String) -> Int32 raise Fault {
  @string.parse_int(text.trim()) catch {
    _ => error("Integer out of range")
  }
}

///|
pub fn index(n : Nat) -> Int32 raise Fault {
  require(n <= nint(2147483647), "Natural number does not fit in an index")
  n.to_int()
}

///|
pub fn identifier(name : String) -> String raise Fault {
  let chars = name.to_array()
  require(
    chars.length() > 0 && ident_start(chars[0]) && chars.all(ident_continue),
    "Invalid identifier: " + name,
  )
  name
}

///|
pub fn ident_start(c : Char) -> Bool {
  (c >= 'a' && c <= 'z') || (c >= 'A' && c <= 'Z') || c == '_'
}

///|
pub fn ident_continue(c : Char) -> Bool {
  ident_start(c) || (c >= '0' && c <= '9')
}

///|
pub fn join(values : Array[String], separator : String) -> String {
  let mut result = ""
  for i, value in values {
    if i > 0 {
      result += separator
    }
    result += value
  }
  result
}

///|
pub fn parse_json(text : String) -> Json raise Fault {
  @json.parse(text, reject_duplicate_keys=true, preserve_number_repr=true) catch {
    err => error(err.to_string())
  }
}

///|
pub fn object(fields : Array[(String, Json)]) -> Json {
  Json::object(Map::from_iter(fields.iter()))
}

///|
pub fn jstr(value : String) -> Json {
  Json::string(value)
}

///|
pub fn jint(value : Int32) -> Json {
  Json::number(value.to_double())
}

///|
pub fn jbool(value : Bool) -> Json {
  Json::boolean(value)
}

///|
pub fn jarray(values : Array[Json]) -> Json {
  Json::array(values)
}

///|
pub fn field(value : Json, name : String) -> Json raise Fault {
  match value {
    Object(fields) =>
      match fields.get(name) {
        Some(value) => value
        None => error("Missing field: " + name)
      }
    _ => error("Expected an object")
  }
}

///|
pub fn string(value : Json) -> String raise Fault {
  match value {
    String(value) => value
    _ => error("Expected a string")
  }
}

///|
pub fn int(value : Json) -> Int32 raise Fault {
  match value {
    Number(n, ..) => {
      require(
        n == n.trunc() && n >= -2147483648.0 && n <= 2147483647.0,
        "Expected an integer",
      )
      n.to_int()
    }
    _ => error("Expected an integer")
  }
}

///|
pub fn array(value : Json) -> Array[Json] raise Fault {
  match value {
    Array(values) => values
    _ => error("Expected an array")
  }
}

///|
pub fn boolean(value : Json) -> Bool raise Fault {
  match value {
    True => true
    False => false
    _ => error("Expected a boolean")
  }
}

///|
pub fn json_nat(value : Json) -> Nat raise Fault {
  match value {
    String(s) => nat(s)
    Number(_, repr~) => nat(repr.unwrap_or(value.stringify()))
    _ => error("Expected a natural-number decimal string")
  }
}

///|
pub fn tag(name : String, fields : Array[Json]) -> Json {
  let values = [jstr(name)]
  values.append(fields)
  jarray(values)
}

///|
pub fn tag_name(value : Json) -> String raise Fault {
  let values = array(value)
  require(!values.is_empty(), "Expected a tagged value")
  string(values[0])
}

///|
pub fn tag_arg(value : Json, index : Int32) -> Json raise Fault {
  let values = array(value)
  require(index + 1 < values.length(), "Missing tagged argument")
  values[index + 1]
}

///|
pub fn[T] optional_json(value : T?, encode : (T) -> Json) -> Json {
  match value {
    None => Json::null()
    Some(value) => encode(value)
  }
}

///|
pub fn table(
  title : String,
  columns : Array[String],
  rows : Array[Array[String]],
  highlight~ : Int32 = -1,
) -> Json {
  object([
    ("kind", jstr("table")),
    ("title", jstr(title)),
    ("columns", jarray(columns.map(text))),
    (
      "rows",
      jarray(
        rows.mapi((i, cells) => {
          let fields = [("cells", jarray(cells.map(text)))]
          if i == highlight {
            fields.push(("className", jstr("highlight")))
          }
          object(fields)
        }),
      ),
    ),
  ])
}

///|
pub fn text(value : String) -> Json {
  object([("kind", jstr("text")), ("text", jstr(value))])
}

///|
pub fn titled(title : String, value : String) -> Json {
  object([("kind", jstr("text")), ("title", jstr(title)), ("text", jstr(value))])
}

///|
pub fn highlight(title : String, index : Int32, values : Array[String]) -> Json {
  object([
    ("kind", jstr("container")),
    ("title", jstr(title)),
    ("orientation", jstr("horizontal")),
    ("display", jstr("block")),
    (
      "children",
      jarray(
        values.mapi((i, value) => {
          if i == index {
            object([
              ("kind", jstr("text")),
              ("text", jstr(value)),
              ("className", jstr("highlight")),
            ])
          } else {
            text(value)
          }
        }),
      ),
    ),
  ])
}

///|
pub fn container(title : String, children : Array[Json]) -> Json {
  object([
    ("kind", jstr("container")),
    ("title", jstr(title)),
    ("orientation", jstr("vertical")),
    ("display", jstr("block")),
    ("children", jarray(children)),
  ])
}
