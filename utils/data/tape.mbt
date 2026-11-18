///|
pub(all) struct Tape[T] {
  left : Array[T]
  head : T
  right : Array[T]
}

///|
pub fn[T] blank(value : T) -> Tape[T] {
  { left: [], head: value, right: [], }
}

///|
pub fn[T] Tape::shift(
  self : Tape[T],
  direction : Int32,
  default : T,
) -> Tape[T] {
  if direction == 0 {
    self
  } else if direction < 0 {
    let right = [self.head]
    right.append(self.right)
    if self.left.is_empty() {
      { left: [], head: default, right, }
    } else {
      { left: self.left[1:].to_owned(), head: self.left[0], right, }
    }
  } else {
    let left = [self.head]
    left.append(self.left)
    if self.right.is_empty() {
      { left, head: default, right: [], }
    } else {
      { left, head: self.right[0], right: self.right[1:].to_owned(), }
    }
  }
}

///|
pub fn[T] Tape::write(self : Tape[T], value : T) -> Tape[T] {
  { ..self, head: value, }
}

///|
pub fn[T] Tape::values(self : Tape[T]) -> Array[T] {
  let values = self.left.copy()
  values.rev_in_place()
  values.push(self.head)
  values.append(self.right)
  values
}

///|
pub fn[T] parse_tape(
  text : String,
  parse : (String) -> T raise @base.Fault,
) -> Tape[T] raise @base.Fault {
  let parts = text.split("|").map(view => view.to_owned()).collect()
  @base.require(parts.length() == 3, "Expected tape ...|head|...")
  fn side(text : String) -> Array[T] raise @base.Fault {
    if text.trim().is_empty() {
      []
    } else {
      text.split(",").map(value => value.trim().to_owned()).collect().map(parse)
    }
  }
  let left = side(parts[0])
  left.rev_in_place()
  { left, head: parse(parts[1].trim().to_owned()), right: side(parts[2]), }
}

///|
pub fn[T] Tape::text(self : Tape[T], print : (T) -> String) -> String {
  let left = self.left.copy()
  left.rev_in_place()
  @base.join(left.map(print), ",") +
  "|" +
  print(self.head) +
  "|" +
  @base.join(self.right.map(print), ",")
}

///|
pub fn[T] Tape::json(self : Tape[T], encode : (T) -> Json) -> Json {
  @base.object([
    ("left", @base.jarray(self.left.map(encode))),
    ("head", encode(self.head)),
    ("right", @base.jarray(self.right.map(encode))),
  ])
}

///|
pub fn[T] decode_tape(
  value : Json,
  decode : (Json) -> T raise @base.Fault,
) -> Tape[T] raise @base.Fault {
  {
    left: @base.array(@base.field(value, "left")).map(decode),
    head: decode(@base.field(value, "head")),
    right: @base.array(@base.field(value, "right")).map(decode),
  }
}

///|
pub(all) struct LoopCode {
  instructions : Array[String]
  jumps : Array[Int32]
}

///|
pub fn loop_code(
  tokens : Array[String],
  opening : String,
  closing : String,
  allow_empty : Bool,
) -> LoopCode raise @base.Fault {
  @base.require(
    allow_empty || !tokens.is_empty(),
    "Expected a nonempty program",
  )
  let stack : Array[Int32] = []
  let jumps = Array::make(tokens.length(), -1)
  for i, token in tokens {
    if token == opening {
      stack.push(i)
    } else if token == closing {
      match stack.pop() {
        None => @base.error("Unmatched closing loop")
        Some(start) => {
          @base.require(allow_empty || i != start + 1, "Empty loop body")
          jumps[i] = start
          jumps[start] = i
        }
      }
    }
  }
  @base.require(stack.is_empty(), "Unclosed loop")
  { instructions: tokens, jumps, }
}
