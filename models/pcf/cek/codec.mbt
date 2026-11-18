///|
fn frame_json(v : Frame, statements : Bool) -> Json {
  match v {
    BinOpLeft(v0, v1, v2) =>
      @base.tag("BinOpLeft", [
        @s.binop_json(v0),
        @s.expr_json(v1),
        @s.env_json(v2),
      ])
    BinOpRight(v0, v1) =>
      @base.tag("BinOpRight", [@s.value_json(v0), @s.binop_json(v1)])
    Unary(v0) => @base.tag("Unary", [@s.unop_json(v0)])
    AppFun(v0, v1) => @base.tag("AppFun", [@s.expr_json(v0), @s.env_json(v1)])
    AppArg(v0) => @base.tag("AppArg", [@s.value_json(v0)])
    IfExpr(v0, v1, v2) =>
      @base.tag(if statements { "IfExpr" } else { "IfFrame" }, [
        @s.expr_json(v0),
        @s.expr_json(v1),
        @s.env_json(v2),
      ])
    LetFrame(v0, v1) => @base.tag("LetFrame", [@base.jstr(v0), @s.env_json(v1)])
    SeqFrame(v0) => @base.tag("SeqFrame", [@s.stmt_json(v0)])
    IfStmtFrame(v0, v1) =>
      @base.tag("IfStmtFrame", [@s.stmt_json(v0), @s.env_json(v1)])
    WhileFrame(v0, v1, v2) =>
      @base.tag("WhileFrame", [
        @s.expr_json(v0),
        @s.stmt_json(v1),
        @s.env_json(v2),
      ])
    ScopeOut(v0) => @base.tag("ScopeOut", [@s.expr_json(v0)])
    Init => @base.tag("Init", [])
  }
}

///|
fn decode_frame(j : Json, _statements : Bool) -> Frame raise @base.Fault {
  match @base.tag_name(j) {
    "BinOpLeft" =>
      BinOpLeft(
        @s.decode_binop(@base.tag_arg(j, 0)),
        @s.decode_expr(@base.tag_arg(j, 1)),
        @s.decode_env(@base.tag_arg(j, 2)),
      )
    "BinOpRight" =>
      BinOpRight(
        @s.decode_value(@base.tag_arg(j, 0)),
        @s.decode_binop(@base.tag_arg(j, 1)),
      )
    "Unary" => Unary(@s.decode_unop(@base.tag_arg(j, 0)))
    "AppFun" =>
      AppFun(
        @s.decode_expr(@base.tag_arg(j, 0)),
        @s.decode_env(@base.tag_arg(j, 1)),
      )
    "AppArg" => AppArg(@s.decode_value(@base.tag_arg(j, 0)))
    "IfExpr" | "IfFrame" =>
      IfExpr(
        @s.decode_expr(@base.tag_arg(j, 0)),
        @s.decode_expr(@base.tag_arg(j, 1)),
        @s.decode_env(@base.tag_arg(j, 2)),
      )
    "LetFrame" =>
      LetFrame(
        @base.identifier(@base.string(@base.tag_arg(j, 0))),
        @s.decode_env(@base.tag_arg(j, 1)),
      )
    "SeqFrame" => SeqFrame(@s.decode_stmt(@base.tag_arg(j, 0)))
    "IfStmtFrame" =>
      IfStmtFrame(
        @s.decode_stmt(@base.tag_arg(j, 0)),
        @s.decode_env(@base.tag_arg(j, 1)),
      )
    "WhileFrame" =>
      WhileFrame(
        @s.decode_expr(@base.tag_arg(j, 0)),
        @s.decode_stmt(@base.tag_arg(j, 1)),
        @s.decode_env(@base.tag_arg(j, 2)),
      )
    "ScopeOut" => ScopeOut(@s.decode_expr(@base.tag_arg(j, 0)))
    "Init" => Init
    _ => @base.error("Invalid Frame")
  }
}

///|
fn phase_json(v : Phase, statements : Bool) -> Json {
  match v {
    EvalExpr(v0, v1, v2) =>
      @base.tag(if statements { "EvalExpr" } else { "Eval" }, [
        @s.expr_json(v0),
        @s.env_json(v1),
        @base.jarray(v2.map(f => frame_json(f, statements))),
      ])
    Return(v0, v1) =>
      @base.tag("Return", [
        @s.value_json(v0),
        @base.jarray(v1.map(f => frame_json(f, statements))),
      ])
    EvalStmt(v0, v1, v2) =>
      @base.tag("EvalStmt", [
        @s.stmt_json(v0),
        @s.env_json(v1),
        @base.jarray(v2.map(f => frame_json(f, statements))),
      ])
    Done(v0, v1) =>
      @base.tag("Done", [
        @s.env_json(v0),
        @base.jarray(v1.map(f => frame_json(f, statements))),
      ])
  }
}

///|
fn decode_phase(j : Json, statements : Bool) -> Phase raise @base.Fault {
  match @base.tag_name(j) {
    "EvalExpr" | "Eval" =>
      EvalExpr(
        @s.decode_expr(@base.tag_arg(j, 0)),
        @s.decode_env(@base.tag_arg(j, 1)),
        @base.array(@base.tag_arg(j, 2)).map(j => decode_frame(j, statements)),
      )
    "Return" =>
      Return(
        @s.decode_value(@base.tag_arg(j, 0)),
        @base.array(@base.tag_arg(j, 1)).map(j => decode_frame(j, statements)),
      )
    "EvalStmt" =>
      EvalStmt(
        @s.decode_stmt(@base.tag_arg(j, 0)),
        @s.decode_env(@base.tag_arg(j, 1)),
        @base.array(@base.tag_arg(j, 2)).map(j => decode_frame(j, statements)),
      )
    "Done" =>
      Done(
        @s.decode_env(@base.tag_arg(j, 0)),
        @base.array(@base.tag_arg(j, 1)).map(j => decode_frame(j, statements)),
      )
    _ => @base.error("Invalid Phase")
  }
}
