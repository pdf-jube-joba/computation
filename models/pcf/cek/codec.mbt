///|
fn frame_json(v : Frame, statements : Bool) -> Json {
  match v {
    BinOpLeft(v0, v1, v2) =>
      @utils.tag("BinOpLeft", [
        @s.binop_json(v0),
        @s.expr_json(v1),
        @s.env_json(v2),
      ])
    BinOpRight(v0, v1) =>
      @utils.tag("BinOpRight", [@s.value_json(v0), @s.binop_json(v1)])
    Unary(v0) => @utils.tag("Unary", [@s.unop_json(v0)])
    AppFun(v0, v1) => @utils.tag("AppFun", [@s.expr_json(v0), @s.env_json(v1)])
    AppArg(v0) => @utils.tag("AppArg", [@s.value_json(v0)])
    IfExpr(v0, v1, v2) =>
      @utils.tag(if statements { "IfExpr" } else { "IfFrame" }, [
        @s.expr_json(v0),
        @s.expr_json(v1),
        @s.env_json(v2),
      ])
    LetFrame(v0, v1) =>
      @utils.tag("LetFrame", [@utils.jstr(v0), @s.env_json(v1)])
    SeqFrame(v0) => @utils.tag("SeqFrame", [@s.stmt_json(v0)])
    IfStmtFrame(v0, v1) =>
      @utils.tag("IfStmtFrame", [@s.stmt_json(v0), @s.env_json(v1)])
    WhileFrame(v0, v1, v2) =>
      @utils.tag("WhileFrame", [
        @s.expr_json(v0),
        @s.stmt_json(v1),
        @s.env_json(v2),
      ])
    ScopeOut(v0) => @utils.tag("ScopeOut", [@s.expr_json(v0)])
    Init => @utils.tag("Init", [])
  }
}

///|
fn decode_frame(j : Json, _statements : Bool) -> Frame raise @utils.Fault {
  match @utils.tag_name(j) {
    "BinOpLeft" =>
      BinOpLeft(
        @s.decode_binop(@utils.tag_arg(j, 0)),
        @s.decode_expr(@utils.tag_arg(j, 1)),
        @s.decode_env(@utils.tag_arg(j, 2)),
      )
    "BinOpRight" =>
      BinOpRight(
        @s.decode_value(@utils.tag_arg(j, 0)),
        @s.decode_binop(@utils.tag_arg(j, 1)),
      )
    "Unary" => Unary(@s.decode_unop(@utils.tag_arg(j, 0)))
    "AppFun" =>
      AppFun(
        @s.decode_expr(@utils.tag_arg(j, 0)),
        @s.decode_env(@utils.tag_arg(j, 1)),
      )
    "AppArg" => AppArg(@s.decode_value(@utils.tag_arg(j, 0)))
    "IfExpr" | "IfFrame" =>
      IfExpr(
        @s.decode_expr(@utils.tag_arg(j, 0)),
        @s.decode_expr(@utils.tag_arg(j, 1)),
        @s.decode_env(@utils.tag_arg(j, 2)),
      )
    "LetFrame" =>
      LetFrame(
        @utils.identifier(@utils.string(@utils.tag_arg(j, 0))),
        @s.decode_env(@utils.tag_arg(j, 1)),
      )
    "SeqFrame" => SeqFrame(@s.decode_stmt(@utils.tag_arg(j, 0)))
    "IfStmtFrame" =>
      IfStmtFrame(
        @s.decode_stmt(@utils.tag_arg(j, 0)),
        @s.decode_env(@utils.tag_arg(j, 1)),
      )
    "WhileFrame" =>
      WhileFrame(
        @s.decode_expr(@utils.tag_arg(j, 0)),
        @s.decode_stmt(@utils.tag_arg(j, 1)),
        @s.decode_env(@utils.tag_arg(j, 2)),
      )
    "ScopeOut" => ScopeOut(@s.decode_expr(@utils.tag_arg(j, 0)))
    "Init" => Init
    _ => @utils.error("Invalid Frame")
  }
}

///|
fn phase_json(v : Phase, statements : Bool) -> Json {
  match v {
    EvalExpr(v0, v1, v2) =>
      @utils.tag(if statements { "EvalExpr" } else { "Eval" }, [
        @s.expr_json(v0),
        @s.env_json(v1),
        @utils.jarray(v2.map(f => frame_json(f, statements))),
      ])
    Return(v0, v1) =>
      @utils.tag("Return", [
        @s.value_json(v0),
        @utils.jarray(v1.map(f => frame_json(f, statements))),
      ])
    EvalStmt(v0, v1, v2) =>
      @utils.tag("EvalStmt", [
        @s.stmt_json(v0),
        @s.env_json(v1),
        @utils.jarray(v2.map(f => frame_json(f, statements))),
      ])
    Done(v0, v1) =>
      @utils.tag("Done", [
        @s.env_json(v0),
        @utils.jarray(v1.map(f => frame_json(f, statements))),
      ])
  }
}

///|
fn decode_phase(j : Json, statements : Bool) -> Phase raise @utils.Fault {
  match @utils.tag_name(j) {
    "EvalExpr" | "Eval" =>
      EvalExpr(
        @s.decode_expr(@utils.tag_arg(j, 0)),
        @s.decode_env(@utils.tag_arg(j, 1)),
        @utils.array(@utils.tag_arg(j, 2)).map(j => decode_frame(j, statements)),
      )
    "Return" =>
      Return(
        @s.decode_value(@utils.tag_arg(j, 0)),
        @utils.array(@utils.tag_arg(j, 1)).map(j => decode_frame(j, statements)),
      )
    "EvalStmt" =>
      EvalStmt(
        @s.decode_stmt(@utils.tag_arg(j, 0)),
        @s.decode_env(@utils.tag_arg(j, 1)),
        @utils.array(@utils.tag_arg(j, 2)).map(j => decode_frame(j, statements)),
      )
    "Done" =>
      Done(
        @s.decode_env(@utils.tag_arg(j, 0)),
        @utils.array(@utils.tag_arg(j, 1)).map(j => decode_frame(j, statements)),
      )
    _ => @utils.error("Invalid Phase")
  }
}
