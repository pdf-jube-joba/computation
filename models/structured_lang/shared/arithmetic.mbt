///|
pub type Env = Array[(String, @base.Nat)]

///|
pub(all) enum Atom {
  Var(String)
  Imm(@base.Nat)
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
pub fn names(p : @p.Parser) -> Array[String] raise @base.Fault {
  let result = []
  match p.peek() {
    @p.Ident(_) => {
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
pub fn parse_atom(p : @p.Parser) -> Atom raise @base.Fault {
  match p.peek() {
    @p.Number(_) => Imm(p.natural())
    _ => Var(p.ident())
  }
}

///|
pub fn parse_aexp(p : @p.Parser) -> Aexp raise @base.Fault {
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
pub fn parse_bexp(p : @p.Parser) -> Bexp raise @base.Fault {
  let left = parse_atom(p)
  let op = match p.bump() {
    @p.Symbol('<') => Lt
    @p.Symbol('=') => Eq
    @p.Symbol('>') => Gt
    _ => p.error("Expected comparison")
  }
  { left, op, right: parse_atom(p), }
}

///|
pub fn env_get(env : Env, name : String) -> @base.Nat {
  @data.env_get(env, name).unwrap_or(@base.zero())
}

///|
pub fn parse_env(text : String) -> Env raise @base.Fault {
  @data.parse_env(text, @base.nat)
}

///|
pub fn print_env(env : Env, trailing_newline~ : Bool = true) -> String {
  @data.print_env(env, n => n.to_string(), trailing_newline~)
}

///|
pub fn validate_env(
  statics : Array[String],
  env : Env,
  duplicate~ : String,
  extra~ : String,
  missing~ : String,
) -> Unit raise @base.Fault {
  @data.unique(statics, duplicate)
  for pair in env {
    @base.require(statics.contains(pair.0), extra + pair.0)
  }
  for name in statics {
    @base.require(@data.env_get(env, name) is Some(_), missing + name)
  }
}

///|
pub fn eval_atom(
  a : Atom,
  lookup : (String) -> @base.Nat raise @base.Fault,
) -> @base.Nat raise @base.Fault {
  match a {
    Var(n) => lookup(n)
    Imm(n) => n
  }
}

///|
pub fn eval_aexp(
  a : Aexp,
  lookup : (String) -> @base.Nat raise @base.Fault,
) -> @base.Nat raise @base.Fault {
  match a {
    Atom(a) => eval_atom(a, lookup)
    Bin(a, add, b) => {
      let a = eval_atom(a, lookup)
      let b = eval_atom(b, lookup)
      if add {
        a + b
      } else {
        @base.sub(a, b)
      }
    }
  }
}

///|
pub fn eval_bexp(
  b : Bexp,
  lookup : (String) -> @base.Nat raise @base.Fault,
) -> Bool raise @base.Fault {
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
    Var(n) => @base.tag("Var", [@base.jstr(n)])
    Imm(n) => @base.tag("Imm", [@base.jstr(n.to_string())])
  }
}

///|
pub fn decode_atom(j : Json) -> Atom raise @base.Fault {
  match @base.tag_name(j) {
    "Var" => Var(@base.identifier(@base.string(@base.tag_arg(j, 0))))
    "Imm" => Imm(@base.json_nat(@base.tag_arg(j, 0)))
    _ => @base.error("Invalid arithmetic atom")
  }
}

///|
pub fn aexp_json(a : Aexp) -> Json {
  match a {
    Atom(a) => @base.tag("Atom", [atom_json(a)])
    Bin(a, add, b) =>
      @base.tag("Bin", [
        atom_json(a),
        @base.tag(if add { "Add" } else { "Sub" }, []),
        atom_json(b),
      ])
  }
}

///|
pub fn decode_aexp(j : Json) -> Aexp raise @base.Fault {
  fn a(i : Int32) -> Json raise @base.Fault {
    @base.tag_arg(j, i)
  }
  match @base.tag_name(j) {
    "Atom" => Atom(decode_atom(a(0)))
    "Bin" => {
      let op = @base.tag_name(a(1))
      @base.require(op == "Add" || op == "Sub", "Invalid arithmetic operator")
      Bin(decode_atom(a(0)), op == "Add", decode_atom(a(2)))
    }
    _ => @base.error("Invalid arithmetic expression")
  }
}

///|
pub fn bexp_json(b : Bexp) -> Json {
  @base.jarray([
    atom_json(b.left),
    @base.tag(
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
pub fn decode_bexp(j : Json) -> Bexp raise @base.Fault {
  let a = @base.array(j)
  @base.require(a.length() == 3, "Invalid comparison")
  let op = match @base.tag_name(a[1]) {
    "Lt" => Lt
    "Eq" => Eq
    "Gt" => Gt
    _ => @base.error("Invalid comparison operator")
  }
  { left: decode_atom(a[0]), op, right: decode_atom(a[2]), }
}

///|
pub fn env_json(env : Env) -> Json {
  @data.env_json(env, n => @base.jstr(n.to_string()))
}

///|
pub fn decode_env(j : Json) -> Env raise @base.Fault {
  @data.decode_env(j, @base.json_nat)
}

///|
pub fn names_json(names : Array[String]) -> Json {
  @base.jarray(names.map(@base.jstr))
}

///|
pub fn decode_names(j : Json) -> Array[String] raise @base.Fault {
  @base.array(j).map(j => @base.identifier(@base.string(j)))
}
