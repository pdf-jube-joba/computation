///|
fn atom_json(v : Atom) -> Json {
  match v {
    Var(v0) => @utils.tag("Var", [@utils.jstr(v0)])
    Imm(v0) => @utils.tag("Imm", [@utils.jstr(v0.to_string())])
  }
}

///|
fn decode_atom(j : Json) -> Atom raise @utils.Fault {
  match @utils.tag_name(j) {
    "Var" => Var(@utils.identifier(@utils.string(@utils.tag_arg(j, 0))))
    "Imm" => Imm(@utils.json_nat(@utils.tag_arg(j, 0)))
    _ => @utils.error("Invalid coroutine Atom")
  }
}

///|
fn binop_json(v : Binop) -> Json {
  match v {
    Add => @utils.tag("Add", [])
    Sub => @utils.tag("Sub", [])
    Mul => @utils.tag("Mul", [])
  }
}

///|
fn decode_binop(j : Json) -> Binop raise @utils.Fault {
  match @utils.tag_name(j) {
    "Add" => Add
    "Sub" => Sub
    "Mul" => Mul
    _ => @utils.error("Invalid coroutine Binop")
  }
}

///|
fn aexp_json(v : Aexp) -> Json {
  match v {
    Atom(v0) => @utils.tag("Atom", [atom_json(v0)])
    Bin(v0, v1, v2) =>
      @utils.tag("Bin", [aexp_json(v0), binop_json(v1), aexp_json(v2)])
  }
}

///|
fn decode_aexp(j : Json) -> Aexp raise @utils.Fault {
  match @utils.tag_name(j) {
    "Atom" => Atom(decode_atom(@utils.tag_arg(j, 0)))
    "Bin" =>
      Bin(
        decode_aexp(@utils.tag_arg(j, 0)),
        decode_binop(@utils.tag_arg(j, 1)),
        decode_aexp(@utils.tag_arg(j, 2)),
      )
    _ => @utils.error("Invalid coroutine Aexp")
  }
}

///|
fn relop_json(v : Relop) -> Json {
  match v {
    Eq => @utils.tag("Eq", [])
    Ne => @utils.tag("Ne", [])
    Lt => @utils.tag("Lt", [])
  }
}

///|
fn decode_relop(j : Json) -> Relop raise @utils.Fault {
  match @utils.tag_name(j) {
    "Eq" => Eq
    "Ne" => Ne
    "Lt" => Lt
    _ => @utils.error("Invalid coroutine Relop")
  }
}

///|
fn bexp_json(v : Bexp) -> Json {
  match v {
    Not(v0) => @utils.tag("Not", [bexp_json(v0)])
    Rel(v0, v1, v2) =>
      @utils.tag("Rel", [aexp_json(v0), relop_json(v1), aexp_json(v2)])
    And(v0, v1) => @utils.tag("And", [bexp_json(v0), bexp_json(v1)])
    Or(v0, v1) => @utils.tag("Or", [bexp_json(v0), bexp_json(v1)])
    Done(v0) => @utils.tag("Done", [@utils.jstr(v0)])
  }
}

///|
fn decode_bexp(j : Json) -> Bexp raise @utils.Fault {
  match @utils.tag_name(j) {
    "Not" => Not(decode_bexp(@utils.tag_arg(j, 0)))
    "Rel" =>
      Rel(
        decode_aexp(@utils.tag_arg(j, 0)),
        decode_relop(@utils.tag_arg(j, 1)),
        decode_aexp(@utils.tag_arg(j, 2)),
      )
    "And" =>
      And(decode_bexp(@utils.tag_arg(j, 0)), decode_bexp(@utils.tag_arg(j, 1)))
    "Or" =>
      Or(decode_bexp(@utils.tag_arg(j, 0)), decode_bexp(@utils.tag_arg(j, 1)))
    "Done" => Done(@utils.identifier(@utils.string(@utils.tag_arg(j, 0))))
    _ => @utils.error("Invalid coroutine Bexp")
  }
}

///|
fn stmt_json(v : Stmt) -> Json {
  match v {
    Assign(v0, v1) => @utils.tag("Assign", [@utils.jstr(v0), aexp_json(v1)])
    IfGoto(v0, v1) => @utils.tag("IfGoto", [bexp_json(v0), @utils.jint(v1)])
    Goto(v0) => @utils.tag("Goto", [@utils.jint(v0)])
    Call(v0) => @utils.tag("Call", [@utils.jstr(v0)])
    Run(v0, v1) => @utils.tag("Run", [@utils.jstr(v0), @utils.jstr(v1)])
    Yield => @utils.tag("Yield", [])
  }
}

///|
fn decode_stmt(j : Json) -> Stmt raise @utils.Fault {
  match @utils.tag_name(j) {
    "Assign" =>
      Assign(
        @utils.identifier(@utils.string(@utils.tag_arg(j, 0))),
        decode_aexp(@utils.tag_arg(j, 1)),
      )
    "IfGoto" =>
      IfGoto(
        decode_bexp(@utils.tag_arg(j, 0)),
        @utils.int(@utils.tag_arg(j, 1)),
      )
    "Goto" => Goto(@utils.int(@utils.tag_arg(j, 0)))
    "Call" => Call(@utils.identifier(@utils.string(@utils.tag_arg(j, 0))))
    "Run" =>
      Run(
        @utils.identifier(@utils.string(@utils.tag_arg(j, 0))),
        @utils.identifier(@utils.string(@utils.tag_arg(j, 1))),
      )
    "Yield" => Yield
    _ => @utils.error("Invalid coroutine Stmt")
  }
}
