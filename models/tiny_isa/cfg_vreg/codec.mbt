///|
fn addr_json(v : Addr) -> Json {
  match v {
    VReg(v0) => @base.tag("VReg", [@base.jint(v0)])
    Label(v0) => @base.tag("Label", [@base.jstr(v0)])
    Imm(v0) => @base.tag("Imm", [@base.jstr(v0.to_string())])
  }
}

///|
fn decode_addr(j : Json) -> Addr raise @base.Fault {
  match @base.tag_name(j) {
    "VReg" => VReg(@base.int(@base.tag_arg(j, 0)))
    "Label" => Label(@base.identifier(@base.string(@base.tag_arg(j, 0))))
    "Imm" => Imm(@base.json_nat(@base.tag_arg(j, 0)))
    _ => @base.error("Invalid CFG snapshot")
  }
}

///|
fn value_json(v : Value) -> Json {
  match v {
    VRegValue(v0) => @base.tag("VRegValue", [@base.jint(v0)])
    ImmValue(v0) => @base.tag("ImmValue", [@base.jstr(v0.to_string())])
    LabelValue(v0) => @base.tag("LabelValue", [@base.jstr(v0)])
    Deref(v0) => @base.tag("Deref", [addr_json(v0)])
  }
}

///|
fn decode_value(j : Json) -> Value raise @base.Fault {
  match @base.tag_name(j) {
    "VRegValue" => VRegValue(@base.int(@base.tag_arg(j, 0)))
    "ImmValue" => ImmValue(@base.json_nat(@base.tag_arg(j, 0)))
    "LabelValue" =>
      LabelValue(@base.identifier(@base.string(@base.tag_arg(j, 0))))
    "Deref" => Deref(decode_addr(@base.tag_arg(j, 0)))
    _ => @base.error("Invalid CFG snapshot")
  }
}

///|
fn binop_json(v : Binop) -> Json {
  match v {
    Add => @base.tag("Add", [])
    Sub => @base.tag("Sub", [])
  }
}

///|
fn decode_binop(j : Json) -> Binop raise @base.Fault {
  match @base.tag_name(j) {
    "Add" => Add
    "Sub" => Sub
    _ => @base.error("Invalid CFG snapshot")
  }
}

///|
fn cond_json(v : Cond) -> Json {
  match v {
    Lt(v0, v1) => @base.tag("Lt", [value_json(v0), value_json(v1)])
    Eq(v0, v1) => @base.tag("Eq", [value_json(v0), value_json(v1)])
    Gt(v0, v1) => @base.tag("Gt", [value_json(v0), value_json(v1)])
  }
}

///|
fn decode_cond(j : Json) -> Cond raise @base.Fault {
  match @base.tag_name(j) {
    "Lt" =>
      Lt(decode_value(@base.tag_arg(j, 0)), decode_value(@base.tag_arg(j, 1)))
    "Eq" =>
      Eq(decode_value(@base.tag_arg(j, 0)), decode_value(@base.tag_arg(j, 1)))
    "Gt" =>
      Gt(decode_value(@base.tag_arg(j, 0)), decode_value(@base.tag_arg(j, 1)))
    _ => @base.error("Invalid CFG snapshot")
  }
}

///|
fn stmt_json(v : Stmt) -> Json {
  match v {
    Assign(v0, v1) => @base.tag("Assign", [@base.jint(v0), value_json(v1)])
    BinOp(v0, v1, v2, v3) =>
      @base.tag("BinOp", [
        @base.jint(v0),
        value_json(v1),
        binop_json(v2),
        value_json(v3),
      ])
    Store(v0, v1) => @base.tag("Store", [addr_json(v0), value_json(v1)])
  }
}

///|
fn decode_stmt(j : Json) -> Stmt raise @base.Fault {
  match @base.tag_name(j) {
    "Assign" =>
      Assign(@base.int(@base.tag_arg(j, 0)), decode_value(@base.tag_arg(j, 1)))
    "BinOp" =>
      BinOp(
        @base.int(@base.tag_arg(j, 0)),
        decode_value(@base.tag_arg(j, 1)),
        decode_binop(@base.tag_arg(j, 2)),
        decode_value(@base.tag_arg(j, 3)),
      )
    "Store" =>
      Store(decode_addr(@base.tag_arg(j, 0)), decode_value(@base.tag_arg(j, 1)))
    _ => @base.error("Invalid CFG snapshot")
  }
}

///|
fn block_json(b : Block) -> Json {
  @base.object([
    ("label", @base.jstr(b.label)),
    ("stmts", @base.jarray(b.stmts.map(stmt_json))),
    (
      "cont",
      @base.object([
        (
          "ifs",
          @base.jarray(
            b.cont.ifs.map(j => {
              @base.object([
                ("cond", cond_json(j.cond)),
                ("target", addr_json(j.target)),
              ])
            }),
          ),
        ),
        ("jump", addr_json(b.cont.jump)),
      ]),
    ),
  ])
}

///|
fn decode_block(j : Json) -> Block raise @base.Fault {
  let c = @base.field(j, "cont")
  {
    label: @base.identifier(@base.string(@base.field(j, "label"))),
    stmts: @base.array(@base.field(j, "stmts")).map(decode_stmt),
    cont: {
      ifs: @base.array(@base.field(c, "ifs")).map(j => {
        cond: decode_cond(@base.field(j, "cond")),
        target: decode_addr(@base.field(j, "target")),
      }),
      jump: decode_addr(@base.field(c, "jump")),
    },
  }
}

///|
fn code_json(c : Code) -> Json {
  @base.object([
    (
      "statics",
      @base.jarray(
        c.statics.map(s => {
          @base.object([
            ("label", @base.jstr(s.label)),
            ("value", @base.jstr(s.value.to_string())),
          ])
        }),
      ),
    ),
    ("blocks", @base.jarray(c.blocks.map(block_json))),
  ])
}

///|
fn decode_code(j : Json) -> Code raise @base.Fault {
  {
    statics: @base.array(@base.field(j, "statics")).map(j => {
      label: @base.identifier(@base.string(@base.field(j, "label"))),
      value: @base.json_nat(@base.field(j, "value")),
    }),
    blocks: @base.array(@base.field(j, "blocks")).map(decode_block),
  }
}

///|
fn compiled_json(c : Compiled) -> Json {
  @base.object([
    ("blocks", @base.jarray(c.blocks.map(block_json))),
    ("entry_block", @base.jint(c.entry_block)),
    ("block_index", @data.env_json(c.block_index, @base.jint)),
    ("static_labels", @data.env_json(c.static_labels, @base.jint)),
  ])
}

///|
fn decode_compiled(j : Json) -> Compiled raise @base.Fault {
  {
    blocks: @base.array(@base.field(j, "blocks")).map(decode_block),
    entry_block: @base.int(@base.field(j, "entry_block")),
    block_index: @data.decode_env(@base.field(j, "block_index"), @base.int),
    static_labels: @data.decode_env(@base.field(j, "static_labels"), @base.int),
  }
}
