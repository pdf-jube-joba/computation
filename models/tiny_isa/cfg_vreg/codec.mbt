///|
fn addr_json(v : Addr) -> Json {
  match v {
    VReg(v0) => @utils.tag("VReg", [@utils.jint(v0)])
    Label(v0) => @utils.tag("Label", [@utils.jstr(v0)])
    Imm(v0) => @utils.tag("Imm", [@utils.jstr(v0.to_string())])
  }
}

///|
fn decode_addr(j : Json) -> Addr raise @utils.Fault {
  match @utils.tag_name(j) {
    "VReg" => VReg(@utils.int(@utils.tag_arg(j, 0)))
    "Label" => Label(@utils.identifier(@utils.string(@utils.tag_arg(j, 0))))
    "Imm" => Imm(@utils.json_nat(@utils.tag_arg(j, 0)))
    _ => @utils.error("Invalid CFG snapshot")
  }
}

///|
fn value_json(v : Value) -> Json {
  match v {
    VRegValue(v0) => @utils.tag("VRegValue", [@utils.jint(v0)])
    ImmValue(v0) => @utils.tag("ImmValue", [@utils.jstr(v0.to_string())])
    LabelValue(v0) => @utils.tag("LabelValue", [@utils.jstr(v0)])
    Deref(v0) => @utils.tag("Deref", [addr_json(v0)])
  }
}

///|
fn decode_value(j : Json) -> Value raise @utils.Fault {
  match @utils.tag_name(j) {
    "VRegValue" => VRegValue(@utils.int(@utils.tag_arg(j, 0)))
    "ImmValue" => ImmValue(@utils.json_nat(@utils.tag_arg(j, 0)))
    "LabelValue" =>
      LabelValue(@utils.identifier(@utils.string(@utils.tag_arg(j, 0))))
    "Deref" => Deref(decode_addr(@utils.tag_arg(j, 0)))
    _ => @utils.error("Invalid CFG snapshot")
  }
}

///|
fn binop_json(v : Binop) -> Json {
  match v {
    Add => @utils.tag("Add", [])
    Sub => @utils.tag("Sub", [])
  }
}

///|
fn decode_binop(j : Json) -> Binop raise @utils.Fault {
  match @utils.tag_name(j) {
    "Add" => Add
    "Sub" => Sub
    _ => @utils.error("Invalid CFG snapshot")
  }
}

///|
fn cond_json(v : Cond) -> Json {
  match v {
    Lt(v0, v1) => @utils.tag("Lt", [value_json(v0), value_json(v1)])
    Eq(v0, v1) => @utils.tag("Eq", [value_json(v0), value_json(v1)])
    Gt(v0, v1) => @utils.tag("Gt", [value_json(v0), value_json(v1)])
  }
}

///|
fn decode_cond(j : Json) -> Cond raise @utils.Fault {
  match @utils.tag_name(j) {
    "Lt" =>
      Lt(decode_value(@utils.tag_arg(j, 0)), decode_value(@utils.tag_arg(j, 1)))
    "Eq" =>
      Eq(decode_value(@utils.tag_arg(j, 0)), decode_value(@utils.tag_arg(j, 1)))
    "Gt" =>
      Gt(decode_value(@utils.tag_arg(j, 0)), decode_value(@utils.tag_arg(j, 1)))
    _ => @utils.error("Invalid CFG snapshot")
  }
}

///|
fn stmt_json(v : Stmt) -> Json {
  match v {
    Assign(v0, v1) => @utils.tag("Assign", [@utils.jint(v0), value_json(v1)])
    BinOp(v0, v1, v2, v3) =>
      @utils.tag("BinOp", [
        @utils.jint(v0),
        value_json(v1),
        binop_json(v2),
        value_json(v3),
      ])
    Store(v0, v1) => @utils.tag("Store", [addr_json(v0), value_json(v1)])
  }
}

///|
fn decode_stmt(j : Json) -> Stmt raise @utils.Fault {
  match @utils.tag_name(j) {
    "Assign" =>
      Assign(
        @utils.int(@utils.tag_arg(j, 0)),
        decode_value(@utils.tag_arg(j, 1)),
      )
    "BinOp" =>
      BinOp(
        @utils.int(@utils.tag_arg(j, 0)),
        decode_value(@utils.tag_arg(j, 1)),
        decode_binop(@utils.tag_arg(j, 2)),
        decode_value(@utils.tag_arg(j, 3)),
      )
    "Store" =>
      Store(
        decode_addr(@utils.tag_arg(j, 0)),
        decode_value(@utils.tag_arg(j, 1)),
      )
    _ => @utils.error("Invalid CFG snapshot")
  }
}

///|
fn block_json(b : Block) -> Json {
  @utils.object([
    ("label", @utils.jstr(b.label)),
    ("stmts", @utils.jarray(b.stmts.map(stmt_json))),
    (
      "cont",
      @utils.object([
        (
          "ifs",
          @utils.jarray(
            b.cont.ifs.map(j => {
              @utils.object([
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
fn decode_block(j : Json) -> Block raise @utils.Fault {
  let c = @utils.field(j, "cont")
  {
    label: @utils.identifier(@utils.string(@utils.field(j, "label"))),
    stmts: @utils.array(@utils.field(j, "stmts")).map(decode_stmt),
    cont: {
      ifs: @utils.array(@utils.field(c, "ifs")).map(j => {
        cond: decode_cond(@utils.field(j, "cond")),
        target: decode_addr(@utils.field(j, "target")),
      }),
      jump: decode_addr(@utils.field(c, "jump")),
    },
  }
}

///|
fn code_json(c : Code) -> Json {
  @utils.object([
    (
      "statics",
      @utils.jarray(
        c.statics.map(s => {
          @utils.object([
            ("label", @utils.jstr(s.label)),
            ("value", @utils.jstr(s.value.to_string())),
          ])
        }),
      ),
    ),
    ("blocks", @utils.jarray(c.blocks.map(block_json))),
  ])
}

///|
fn decode_code(j : Json) -> Code raise @utils.Fault {
  {
    statics: @utils.array(@utils.field(j, "statics")).map(j => {
      label: @utils.identifier(@utils.string(@utils.field(j, "label"))),
      value: @utils.json_nat(@utils.field(j, "value")),
    }),
    blocks: @utils.array(@utils.field(j, "blocks")).map(decode_block),
  }
}

///|
fn compiled_json(c : Compiled) -> Json {
  @utils.object([
    ("blocks", @utils.jarray(c.blocks.map(block_json))),
    ("entry_block", @utils.jint(c.entry_block)),
    ("block_index", @utils.env_json(c.block_index, @utils.jint)),
    ("static_labels", @utils.env_json(c.static_labels, @utils.jint)),
  ])
}

///|
fn decode_compiled(j : Json) -> Compiled raise @utils.Fault {
  {
    blocks: @utils.array(@utils.field(j, "blocks")).map(decode_block),
    entry_block: @utils.int(@utils.field(j, "entry_block")),
    block_index: @utils.decode_env(@utils.field(j, "block_index"), @utils.int),
    static_labels: @utils.decode_env(
      @utils.field(j, "static_labels"),
      @utils.int,
    ),
  }
}
