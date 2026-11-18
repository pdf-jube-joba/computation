///|
pub fn binop_json(value : Binop) -> Json {
  match value {
    Add => @base.tag("Add", [])
    Sub => @base.tag("Sub", [])
    And => @base.tag("And", [])
  }
}

///|
pub fn decode_binop(j : Json) -> Binop raise @base.Fault {
  match @base.tag_name(j) {
    "Add" => Add
    "Sub" => Sub
    "And" => And
    _ => @base.error("Invalid Binop snapshot")
  }
}

///|
pub fn unop_json(value : Unop) -> Json {
  match value {
    Inc => @base.tag("Inc", [])
    Dec => @base.tag("Dec", [])
    Not => @base.tag("Not", [])
  }
}

///|
pub fn decode_unop(j : Json) -> Unop raise @base.Fault {
  match @base.tag_name(j) {
    "Inc" => Inc
    "Dec" => Dec
    "Not" => Not
    _ => @base.error("Invalid Unop snapshot")
  }
}

///|
pub fn expr_json(value : Expr) -> Json {
  match value {
    Nat(v0) => @base.tag("Nat", [@base.jstr(v0.to_string())])
    Bool(v0) => @base.tag("Bool", [@base.jbool(v0)])
    Unit => @base.tag("Unit", [])
    Print(v0) => @base.tag("Print", [@base.jstr(v0.to_string())])
    Var(v0) => @base.tag("Var", [@base.jstr(v0)])
    BinOp(v0, v1, v2) =>
      @base.tag("BinOp", [expr_json(v0), binop_json(v1), expr_json(v2)])
    UnOp(v0, v1) => @base.tag("UnOp", [unop_json(v0), expr_json(v1)])
    Fun(v0, v1) => @base.tag("Fun", [@base.jstr(v0), expr_json(v1)])
    Rec(v0, v1, v2) =>
      @base.tag("Rec", [@base.jstr(v0), @base.jstr(v1), expr_json(v2)])
    App(v0, v1) => @base.tag("App", [expr_json(v0), expr_json(v1)])
    If(v0, v1, v2) =>
      @base.tag("If", [expr_json(v0), expr_json(v1), expr_json(v2)])
    Block(v0, v1) => @base.tag("Block", [stmt_json(v0), expr_json(v1)])
  }
}

///|
pub fn decode_expr(j : Json) -> Expr raise @base.Fault {
  match @base.tag_name(j) {
    "Nat" => Nat(@base.json_nat(@base.tag_arg(j, 0)))
    "Bool" => Bool(@base.boolean(@base.tag_arg(j, 0)))
    "Unit" => Unit
    "Print" => Print(@base.json_nat(@base.tag_arg(j, 0)))
    "Var" => Var(@base.identifier(@base.string(@base.tag_arg(j, 0))))
    "BinOp" =>
      BinOp(
        decode_expr(@base.tag_arg(j, 0)),
        decode_binop(@base.tag_arg(j, 1)),
        decode_expr(@base.tag_arg(j, 2)),
      )
    "UnOp" =>
      UnOp(decode_unop(@base.tag_arg(j, 0)), decode_expr(@base.tag_arg(j, 1)))
    "Fun" =>
      Fun(
        @base.identifier(@base.string(@base.tag_arg(j, 0))),
        decode_expr(@base.tag_arg(j, 1)),
      )
    "Rec" =>
      Rec(
        @base.identifier(@base.string(@base.tag_arg(j, 0))),
        @base.identifier(@base.string(@base.tag_arg(j, 1))),
        decode_expr(@base.tag_arg(j, 2)),
      )
    "App" =>
      App(decode_expr(@base.tag_arg(j, 0)), decode_expr(@base.tag_arg(j, 1)))
    "If" =>
      If(
        decode_expr(@base.tag_arg(j, 0)),
        decode_expr(@base.tag_arg(j, 1)),
        decode_expr(@base.tag_arg(j, 2)),
      )
    "Block" =>
      Block(decode_stmt(@base.tag_arg(j, 0)), decode_expr(@base.tag_arg(j, 1)))
    _ => @base.error("Invalid Expr snapshot")
  }
}

///|
pub fn stmt_json(value : Stmt) -> Json {
  match value {
    Skip => @base.tag("Skip", [])
    PrintStmt(v0) => @base.tag("PrintStmt", [@base.jstr(v0.to_string())])
    Seq(v0, v1) => @base.tag("Seq", [stmt_json(v0), stmt_json(v1)])
    Let(v0, v1) => @base.tag("Let", [@base.jstr(v0), expr_json(v1)])
    IfStmt(v0, v1) => @base.tag("IfStmt", [expr_json(v0), stmt_json(v1)])
    While(v0, v1) => @base.tag("While", [expr_json(v0), stmt_json(v1)])
  }
}

///|
pub fn decode_stmt(j : Json) -> Stmt raise @base.Fault {
  match @base.tag_name(j) {
    "Skip" => Skip
    "PrintStmt" => PrintStmt(@base.json_nat(@base.tag_arg(j, 0)))
    "Seq" =>
      Seq(decode_stmt(@base.tag_arg(j, 0)), decode_stmt(@base.tag_arg(j, 1)))
    "Let" =>
      Let(
        @base.identifier(@base.string(@base.tag_arg(j, 0))),
        decode_expr(@base.tag_arg(j, 1)),
      )
    "IfStmt" =>
      IfStmt(decode_expr(@base.tag_arg(j, 0)), decode_stmt(@base.tag_arg(j, 1)))
    "While" =>
      While(decode_expr(@base.tag_arg(j, 0)), decode_stmt(@base.tag_arg(j, 1)))
    _ => @base.error("Invalid Stmt snapshot")
  }
}

///|
pub fn value_json(value : Value) -> Json {
  match value {
    VNat(v0) => @base.tag("VNat", [@base.jstr(v0.to_string())])
    VBool(v0) => @base.tag("VBool", [@base.jbool(v0)])
    VUnit => @base.tag("VUnit", [])
    VFun(v0, v1, v2) =>
      @base.tag("VFun", [@base.jstr(v0), expr_json(v1), env_json(v2)])
    VRec(v0, v1, v2, v3) =>
      @base.tag("VRec", [
        @base.jstr(v0),
        @base.jstr(v1),
        expr_json(v2),
        env_json(v3),
      ])
  }
}

///|
pub fn decode_value(j : Json) -> Value raise @base.Fault {
  match @base.tag_name(j) {
    "VNat" => VNat(@base.json_nat(@base.tag_arg(j, 0)))
    "VBool" => VBool(@base.boolean(@base.tag_arg(j, 0)))
    "VUnit" => VUnit
    "VFun" =>
      VFun(
        @base.identifier(@base.string(@base.tag_arg(j, 0))),
        decode_expr(@base.tag_arg(j, 1)),
        decode_env(@base.tag_arg(j, 2)),
      )
    "VRec" =>
      VRec(
        @base.identifier(@base.string(@base.tag_arg(j, 0))),
        @base.identifier(@base.string(@base.tag_arg(j, 1))),
        decode_expr(@base.tag_arg(j, 2)),
        decode_env(@base.tag_arg(j, 3)),
      )
    _ => @base.error("Invalid Value snapshot")
  }
}

///|
pub fn env_json(env : Env) -> Json {
  @data.env_json(env, value_json)
}

///|
pub fn decode_env(j : Json) -> Env raise @base.Fault {
  @data.decode_env(j, decode_value)
}
