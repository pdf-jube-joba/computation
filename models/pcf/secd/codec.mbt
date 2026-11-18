///|
fn instr_snapshot_json(v : Instr) -> Json {
  match v {
    PushNat(v0) => @base.tag("PushNat", [@base.jstr(v0.to_string())])
    PushBool(v0) => @base.tag("PushBool", [@base.jbool(v0)])
    PushUnit => @base.tag("PushUnit", [])
    Print(v0) => @base.tag("Print", [@base.jstr(v0.to_string())])
    Acc(v0) => @base.tag("Acc", [@base.jstr(v0)])
    MkCls(v0, v1) =>
      @base.tag("MkCls", [
        @base.jstr(v0),
        @base.jarray(v1.map(instr_snapshot_json)),
      ])
    MkRec(v0, v1, v2) =>
      @base.tag("MkRec", [
        @base.jstr(v0),
        @base.jstr(v1),
        @base.jarray(v2.map(instr_snapshot_json)),
      ])
    BinOp(v0) => @base.tag("BinOp", [@s.binop_json(v0)])
    UnOp(v0) => @base.tag("UnOp", [@s.unop_json(v0)])
    Ap => @base.tag("Ap", [])
    Ret => @base.tag("Ret", [])
    If(v0, v1) =>
      @base.tag("If", [
        @base.jarray(v0.map(instr_snapshot_json)),
        @base.jarray(v1.map(instr_snapshot_json)),
      ])
  }
}

///|
fn decode_instr(j : Json) -> Instr raise @base.Fault {
  match @base.tag_name(j) {
    "PushNat" => PushNat(@base.json_nat(@base.tag_arg(j, 0)))
    "PushBool" => PushBool(@base.boolean(@base.tag_arg(j, 0)))
    "PushUnit" => PushUnit
    "Print" => Print(@base.json_nat(@base.tag_arg(j, 0)))
    "Acc" => Acc(@base.identifier(@base.string(@base.tag_arg(j, 0))))
    "MkCls" =>
      MkCls(
        @base.identifier(@base.string(@base.tag_arg(j, 0))),
        @base.array(@base.tag_arg(j, 1)).map(decode_instr),
      )
    "MkRec" =>
      MkRec(
        @base.identifier(@base.string(@base.tag_arg(j, 0))),
        @base.identifier(@base.string(@base.tag_arg(j, 1))),
        @base.array(@base.tag_arg(j, 2)).map(decode_instr),
      )
    "BinOp" => BinOp(@s.decode_binop(@base.tag_arg(j, 0)))
    "UnOp" => UnOp(@s.decode_unop(@base.tag_arg(j, 0)))
    "Ap" => Ap
    "Ret" => Ret
    "If" =>
      If(
        @base.array(@base.tag_arg(j, 0)).map(decode_instr),
        @base.array(@base.tag_arg(j, 1)).map(decode_instr),
      )
    _ => @base.error("Invalid Instr")
  }
}

///|
fn value_json(v : Value) -> Json {
  match v {
    VNat(v0) => @base.tag("VNat", [@base.jstr(v0.to_string())])
    VBool(v0) => @base.tag("VBool", [@base.jbool(v0)])
    VUnit => @base.tag("VUnit", [])
    VFun(v0, v1, v2) =>
      @base.tag("VFun", [
        @base.jstr(v0),
        @base.jarray(v1.map(instr_snapshot_json)),
        env_json(v2),
      ])
    VRec(v0, v1, v2, v3) =>
      @base.tag("VRec", [
        @base.jstr(v0),
        @base.jstr(v1),
        @base.jarray(v2.map(instr_snapshot_json)),
        env_json(v3),
      ])
  }
}

///|
fn decode_value(j : Json) -> Value raise @base.Fault {
  match @base.tag_name(j) {
    "VNat" => VNat(@base.json_nat(@base.tag_arg(j, 0)))
    "VBool" => VBool(@base.boolean(@base.tag_arg(j, 0)))
    "VUnit" => VUnit
    "VFun" =>
      VFun(
        @base.identifier(@base.string(@base.tag_arg(j, 0))),
        @base.array(@base.tag_arg(j, 1)).map(decode_instr),
        decode_env(@base.tag_arg(j, 2)),
      )
    "VRec" =>
      VRec(
        @base.identifier(@base.string(@base.tag_arg(j, 0))),
        @base.identifier(@base.string(@base.tag_arg(j, 1))),
        @base.array(@base.tag_arg(j, 2)).map(decode_instr),
        decode_env(@base.tag_arg(j, 3)),
      )
    _ => @base.error("Invalid Value")
  }
}

///|
fn env_json(env : Env) -> Json {
  @data.env_json(env, value_json)
}

///|
fn decode_env(j : Json) -> Env raise @base.Fault {
  @data.decode_env(j, decode_value)
}
