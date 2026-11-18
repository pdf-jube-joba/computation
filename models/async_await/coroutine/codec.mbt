///|
fn atom_json(v : Atom) -> Json {
  match v {
    Var(v0) => @base.tag("Var", [@base.jstr(v0)])
    Imm(v0) => @base.tag("Imm", [@base.jstr(v0.to_string())])
  }
}

///|
fn decode_atom(j : Json) -> Atom raise @base.Fault {
  match @base.tag_name(j) {
    "Var" => Var(@base.identifier(@base.string(@base.tag_arg(j, 0))))
    "Imm" => Imm(@base.json_nat(@base.tag_arg(j, 0)))
    _ => @base.error("Invalid coroutine Atom")
  }
}

///|
fn binop_json(v : Binop) -> Json {
  match v {
    Add => @base.tag("Add", [])
    Sub => @base.tag("Sub", [])
    Mul => @base.tag("Mul", [])
  }
}

///|
fn decode_binop(j : Json) -> Binop raise @base.Fault {
  match @base.tag_name(j) {
    "Add" => Add
    "Sub" => Sub
    "Mul" => Mul
    _ => @base.error("Invalid coroutine Binop")
  }
}

///|
fn aexp_json(v : Aexp) -> Json {
  match v {
    Atom(v0) => @base.tag("Atom", [atom_json(v0)])
    Bin(v0, v1, v2) =>
      @base.tag("Bin", [aexp_json(v0), binop_json(v1), aexp_json(v2)])
  }
}

///|
fn decode_aexp(j : Json) -> Aexp raise @base.Fault {
  match @base.tag_name(j) {
    "Atom" => Atom(decode_atom(@base.tag_arg(j, 0)))
    "Bin" =>
      Bin(
        decode_aexp(@base.tag_arg(j, 0)),
        decode_binop(@base.tag_arg(j, 1)),
        decode_aexp(@base.tag_arg(j, 2)),
      )
    _ => @base.error("Invalid coroutine Aexp")
  }
}

///|
fn relop_json(v : Relop) -> Json {
  match v {
    Eq => @base.tag("Eq", [])
    Ne => @base.tag("Ne", [])
    Lt => @base.tag("Lt", [])
  }
}

///|
fn decode_relop(j : Json) -> Relop raise @base.Fault {
  match @base.tag_name(j) {
    "Eq" => Eq
    "Ne" => Ne
    "Lt" => Lt
    _ => @base.error("Invalid coroutine Relop")
  }
}

///|
fn bexp_json(v : Bexp) -> Json {
  match v {
    Not(v0) => @base.tag("Not", [bexp_json(v0)])
    Rel(v0, v1, v2) =>
      @base.tag("Rel", [aexp_json(v0), relop_json(v1), aexp_json(v2)])
    And(v0, v1) => @base.tag("And", [bexp_json(v0), bexp_json(v1)])
    Or(v0, v1) => @base.tag("Or", [bexp_json(v0), bexp_json(v1)])
    Done(v0) => @base.tag("Done", [@base.jstr(v0)])
  }
}

///|
fn decode_bexp(j : Json) -> Bexp raise @base.Fault {
  match @base.tag_name(j) {
    "Not" => Not(decode_bexp(@base.tag_arg(j, 0)))
    "Rel" =>
      Rel(
        decode_aexp(@base.tag_arg(j, 0)),
        decode_relop(@base.tag_arg(j, 1)),
        decode_aexp(@base.tag_arg(j, 2)),
      )
    "And" =>
      And(decode_bexp(@base.tag_arg(j, 0)), decode_bexp(@base.tag_arg(j, 1)))
    "Or" =>
      Or(decode_bexp(@base.tag_arg(j, 0)), decode_bexp(@base.tag_arg(j, 1)))
    "Done" => Done(@base.identifier(@base.string(@base.tag_arg(j, 0))))
    _ => @base.error("Invalid coroutine Bexp")
  }
}

///|
fn stmt_json(v : Stmt) -> Json {
  match v {
    Assign(v0, v1) => @base.tag("Assign", [@base.jstr(v0), aexp_json(v1)])
    IfGoto(v0, v1) => @base.tag("IfGoto", [bexp_json(v0), @base.jint(v1)])
    Goto(v0) => @base.tag("Goto", [@base.jint(v0)])
    Call(v0) => @base.tag("Call", [@base.jstr(v0)])
    Run(v0, v1) => @base.tag("Run", [@base.jstr(v0), @base.jstr(v1)])
    Yield => @base.tag("Yield", [])
  }
}

///|
fn decode_stmt(j : Json) -> Stmt raise @base.Fault {
  match @base.tag_name(j) {
    "Assign" =>
      Assign(
        @base.identifier(@base.string(@base.tag_arg(j, 0))),
        decode_aexp(@base.tag_arg(j, 1)),
      )
    "IfGoto" =>
      IfGoto(decode_bexp(@base.tag_arg(j, 0)), @base.int(@base.tag_arg(j, 1)))
    "Goto" => Goto(@base.int(@base.tag_arg(j, 0)))
    "Call" => Call(@base.identifier(@base.string(@base.tag_arg(j, 0))))
    "Run" =>
      Run(
        @base.identifier(@base.string(@base.tag_arg(j, 0))),
        @base.identifier(@base.string(@base.tag_arg(j, 1))),
      )
    "Yield" => Yield
    _ => @base.error("Invalid coroutine Stmt")
  }
}
