///|
fn instr_snapshot_json(v : Instr) -> Json {
  match v {
    PushNat(v0) => @utils.tag("PushNat", [@utils.jstr(v0.to_string())])
    PushBool(v0) => @utils.tag("PushBool", [@utils.jbool(v0)])
    PushUnit => @utils.tag("PushUnit", [])
    Print(v0) => @utils.tag("Print", [@utils.jstr(v0.to_string())])
    Acc(v0) => @utils.tag("Acc", [@utils.jstr(v0)])
    MkCls(v0, v1) =>
      @utils.tag("MkCls", [
        @utils.jstr(v0),
        @utils.jarray(v1.map(instr_snapshot_json)),
      ])
    MkRec(v0, v1, v2) =>
      @utils.tag("MkRec", [
        @utils.jstr(v0),
        @utils.jstr(v1),
        @utils.jarray(v2.map(instr_snapshot_json)),
      ])
    BinOp(v0) => @utils.tag("BinOp", [@s.binop_json(v0)])
    UnOp(v0) => @utils.tag("UnOp", [@s.unop_json(v0)])
    Ap => @utils.tag("Ap", [])
    Ret => @utils.tag("Ret", [])
    If(v0, v1) =>
      @utils.tag("If", [
        @utils.jarray(v0.map(instr_snapshot_json)),
        @utils.jarray(v1.map(instr_snapshot_json)),
      ])
  }
}

///|
fn decode_instr(j : Json) -> Instr raise @utils.Fault {
  match @utils.tag_name(j) {
    "PushNat" => PushNat(@utils.json_nat(@utils.tag_arg(j, 0)))
    "PushBool" => PushBool(@utils.boolean(@utils.tag_arg(j, 0)))
    "PushUnit" => PushUnit
    "Print" => Print(@utils.json_nat(@utils.tag_arg(j, 0)))
    "Acc" => Acc(@utils.identifier(@utils.string(@utils.tag_arg(j, 0))))
    "MkCls" =>
      MkCls(
        @utils.identifier(@utils.string(@utils.tag_arg(j, 0))),
        @utils.array(@utils.tag_arg(j, 1)).map(decode_instr),
      )
    "MkRec" =>
      MkRec(
        @utils.identifier(@utils.string(@utils.tag_arg(j, 0))),
        @utils.identifier(@utils.string(@utils.tag_arg(j, 1))),
        @utils.array(@utils.tag_arg(j, 2)).map(decode_instr),
      )
    "BinOp" => BinOp(@s.decode_binop(@utils.tag_arg(j, 0)))
    "UnOp" => UnOp(@s.decode_unop(@utils.tag_arg(j, 0)))
    "Ap" => Ap
    "Ret" => Ret
    "If" =>
      If(
        @utils.array(@utils.tag_arg(j, 0)).map(decode_instr),
        @utils.array(@utils.tag_arg(j, 1)).map(decode_instr),
      )
    _ => @utils.error("Invalid Instr")
  }
}

///|
fn value_json(v : Value) -> Json {
  match v {
    VNat(v0) => @utils.tag("VNat", [@utils.jstr(v0.to_string())])
    VBool(v0) => @utils.tag("VBool", [@utils.jbool(v0)])
    VUnit => @utils.tag("VUnit", [])
    VFun(v0, v1, v2) =>
      @utils.tag("VFun", [
        @utils.jstr(v0),
        @utils.jarray(v1.map(instr_snapshot_json)),
        env_json(v2),
      ])
    VRec(v0, v1, v2, v3) =>
      @utils.tag("VRec", [
        @utils.jstr(v0),
        @utils.jstr(v1),
        @utils.jarray(v2.map(instr_snapshot_json)),
        env_json(v3),
      ])
  }
}

///|
fn decode_value(j : Json) -> Value raise @utils.Fault {
  match @utils.tag_name(j) {
    "VNat" => VNat(@utils.json_nat(@utils.tag_arg(j, 0)))
    "VBool" => VBool(@utils.boolean(@utils.tag_arg(j, 0)))
    "VUnit" => VUnit
    "VFun" =>
      VFun(
        @utils.identifier(@utils.string(@utils.tag_arg(j, 0))),
        @utils.array(@utils.tag_arg(j, 1)).map(decode_instr),
        decode_env(@utils.tag_arg(j, 2)),
      )
    "VRec" =>
      VRec(
        @utils.identifier(@utils.string(@utils.tag_arg(j, 0))),
        @utils.identifier(@utils.string(@utils.tag_arg(j, 1))),
        @utils.array(@utils.tag_arg(j, 2)).map(decode_instr),
        decode_env(@utils.tag_arg(j, 3)),
      )
    _ => @utils.error("Invalid Value")
  }
}

///|
fn env_json(env : Env) -> Json {
  @utils.env_json(env, value_json)
}

///|
fn decode_env(j : Json) -> Env raise @utils.Fault {
  @utils.decode_env(j, decode_value)
}
