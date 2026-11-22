///|
fn valueexpr_json(v : ValueExpr) -> Json {
  match v {
    Reg(v0) => @utils.tag("Reg", [@utils.jstr(v0)])
    Imm(v0) => @utils.tag("Imm", [@utils.jstr(v0.to_string())])
    Label(v0) => @utils.tag("Label", [@utils.jstr(v0)])
    Memory(v0) => @utils.tag("Memory", [@utils.jint(v0)])
  }
}

///|
fn decode_valueexpr(j : Json) -> ValueExpr raise @utils.Fault {
  match @utils.tag_name(j) {
    "Reg" => Reg(@utils.identifier(@utils.string(@utils.tag_arg(j, 0))))
    "Imm" => Imm(@utils.json_nat(@utils.tag_arg(j, 0)))
    "Label" => Label(@utils.identifier(@utils.string(@utils.tag_arg(j, 0))))
    "Memory" => Memory(@utils.int(@utils.tag_arg(j, 0)))
    _ => @utils.error("Invalid core machine snapshot")
  }
}

///|
fn place_json(v : Place) -> Json {
  match v {
    Deref(v0) => @utils.tag("Deref", [valueexpr_json(v0)])
  }
}

///|
fn decode_place(j : Json) -> Place raise @utils.Fault {
  match @utils.tag_name(j) {
    "Deref" => Deref(decode_valueexpr(@utils.tag_arg(j, 0)))
    _ => @utils.error("Invalid core machine snapshot")
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
    _ => @utils.error("Invalid core machine snapshot")
  }
}

///|
fn rhs_json(v : Rhs) -> Json {
  match v {
    Value(v0) => @utils.tag("Value", [valueexpr_json(v0)])
    Binop(v0, v1, v2) =>
      @utils.tag("Binop", [
        valueexpr_json(v0),
        binop_json(v1),
        valueexpr_json(v2),
      ])
    Load(v0) => @utils.tag("Load", [place_json(v0)])
    Slice(v0, v1, v2) =>
      @utils.tag("Slice", [
        valueexpr_json(v0),
        valueexpr_json(v1),
        valueexpr_json(v2),
      ])
    Addr(v0, v1) => @utils.tag("Addr", [valueexpr_json(v0), valueexpr_json(v1)])
    Len(v0) => @utils.tag("Len", [valueexpr_json(v0)])
  }
}

///|
fn decode_rhs(j : Json) -> Rhs raise @utils.Fault {
  match @utils.tag_name(j) {
    "Value" => Value(decode_valueexpr(@utils.tag_arg(j, 0)))
    "Binop" =>
      Binop(
        decode_valueexpr(@utils.tag_arg(j, 0)),
        decode_binop(@utils.tag_arg(j, 1)),
        decode_valueexpr(@utils.tag_arg(j, 2)),
      )
    "Load" => Load(decode_place(@utils.tag_arg(j, 0)))
    "Slice" =>
      Slice(
        decode_valueexpr(@utils.tag_arg(j, 0)),
        decode_valueexpr(@utils.tag_arg(j, 1)),
        decode_valueexpr(@utils.tag_arg(j, 2)),
      )
    "Addr" =>
      Addr(
        decode_valueexpr(@utils.tag_arg(j, 0)),
        decode_valueexpr(@utils.tag_arg(j, 1)),
      )
    "Len" => Len(decode_valueexpr(@utils.tag_arg(j, 0)))
    _ => @utils.error("Invalid core machine snapshot")
  }
}

///|
fn stmt_json(v : Stmt) -> Json {
  match v {
    Assign(v0, v1) => @utils.tag("Assign", [@utils.jstr(v0), rhs_json(v1)])
    Store(v0, v1) => @utils.tag("Store", [place_json(v0), valueexpr_json(v1)])
    Print(v0) => @utils.tag("Print", [valueexpr_json(v0)])
    Input(v0) => @utils.tag("Input", [place_json(v0)])
  }
}

///|
fn decode_stmt(j : Json) -> Stmt raise @utils.Fault {
  match @utils.tag_name(j) {
    "Assign" =>
      Assign(
        @utils.identifier(@utils.string(@utils.tag_arg(j, 0))),
        decode_rhs(@utils.tag_arg(j, 1)),
      )
    "Store" =>
      Store(
        decode_place(@utils.tag_arg(j, 0)),
        decode_valueexpr(@utils.tag_arg(j, 1)),
      )
    "Print" => Print(decode_valueexpr(@utils.tag_arg(j, 0)))
    "Input" => Input(decode_place(@utils.tag_arg(j, 0)))
    _ => @utils.error("Invalid core machine snapshot")
  }
}

///|
fn rel_json(v : Rel) -> Json {
  match v {
    Eq => @utils.tag("Eq", [])
    Lt => @utils.tag("Lt", [])
    Gt => @utils.tag("Gt", [])
  }
}

///|
fn decode_rel(j : Json) -> Rel raise @utils.Fault {
  match @utils.tag_name(j) {
    "Eq" => Eq
    "Lt" => Lt
    "Gt" => Gt
    _ => @utils.error("Invalid core machine snapshot")
  }
}

///|
fn ending_json(v : Ending) -> Json {
  match v {
    Goto(v0) => @utils.tag("Goto", [valueexpr_json(v0)])
    Halt => @utils.tag("Halt", [])
  }
}

///|
fn decode_ending(j : Json) -> Ending raise @utils.Fault {
  match @utils.tag_name(j) {
    "Goto" => Goto(decode_valueexpr(@utils.tag_arg(j, 0)))
    "Halt" => Halt
    _ => @utils.error("Invalid core machine snapshot")
  }
}

///|
fn slicevalue_json(v : SliceValue) -> Json {
  match v {
    Whole(v0) => @utils.tag("Whole", [@utils.jint(v0)])
    Subslice(v0, v1, v2) =>
      @utils.tag("Subslice", [
        slicevalue_json(v0),
        @utils.jstr(v1.to_string()),
        @utils.jstr(v2.to_string()),
      ])
  }
}

///|
fn decode_slicevalue(j : Json) -> SliceValue raise @utils.Fault {
  match @utils.tag_name(j) {
    "Whole" => Whole(@utils.int(@utils.tag_arg(j, 0)))
    "Subslice" =>
      Subslice(
        decode_slicevalue(@utils.tag_arg(j, 0)),
        @utils.json_nat(@utils.tag_arg(j, 1)),
        @utils.json_nat(@utils.tag_arg(j, 2)),
      )
    _ => @utils.error("Invalid core machine snapshot")
  }
}

///|
fn runtimevalue_json(v : RuntimeValue) -> Json {
  match v {
    Num(v0) => @utils.tag("Num", [@utils.jstr(v0.to_string())])
    CodeLabel(v0) => @utils.tag("Code_label", [@utils.jstr(v0)])
    SliceValue(v0) => @utils.tag("Slice_value", [slicevalue_json(v0)])
    Address(v0, v1) =>
      @utils.tag("Address", [slicevalue_json(v0), @utils.jstr(v1.to_string())])
  }
}

///|
fn decode_runtimevalue(j : Json) -> RuntimeValue raise @utils.Fault {
  match @utils.tag_name(j) {
    "Num" => Num(@utils.json_nat(@utils.tag_arg(j, 0)))
    "Code_label" =>
      CodeLabel(@utils.identifier(@utils.string(@utils.tag_arg(j, 0))))
    "Slice_value" => SliceValue(decode_slicevalue(@utils.tag_arg(j, 0)))
    "Address" =>
      Address(
        decode_slicevalue(@utils.tag_arg(j, 0)),
        @utils.json_nat(@utils.tag_arg(j, 1)),
      )
    _ => @utils.error("Invalid core machine snapshot")
  }
}

///|
fn cond_json(c : Cond) -> Json {
  @utils.jarray([valueexpr_json(c.0), rel_json(c.1), valueexpr_json(c.2)])
}

///|
fn decode_cond(j : Json) -> Cond raise @utils.Fault {
  let a = @utils.array(j)
  @utils.require(a.length() == 3, "Invalid comparison")
  (decode_valueexpr(a[0]), decode_rel(a[1]), decode_valueexpr(a[2]))
}

///|
fn block_json(b : Block) -> Json {
  @utils.object([
    ("label", @utils.jstr(b.label)),
    ("stmts", @utils.jarray(b.stmts.map(stmt_json))),
    (
      "ifs",
      @utils.jarray(
        b.ifs.map(j => {
          @utils.object([
            ("cond", cond_json(j.cond)),
            ("target", valueexpr_json(j.target)),
          ])
        }),
      ),
    ),
    ("ending", ending_json(b.ending)),
  ])
}

///|
fn decode_block(j : Json) -> Block raise @utils.Fault {
  {
    label: @utils.identifier(@utils.string(@utils.field(j, "label"))),
    stmts: @utils.array(@utils.field(j, "stmts")).map(decode_stmt),
    ifs: @utils.array(@utils.field(j, "ifs")).map(j => {
      cond: decode_cond(@utils.field(j, "cond")),
      target: decode_valueexpr(@utils.field(j, "target")),
    }),
    ending: decode_ending(@utils.field(j, "ending")),
  }
}

///|
fn code_json(c : Code) -> Json {
  @utils.object([
    ("memories", @utils.jint(c.memories)),
    ("blocks", @utils.jarray(c.blocks.map(block_json))),
  ])
}

///|
fn decode_code(j : Json) -> Code raise @utils.Fault {
  {
    memories: @utils.int(@utils.field(j, "memories")),
    blocks: @utils.array(@utils.field(j, "blocks")).map(decode_block),
  }
}

///|
fn memory_json(mem : Memory) -> Json {
  @utils.jarray(
    mem.map(p => {
      @utils.jarray([@utils.jstr(p.0.to_string()), runtimevalue_json(p.1)])
    }),
  )
}

///|
fn decode_memory(j : Json) -> Memory raise @utils.Fault {
  @utils.array(j).map(j => {
    let a = @utils.array(j)
    @utils.require(a.length() == 2, "Invalid memory entry")
    (@utils.json_nat(a[0]), decode_runtimevalue(a[1]))
  })
}
