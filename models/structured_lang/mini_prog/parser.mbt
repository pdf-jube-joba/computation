///|
fn type_atom(p : @utils.Parser) -> Ty raise @utils.Fault {
  match p.keyword() {
    "num" => Num
    "u8" => U8
    "bool" => Bool
    "unit" => Unit
    "ptr" => Ptr
    "fn" => Fn
    _ => p.error("Expected type atom")
  }
}

///|
fn parse_type(p : @utils.Parser) -> Ty raise @utils.Fault {
  if p.peek() == @utils.Symbol('[') {
    let g = p.group('[')
    let t = parse_type(g)
    g.expect(';')
    let n = g.integer()
    g.finish()
    Array(t, n)
  } else {
    let first = type_atom(p)
    let types = [first]
    let mut kind = 0
    while true {
      let next = if p.eat('*') { 1 } else if p.eat('+') { 2 } else { break }
      @utils.require(
        kind == 0 || kind == next,
        "cannot mix '*' and '+' in a type without brackets",
      )
      kind = next
      types.push(type_atom(p))
    }
    match kind {
      1 => Product(types)
      2 => Sum(types)
      _ => first
    }
  }
}

///|
fn parse_place(p : @utils.Parser) -> Place raise @utils.Fault {
  let mut place = if p.eat_keyword("static") {
    Static(p.ident())
  } else if p.eat_keyword("local") {
    Local(p.ident())
  } else {
    let g = p.group('(')
    let v = parse_value(g, p => p.eof())
    g.finish()
    p.expect_keyword("loc")
    Deref(v)
  }
  while true {
    if p.eat('.') {
      place = Field(place, p.integer())
    } else if p.eat('?') {
      place = Tag_place(place, p.integer())
    } else if p.peek() == @utils.Symbol('[') {
      let g = p.group('[')
      let v = parse_value(g, p => p.eof())
      g.finish()
      place = Index(place, v)
    } else {
      break
    }
  }
  place
}

///|
/// Source expressions use an explicit postfix stack.
fn parse_value(
  p : @utils.Parser,
  stop : (@utils.Parser) -> Bool,
) -> Expr raise @utils.Fault {
  let stack = []
  fn pop() -> Expr raise @utils.Fault {
    match stack.pop() {
      Some(v) => v
      None => p.error("operator needs an operand")
    }
  }
  while !p.eof() && !stop(p) {
    match p.peek() {
      @utils.Number(_) => stack.push(Number(p.natural()))
      @utils.Character(c) => {
        ignore(p.bump())
        stack.push(Char(c))
      }
      _ =>
        if p.eat_keyword("true") {
          stack.push(Boolean(true))
        } else if p.eat_keyword("false") {
          stack.push(Boolean(false))
        } else if p.eat_keyword("unit") {
          stack.push(Unit_value)
        } else if p.eat_keyword("null_ptr") {
          stack.push(NullPtr)
        } else if p.eat_keyword("null_fn") {
          stack.push(NullFn)
        } else if p.eat_keyword("fn") {
          stack.push(Function(p.ident()))
        } else if p.eat_keyword("ld") {
          stack.push(Load(parse_place(p)))
        } else if p.peek() == @utils.Symbol('(') ||
          p.peek() == @utils.Keyword("static") ||
          p.peek() == @utils.Keyword("local") {
          let place = parse_place(p)
          p.expect_keyword("addr")
          stack.push(Addr(place))
        } else if p.eat('!') {
          stack.push(UnOp(Not, pop()))
        } else if p.eat_keyword("pair") {
          let g = p.group('(')
          let count = g.integer()
          g.finish()
          @utils.require(
            count <= stack.length(),
            "\\pair arity exceeds expression stack",
          )
          let values = stack[stack.length() - count:].to_owned()
          for _ in 0..<count {
            ignore(stack.pop())
          }
          stack.push(Pair(values))
        } else if p.eat_keyword("tag") {
          let g = p.group('(')
          let tag = g.integer()
          g.finish()
          stack.push(Tag(tag, pop()))
        } else {
          let op = if p.eat('+') {
            Add
          } else if p.eat('-') {
            Sub
          } else if p.eat_text("==") {
            Eq
          } else if p.eat('<') {
            Lt
          } else if p.eat_text("&&") {
            And
          } else if p.eat_text("||") {
            Or
          } else {
            p.error("Unexpected token in value expression")
          }
          let b = pop()
          let a = pop()
          stack.push(BinOp(a, op, b))
        }
    }
  }
  @utils.require(
    stack.length() == 1,
    "value expression did not reduce to a single value",
  )
  stack[0]
}

///|
fn terminator(p : @utils.Parser) -> Bool {
  p.eof() || p.peek() == @utils.Symbol(';')
}

///|
fn statement_start(p : @utils.Parser) -> Bool {
  match p.peek() {
    @utils.Keyword(
      "assign"
      | "if"
      | "case"
      | "halloc"
      | "hfree"
      | "loop"
      | "break"
      | "continue"
      | "call"
      | "return"
      | "block",
    ) => true
    _ => false
  }
}

///|
fn value_list(
  p : @utils.Parser,
  stop : (@utils.Parser) -> Bool,
) -> Array[Expr] raise @utils.Fault {
  let values = []
  if !p.eof() && !stop(p) {
    values.push(parse_value(p, p => p.peek() == @utils.Symbol(',') || stop(p)))
    while p.eat(',') {
      values.push(
        parse_value(p, p => p.peek() == @utils.Symbol(',') || stop(p)),
      )
    }
  }
  values
}

///|
fn parse_stmt(p : @utils.Parser) -> Stmt raise @utils.Fault {
  if p.eat_keyword("assign") {
    let place = parse_place(p)
    p.expect_text(":=")
    Assign(place, parse_value(p, terminator))
  } else if p.eat_keyword("if") {
    let c = parse_value(p, statement_start)
    If(c, parse_stmt(p))
  } else if p.eat_keyword("case") {
    let tag = p.integer()
    p.expect_keyword("of")
    let value = parse_value(p, statement_start)
    Case(tag, value, parse_stmt(p))
  } else if p.eat_keyword("halloc") {
    let ty = parse_type(p)
    p.expect_text("->")
    HAlloc(ty, parse_place(p))
  } else if p.eat_keyword("hfree") {
    HFree(parse_value(p, terminator))
  } else if p.eat_keyword("loop") {
    let label = p.ident()
    p.expect(':')
    Loop(label, parse_stmt(p))
  } else if p.eat_keyword("break") {
    Break(p.ident())
  } else if p.eat_keyword("continue") {
    Continue(p.ident())
  } else if p.eat_keyword("call") {
    let callee = parse_value(p, p => p.peek() == @utils.Symbol('('))
    let g = p.group('(')
    let args = value_list(g, p => p.eof())
    g.finish()
    p.expect_text("->")
    let places = [parse_place(p)]
    while p.eat(',') {
      places.push(parse_place(p))
    }
    Call(callee, args, places)
  } else if p.eat_keyword("return") {
    Return(value_list(p, terminator))
  } else {
    p.expect_keyword("block")
    Block(parse_block(p))
  }
}

///|
fn parse_block(p : @utils.Parser) -> Block raise @utils.Fault {
  let g = p.group('(')
  let bindings = []
  if !g.eof() {
    while true {
      let name = g.ident()
      g.expect(':')
      bindings.push((name, parse_type(g)))
      if !g.eat(',') {
        break
      }
    }
  }
  g.finish()
  let g = p.group('{')
  let stmts = []
  while !g.eof() {
    stmts.push(parse_stmt(g))
    if !g.eat(';') {
      break
    }
  }
  g.finish()
  { bindings, stmts, }
}

///|
fn parse_code(text : String) -> Code raise @utils.Fault {
  let p = @utils.create(text, character_literals=true)
  let statics = []
  let functions = []
  while !p.eof() {
    if p.eat_keyword("static") {
      let name = p.ident()
      p.expect(':')
      statics.push({ name, ty: parse_type(p), })
    } else {
      p.expect_keyword("fn")
      let name = p.ident()
      functions.push({ name, block: parse_block(p), })
    }
  }
  { statics, functions, }
}
