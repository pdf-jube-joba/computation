///|
#derive(Eq, Debug)
pub(all) enum Token {
  Ident(String)
  Keyword(String)
  Number(String)
  StringLit(String)
  Character(String)
  Symbol(Char)
  End
}

///|
pub struct Parser {
  source : String
  tokens : Array[Token]
  offsets : Array[Int32]
  matches : Array[Int32]
  mut pos : Int32
  limit : Int32
}

///|
pub fn create(
  text : String,
  character_literals~ : Bool = false,
  nested_comments~ : Bool = true,
) -> Parser raise @base.Fault {
  let chars = text.to_array()
  let tokens = []
  let offsets = []
  let mut i = 0
  let mut byte_offset = 0
  fn push(token, offset) {
    tokens.push(token)
    offsets.push(offset)
  }
  fn slice(start, end) {
    String::from_array(chars[start:end])
  }
  while i < chars.length() {
    let c = chars[i]
    let begin = i
    let offset = byte_offset
    if c == ' ' || c == '\t' || c == '\n' || c == '\r' {
      i += 1
    } else if c == '/' && i + 1 < chars.length() && chars[i + 1] == '/' {
      while i < chars.length() && chars[i] != '\n' {
        i += 1
      }
    } else if c == '/' && i + 1 < chars.length() && chars[i + 1] == '*' {
      i += 2
      let mut depth = 1
      while i < chars.length() && depth > 0 {
        if i + 1 < chars.length() &&
          chars[i] == '/' &&
          chars[i + 1] == '*' &&
          nested_comments {
          depth += 1
          i += 2
        } else if i + 1 < chars.length() &&
          chars[i] == '*' &&
          chars[i + 1] == '/' {
          depth -= 1
          i += 2
        } else {
          i += 1
        }
      }
      @base.require(depth == 0, "Unclosed comment at byte \{offset}")
    } else if c == '"' {
      i += 1
      let mut escaped = false
      while i < chars.length() && (escaped || chars[i] != '"') {
        if escaped {
          escaped = false
        } else if chars[i] == '\\' {
          escaped = true
        }
        i += 1
      }
      @base.require(i < chars.length(), "Unclosed string at byte \{offset}")
      i += 1
      push(StringLit(@base.string(@base.parse_json(slice(begin, i)))), offset)
    } else if c == '\'' && character_literals {
      i += 1
      @base.require(i < chars.length(), "Unclosed character literal")
      let value = if chars[i] == '\\' {
        i += 1
        @base.require(i < chars.length(), "Unclosed character escape")
        let value = match chars[i] {
          'n' => "\n"
          'r' => "\r"
          't' => "\t"
          '\\' => "\\"
          '\'' => "'"
          other => other.to_string()
        }
        i += 1
        value
      } else {
        let value = chars[i].to_string()
        i += 1
        value
      }
      @base.require(
        i < chars.length() && chars[i] == '\'',
        "Expected closing quote",
      )
      i += 1
      push(Character(value), offset)
    } else if c == '\\' &&
      i + 1 < chars.length() &&
      @base.ident_start(chars[i + 1]) {
      i += 2
      while i < chars.length() && @base.ident_continue(chars[i]) {
        i += 1
      }
      push(Keyword(slice(begin + 1, i)), offset)
    } else if @base.ident_start(c) || (c >= '0' && c <= '9') {
      let number = c >= '0' && c <= '9'
      i += 1
      while i < chars.length() &&
            (if number {
              chars[i] >= '0' && chars[i] <= '9'
            } else {
              @base.ident_continue(chars[i])
            }) {
        i += 1
      }
      push(
        if number {
          Number(slice(begin, i))
        } else {
          Ident(slice(begin, i))
        },
        offset,
      )
    } else if c.to_int() > 0 && c.to_int() < 128 {
      push(Symbol(c), offset)
      i += 1
    } else {
      @base.error("Unexpected character at byte \{offset}")
    }
    byte_offset += @utf8.encode(slice(begin, i)).length()
  }
  push(End, byte_offset)
  let matches = Array::make(tokens.length(), -1)
  let stack : Array[(Int32, Char)] = []
  for i, token in tokens {
    match token {
      Symbol('(') => stack.push((i, ')'))
      Symbol('[') => stack.push((i, ']'))
      Symbol('{') => stack.push((i, '}'))
      Symbol(')' | ']' | '}' as c) =>
        match stack.pop() {
          None =>
            @base.error("Unexpected closing delimiter at byte \{offsets[i]}")
          Some((opening, expected)) => {
            @base.require(
              c == expected,
              "Mismatched delimiter at byte \{offsets[i]}",
            )
            matches[opening] = i
          }
        }
      _ => ()
    }
  }
  @base.require(stack.is_empty(), "Unclosed delimiter")
  {
    source: text,
    tokens,
    offsets,
    matches,
    pos: 0,
    limit: tokens.length() - 1,
  }
}

///|
pub fn Parser::peek(self : Parser) -> Token {
  if self.pos >= self.limit {
    End
  } else {
    self.tokens[self.pos]
  }
}

///|
pub fn Parser::bump(self : Parser) -> Token {
  let token = self.peek()
  if token != End {
    self.pos += 1
  }
  token
}

///|
pub fn Parser::eof(self : Parser) -> Bool {
  self.peek() == End
}

///|
pub fn[T] Parser::error(self : Parser, message : String) -> T raise @base.Fault {
  @base.error(message + " at byte " + self.offsets[self.pos].to_string())
}

///|
pub fn Parser::finish(self : Parser) -> Unit raise @base.Fault {
  if !self.eof() {
    self.error("Unexpected trailing tokens")
  }
}

///|
pub fn Parser::eat(self : Parser, c : Char) -> Bool {
  if self.peek() == Symbol(c) {
    ignore(self.bump())
    true
  } else {
    false
  }
}

///|
pub fn Parser::expect(self : Parser, c : Char) -> Unit raise @base.Fault {
  if !self.eat(c) {
    self.error("Expected '\{c}'")
  }
}

///|
pub fn Parser::eat_keyword(self : Parser, word : String) -> Bool {
  if self.peek() == Keyword(word) {
    ignore(self.bump())
    true
  } else {
    false
  }
}

///|
pub fn Parser::expect_keyword(
  self : Parser,
  word : String,
) -> Unit raise @base.Fault {
  if !self.eat_keyword(word) {
    self.error("Expected \\" + word)
  }
}

///|
pub fn Parser::keyword(self : Parser) -> String raise @base.Fault {
  match self.bump() {
    Keyword(word) => word
    _ => self.error("Expected keyword")
  }
}

///|
pub fn Parser::ident(self : Parser) -> String raise @base.Fault {
  match self.bump() {
    Ident(word) => word
    _ => self.error("Expected identifier")
  }
}

///|
pub fn Parser::natural(self : Parser) -> @base.Nat raise @base.Fault {
  match self.bump() {
    Number(word) => @base.nat(word)
    _ => self.error("Expected natural number")
  }
}

///|
pub fn Parser::integer(self : Parser) -> Int32 raise @base.Fault {
  @base.index(self.natural())
}

///|
pub fn Parser::string_literal(self : Parser) -> String raise @base.Fault {
  match self.bump() {
    StringLit(value) => value
    _ => self.error("Expected string literal")
  }
}

///|
pub fn Parser::character(self : Parser) -> String raise @base.Fault {
  match self.bump() {
    Character(value) => value
    _ => self.error("Expected character literal")
  }
}

///|
pub fn Parser::eat_text(self : Parser, text : String) -> Bool raise @base.Fault {
  let wanted = create(text)
  let saved = self.pos
  while !wanted.eof() {
    if self.bump() != wanted.bump() {
      self.pos = saved
      return false
    }
  }
  true
}

///|
pub fn Parser::expect_text(
  self : Parser,
  text : String,
) -> Unit raise @base.Fault {
  if !self.eat_text(text) {
    self.error("Expected " + text)
  }
}

///|
pub fn Parser::group(self : Parser, opening : Char) -> Parser raise @base.Fault {
  if self.peek() != Symbol(opening) {
    self.error("Expected '\{opening}'")
  }
  let end = self.matches[self.pos]
  @base.require(end >= 0, "Expected a group")
  let inner = { ..self, pos: self.pos + 1, limit: end, }
  self.pos = end + 1
  inner
}

///|
pub fn Parser::position(self : Parser) -> Int32 {
  self.pos
}

///|
pub fn Parser::rewind(self : Parser, pos : Int32) -> Unit {
  self.pos = pos
}

///|
pub fn Parser::offset(self : Parser) -> Int32 {
  self.offsets[self.pos]
}
