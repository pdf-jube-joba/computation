///|
pub fn binop_json(value : Binop) -> Json {
  match value {
    Add => @utils.tag("Add", [])
    Sub => @utils.tag("Sub", [])
    And => @utils.tag("And", [])
  }
}

///|
pub fn decode_binop(j : Json) -> Binop raise @utils.Fault {
  match @utils.tag_name(j) {
    "Add" => Add
    "Sub" => Sub
    "And" => And
    _ => @utils.error("Invalid Binop snapshot")
  }
}

///|
pub fn unop_json(value : Unop) -> Json {
  match value {
    Inc => @utils.tag("Inc", [])
    Dec => @utils.tag("Dec", [])
    Not => @utils.tag("Not", [])
  }
}

///|
pub fn decode_unop(j : Json) -> Unop raise @utils.Fault {
  match @utils.tag_name(j) {
    "Inc" => Inc
    "Dec" => Dec
    "Not" => Not
    _ => @utils.error("Invalid Unop snapshot")
  }
}

///|
pub fn expr_json(value : Expr) -> Json {
  match value {
    Nat(v0) => @utils.tag("Nat", [@utils.jstr(v0.to_string())])
    Bool(v0) => @utils.tag("Bool", [@utils.jbool(v0)])
    Unit => @utils.tag("Unit", [])
    Print(v0) => @utils.tag("Print", [@utils.jstr(v0.to_string())])
    Var(v0) => @utils.tag("Var", [@utils.jstr(v0)])
    BinOp(v0, v1, v2) =>
      @utils.tag("BinOp", [expr_json(v0), binop_json(v1), expr_json(v2)])
    UnOp(v0, v1) => @utils.tag("UnOp", [unop_json(v0), expr_json(v1)])
    Fun(v0, v1) => @utils.tag("Fun", [@utils.jstr(v0), expr_json(v1)])
    Rec(v0, v1, v2) =>
      @utils.tag("Rec", [@utils.jstr(v0), @utils.jstr(v1), expr_json(v2)])
    App(v0, v1) => @utils.tag("App", [expr_json(v0), expr_json(v1)])
    If(v0, v1, v2) =>
      @utils.tag("If", [expr_json(v0), expr_json(v1), expr_json(v2)])
    Block(v0, v1) => @utils.tag("Block", [stmt_json(v0), expr_json(v1)])
  }
}

///|
pub fn decode_expr(j : Json) -> Expr raise @utils.Fault {
  match @utils.tag_name(j) {
    "Nat" => Nat(@utils.json_nat(@utils.tag_arg(j, 0)))
    "Bool" => Bool(@utils.boolean(@utils.tag_arg(j, 0)))
    "Unit" => Unit
    "Print" => Print(@utils.json_nat(@utils.tag_arg(j, 0)))
    "Var" => Var(@utils.identifier(@utils.string(@utils.tag_arg(j, 0))))
    "BinOp" =>
      BinOp(
        decode_expr(@utils.tag_arg(j, 0)),
        decode_binop(@utils.tag_arg(j, 1)),
        decode_expr(@utils.tag_arg(j, 2)),
      )
    "UnOp" =>
      UnOp(decode_unop(@utils.tag_arg(j, 0)), decode_expr(@utils.tag_arg(j, 1)))
    "Fun" =>
      Fun(
        @utils.identifier(@utils.string(@utils.tag_arg(j, 0))),
        decode_expr(@utils.tag_arg(j, 1)),
      )
    "Rec" =>
      Rec(
        @utils.identifier(@utils.string(@utils.tag_arg(j, 0))),
        @utils.identifier(@utils.string(@utils.tag_arg(j, 1))),
        decode_expr(@utils.tag_arg(j, 2)),
      )
    "App" =>
      App(decode_expr(@utils.tag_arg(j, 0)), decode_expr(@utils.tag_arg(j, 1)))
    "If" =>
      If(
        decode_expr(@utils.tag_arg(j, 0)),
        decode_expr(@utils.tag_arg(j, 1)),
        decode_expr(@utils.tag_arg(j, 2)),
      )
    "Block" =>
      Block(
        decode_stmt(@utils.tag_arg(j, 0)),
        decode_expr(@utils.tag_arg(j, 1)),
      )
    _ => @utils.error("Invalid Expr snapshot")
  }
}

///|
pub fn stmt_json(value : Stmt) -> Json {
  match value {
    Skip => @utils.tag("Skip", [])
    PrintStmt(v0) => @utils.tag("PrintStmt", [@utils.jstr(v0.to_string())])
    Seq(v0, v1) => @utils.tag("Seq", [stmt_json(v0), stmt_json(v1)])
    Let(v0, v1) => @utils.tag("Let", [@utils.jstr(v0), expr_json(v1)])
    IfStmt(v0, v1) => @utils.tag("IfStmt", [expr_json(v0), stmt_json(v1)])
    While(v0, v1) => @utils.tag("While", [expr_json(v0), stmt_json(v1)])
  }
}

///|
pub fn decode_stmt(j : Json) -> Stmt raise @utils.Fault {
  match @utils.tag_name(j) {
    "Skip" => Skip
    "PrintStmt" => PrintStmt(@utils.json_nat(@utils.tag_arg(j, 0)))
    "Seq" =>
      Seq(decode_stmt(@utils.tag_arg(j, 0)), decode_stmt(@utils.tag_arg(j, 1)))
    "Let" =>
      Let(
        @utils.identifier(@utils.string(@utils.tag_arg(j, 0))),
        decode_expr(@utils.tag_arg(j, 1)),
      )
    "IfStmt" =>
      IfStmt(
        decode_expr(@utils.tag_arg(j, 0)),
        decode_stmt(@utils.tag_arg(j, 1)),
      )
    "While" =>
      While(
        decode_expr(@utils.tag_arg(j, 0)),
        decode_stmt(@utils.tag_arg(j, 1)),
      )
    _ => @utils.error("Invalid Stmt snapshot")
  }
}

///|
pub fn value_json(value : Value) -> Json {
  match value {
    VNat(v0) => @utils.tag("VNat", [@utils.jstr(v0.to_string())])
    VBool(v0) => @utils.tag("VBool", [@utils.jbool(v0)])
    VUnit => @utils.tag("VUnit", [])
    VFun(v0, v1, v2) =>
      @utils.tag("VFun", [@utils.jstr(v0), expr_json(v1), env_json(v2)])
    VRec(v0, v1, v2, v3) =>
      @utils.tag("VRec", [
        @utils.jstr(v0),
        @utils.jstr(v1),
        expr_json(v2),
        env_json(v3),
      ])
  }
}

///|
pub fn decode_value(j : Json) -> Value raise @utils.Fault {
  match @utils.tag_name(j) {
    "VNat" => VNat(@utils.json_nat(@utils.tag_arg(j, 0)))
    "VBool" => VBool(@utils.boolean(@utils.tag_arg(j, 0)))
    "VUnit" => VUnit
    "VFun" =>
      VFun(
        @utils.identifier(@utils.string(@utils.tag_arg(j, 0))),
        decode_expr(@utils.tag_arg(j, 1)),
        decode_env(@utils.tag_arg(j, 2)),
      )
    "VRec" =>
      VRec(
        @utils.identifier(@utils.string(@utils.tag_arg(j, 0))),
        @utils.identifier(@utils.string(@utils.tag_arg(j, 1))),
        decode_expr(@utils.tag_arg(j, 2)),
        decode_env(@utils.tag_arg(j, 3)),
      )
    _ => @utils.error("Invalid Value snapshot")
  }
}

///|
pub fn env_json(env : Env) -> Json {
  @utils.env_json(env, value_json)
}

///|
pub fn decode_env(j : Json) -> Env raise @utils.Fault {
  @utils.decode_env(j, decode_value)
}
