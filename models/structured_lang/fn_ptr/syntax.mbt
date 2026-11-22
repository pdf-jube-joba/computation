///|
priv enum Binop {
  Add
  Sub
}

///|
priv enum Place {
  Var(String)
  Deref(Expr)
}

///|
priv enum Expr {
  Number(Int64)
  NullPtr
  BinOp(Expr, Binop, Expr)
  Load(Place)
  Addr(Place)
}

///|
priv enum Stmt {
  Assign(Place, Expr)
  Ifz(Expr, Stmt, Stmt)
  Call(String, Array[Expr])
  Return
}

///|
priv struct Function {
  name : String
  params : Array[String]
  body : Array[Stmt]
}

///|
struct Code {
  functions : Array[Function]
}

///|
fn parse_place(p : @utils.Parser) -> Place raise @utils.Fault {
  match p.peek() {
    @utils.Ident(_) => Var(p.ident())
    _ => {
      let g = p.group('(')
      let v = parse_expr(g)
      g.finish()
      p.expect_keyword("loc")
      Deref(v)
    }
  }
}

///|
fn atom(p : @utils.Parser) -> Expr raise @utils.Fault {
  if p.eat_keyword("null_ptr") {
    NullPtr
  } else if p.eat_keyword("ld") {
    Load(parse_place(p))
  } else if p.peek() == @utils.Symbol('(') {
    let g = p.group('(')
    let v = parse_expr(g)
    g.finish()
    v
  } else if p.peek() is @utils.Number(_) {
    let n = p.natural()
    @utils.require(
      n <= @utils.nat("9223372036854775807"),
      "Integer out of range",
    )
    Number(
      @string.parse_int64(n.to_string()) catch {
        _ => @utils.error("Integer out of range")
      },
    )
  } else {
    let place = parse_place(p)
    p.expect_keyword("addr")
    Addr(place)
  }
}

///|
fn parse_expr(p : @utils.Parser) -> Expr raise @utils.Fault {
  let mut value = atom(p)
  while true {
    if p.eat('+') {
      value = BinOp(value, Add, atom(p))
    } else if p.eat('-') {
      value = BinOp(value, Sub, atom(p))
    } else {
      break
    }
  }
  value
}

///|
fn parse_stmt(p : @utils.Parser) -> Stmt raise @utils.Fault {
  if p.eat_keyword("assign") {
    let place = parse_place(p)
    p.expect_text(":=")
    Assign(place, parse_expr(p))
  } else if p.eat_keyword("ifz") {
    let c = parse_expr(p)
    p.expect_keyword("then")
    let a = parse_stmt(p)
    p.expect_keyword("else")
    let b = parse_stmt(p)
    p.expect_keyword("end")
    Ifz(c, a, b)
  } else if p.eat_keyword("call") {
    let name = p.ident()
    let g = p.group('(')
    let args = []
    if !g.eof() {
      args.push(parse_expr(g))
      while g.eat(',') {
        args.push(parse_expr(g))
      }
    }
    g.finish()
    Call(name, args)
  } else {
    p.expect_keyword("return")
    Return
  }
}

///|
fn parse_code(text : String) -> Code raise @utils.Fault {
  let p = @utils.create(text)
  let functions = []
  while !p.eof() {
    p.expect_keyword("fn")
    let name = p.ident()
    let g = p.group('(')
    let params = []
    if !g.eof() {
      params.push(g.ident())
      while g.eat(',') {
        params.push(g.ident())
      }
    }
    g.finish()
    let g = p.group('{')
    let body = []
    while !g.eof() {
      body.push(parse_stmt(g))
      if !g.eat(';') {
        break
      }
    }
    g.finish()
    functions.push({ name, params, body, })
  }
  { functions, }
}

///|
fn place_text(p : Place) -> String {
  match p {
    Var(n) => n
    Deref(v) => "(" + expr_text(v) + ") \\loc"
  }
}

///|
fn expr_text(v : Expr) -> String {
  match v {
    Number(n) => n.to_string()
    NullPtr => "\\null_ptr"
    BinOp(a, op, b) =>
      "(" +
      expr_text(a) +
      (match op {
        Add => " + "
        Sub => " - "
      }) +
      expr_text(b) +
      ")"
    Load(p) => "\\ld " + place_text(p)
    Addr(p) => place_text(p) + " \\addr"
  }
}

///|
fn stmt_text(s : Stmt) -> String {
  match s {
    Assign(p, v) => "\\assign " + place_text(p) + " := " + expr_text(v)
    Ifz(c, a, b) =>
      "\\ifz " +
      expr_text(c) +
      " \\then " +
      stmt_text(a) +
      " \\else " +
      stmt_text(b) +
      " \\end"
    Call(n, args) =>
      "\\call " + n + "(" + @utils.join(args.map(expr_text), ", ") + ")"
    Return => "\\return"
  }
}
