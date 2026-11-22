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
#derive(Eq)
pub struct Nat {
  decimal : String
}

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
  let mut start = 0
  while start + 1 < digits.length() && digits[start] == '0' {
    start += 1
  }
  { decimal: digits[start:].to_owned(), }
}

///|
pub fn Nat::to_string(self : Nat) -> String {
  self.decimal
}

///|
pub fn Nat::is_zero(self : Nat) -> Bool {
  self.decimal == "0"
}

///|
fn Nat::bigint(self : Nat) -> @bigint.BigInt {
  @string.parse_bigint(self.decimal) catch {
    _ => abort("Invalid natural-number representation")
  }
}

///|
pub fn Nat::to_int(self : Nat) -> Int32 {
  self.bigint().to_int()
}

///|
pub fn Nat::to_double(self : Nat) -> Double {
  self.bigint().to_double()
}

///|
pub fn Nat::compare(self : Nat, other : Nat) -> Int32 {
  let length = self.decimal.length().compare(other.decimal.length())
  if length == 0 {
    self.decimal.compare(other.decimal)
  } else {
    length
  }
}

///|
pub impl Compare for Nat with fn compare(self, other) {
  self.compare(other)
}

///|
pub impl Add for Nat with fn add(self, other) {
  { decimal: (self.bigint() + other.bigint()).to_string(), }
}

///|
pub impl Sub for Nat with fn sub(self, other) {
  if self < other {
    zero()
  } else {
    { decimal: (self.bigint() - other.bigint()).to_string(), }
  }
}

///|
pub impl Mul for Nat with fn mul(self, other) {
  { decimal: (self.bigint() * other.bigint()).to_string(), }
}

///|
pub impl Div for Nat with fn div(self, other) {
  { decimal: (self.bigint() / other.bigint()).to_string(), }
}

///|
pub impl Mod for Nat with fn mod(self, other) {
  { decimal: (self.bigint() % other.bigint()).to_string(), }
}

///|
pub fn zero() -> Nat {
  { decimal: "0", }
}

///|
pub fn one() -> Nat {
  { decimal: "1", }
}

///|
pub fn nint(n : UInt32) -> Nat {
  { decimal: n.to_string(), }
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
pub fn unsigned(text : String) -> UInt32 raise Fault {
  @string.parse_uint(nat(text).decimal) catch {
    _ => error("Expected an unsigned 32-bit integer")
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
pub fn juint(value : UInt32) -> Json {
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
pub fn uint(value : Json) -> UInt32 raise Fault {
  match value {
    Number(n, ..) => {
      require(
        n == n.trunc() && n >= 0.0 && n <= 4294967295.0,
        "Expected an unsigned 32-bit integer",
      )
      n.to_uint()
    }
    _ => error("Expected an unsigned 32-bit integer")
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
