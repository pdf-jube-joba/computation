///|
pub type Env = Array[(String, @utils.Nat)]

///|
pub(all) enum Atom {
  Var(String)
  Imm(@utils.Nat)
}

///|
pub(all) enum Aexp {
  Atom(Atom)
  Bin(Atom, Bool, Atom)
}

///|
pub(all) enum Rel {
  Lt
  Eq
  Gt
}

///|
pub(all) struct Bexp {
  left : Atom
  op : Rel
  right : Atom
}

///|
pub fn names(p : @utils.Parser) -> Array[String] raise @utils.Fault {
  let result = []
  match p.peek() {
    @utils.Ident(_) => {
      result.push(p.ident())
      while p.eat(',') {
        result.push(p.ident())
      }
    }
    _ => ()
  }
  result
}

///|
pub fn parse_atom(p : @utils.Parser) -> Atom raise @utils.Fault {
  match p.peek() {
    @utils.Number(_) => Imm(p.natural())
    _ => Var(p.ident())
  }
}

///|
pub fn parse_aexp(p : @utils.Parser) -> Aexp raise @utils.Fault {
  let a = parse_atom(p)
  if p.eat('+') {
    Bin(a, true, parse_atom(p))
  } else if p.eat('-') {
    Bin(a, false, parse_atom(p))
  } else {
    Atom(a)
  }
}

///|
pub fn parse_bexp(p : @utils.Parser) -> Bexp raise @utils.Fault {
  let left = parse_atom(p)
  let op = match p.bump() {
    @utils.Symbol('<') => Lt
    @utils.Symbol('=') => Eq
    @utils.Symbol('>') => Gt
    _ => p.error("Expected comparison")
  }
  { left, op, right: parse_atom(p), }
}

///|
pub fn env_get(env : Env, name : String) -> @utils.Nat {
  @utils.env_get(env, name).unwrap_or(@utils.zero())
}

///|
pub fn parse_env(text : String) -> Env raise @utils.Fault {
  @utils.parse_env(text, @utils.nat)
}

///|
pub fn print_env(env : Env, trailing_newline~ : Bool = true) -> String {
  @utils.print_env(env, n => n.to_string(), trailing_newline~)
}

///|
pub fn validate_env(
  statics : Array[String],
  env : Env,
  duplicate~ : String,
  extra~ : String,
  missing~ : String,
) -> Unit raise @utils.Fault {
  @utils.unique(statics, duplicate)
  for pair in env {
    @utils.require(statics.contains(pair.0), extra + pair.0)
  }
  for name in statics {
    @utils.require(@utils.env_get(env, name) is Some(_), missing + name)
  }
}

///|
pub fn eval_atom(
  a : Atom,
  lookup : (String) -> @utils.Nat raise @utils.Fault,
) -> @utils.Nat raise @utils.Fault {
  match a {
    Var(n) => lookup(n)
    Imm(n) => n
  }
}

///|
pub fn eval_aexp(
  a : Aexp,
  lookup : (String) -> @utils.Nat raise @utils.Fault,
) -> @utils.Nat raise @utils.Fault {
  match a {
    Atom(a) => eval_atom(a, lookup)
    Bin(a, add, b) => {
      let a = eval_atom(a, lookup)
      let b = eval_atom(b, lookup)
      if add {
        a + b
      } else {
        @utils.sub(a, b)
      }
    }
  }
}

///|
pub fn eval_bexp(
  b : Bexp,
  lookup : (String) -> @utils.Nat raise @utils.Fault,
) -> Bool raise @utils.Fault {
  let a = eval_atom(b.left, lookup)
  let c = eval_atom(b.right, lookup)
  match b.op {
    Lt => a < c
    Eq => a == c
    Gt => a > c
  }
}

///|
pub fn atom_text(a : Atom) -> String {
  match a {
    Var(n) => n
    Imm(n) => n.to_string()
  }
}

///|
pub fn aexp_text(a : Aexp) -> String {
  match a {
    Atom(a) => atom_text(a)
    Bin(a, add, b) =>
      atom_text(a) + (if add { " + " } else { " - " }) + atom_text(b)
  }
}

///|
pub fn bexp_text(b : Bexp) -> String {
  atom_text(b.left) +
  " " +
  (match b.op {
    Lt => "<"
    Eq => "="
    Gt => ">"
  }) +
  " " +
  atom_text(b.right)
}

///|
pub fn atom_json(a : Atom) -> Json {
  match a {
    Var(n) => @utils.tag("Var", [@utils.jstr(n)])
    Imm(n) => @utils.tag("Imm", [@utils.jstr(n.to_string())])
  }
}

///|
pub fn decode_atom(j : Json) -> Atom raise @utils.Fault {
  match @utils.tag_name(j) {
    "Var" => Var(@utils.identifier(@utils.string(@utils.tag_arg(j, 0))))
    "Imm" => Imm(@utils.json_nat(@utils.tag_arg(j, 0)))
    _ => @utils.error("Invalid arithmetic atom")
  }
}

///|
pub fn aexp_json(a : Aexp) -> Json {
  match a {
    Atom(a) => @utils.tag("Atom", [atom_json(a)])
    Bin(a, add, b) =>
      @utils.tag("Bin", [
        atom_json(a),
        @utils.tag(if add { "Add" } else { "Sub" }, []),
        atom_json(b),
      ])
  }
}

///|
pub fn decode_aexp(j : Json) -> Aexp raise @utils.Fault {
  fn a(i : Int32) -> Json raise @utils.Fault {
    @utils.tag_arg(j, i)
  }
  match @utils.tag_name(j) {
    "Atom" => Atom(decode_atom(a(0)))
    "Bin" => {
      let op = @utils.tag_name(a(1))
      @utils.require(op == "Add" || op == "Sub", "Invalid arithmetic operator")
      Bin(decode_atom(a(0)), op == "Add", decode_atom(a(2)))
    }
    _ => @utils.error("Invalid arithmetic expression")
  }
}

///|
pub fn bexp_json(b : Bexp) -> Json {
  @utils.jarray([
    atom_json(b.left),
    @utils.tag(
      match b.op {
        Lt => "Lt"
        Eq => "Eq"
        Gt => "Gt"
      },
      [],
    ),
    atom_json(b.right),
  ])
}

///|
pub fn decode_bexp(j : Json) -> Bexp raise @utils.Fault {
  let a = @utils.array(j)
  @utils.require(a.length() == 3, "Invalid comparison")
  let op = match @utils.tag_name(a[1]) {
    "Lt" => Lt
    "Eq" => Eq
    "Gt" => Gt
    _ => @utils.error("Invalid comparison operator")
  }
  { left: decode_atom(a[0]), op, right: decode_atom(a[2]), }
}

///|
pub fn env_json(env : Env) -> Json {
  @utils.env_json(env, n => @utils.jstr(n.to_string()))
}

///|
pub fn decode_env(j : Json) -> Env raise @utils.Fault {
  @utils.decode_env(j, @utils.json_nat)
}

///|
pub fn names_json(names : Array[String]) -> Json {
  @utils.jarray(names.map(@utils.jstr))
}

///|
pub fn decode_names(j : Json) -> Array[String] raise @utils.Fault {
  @utils.array(j).map(j => @utils.identifier(@utils.string(j)))
}
