///|
fn valueexpr_json(v : ValueExpr) -> Json {
  match v {
    Reg(v0) => @base.tag("Reg", [@base.jstr(v0)])
    Imm(v0) => @base.tag("Imm", [@base.jstr(v0.to_string())])
    Label(v0) => @base.tag("Label", [@base.jstr(v0)])
    Memory(v0) => @base.tag("Memory", [@base.jint(v0)])
  }
}

///|
fn decode_valueexpr(j : Json) -> ValueExpr raise @base.Fault {
  match @base.tag_name(j) {
    "Reg" => Reg(@base.identifier(@base.string(@base.tag_arg(j, 0))))
    "Imm" => Imm(@base.json_nat(@base.tag_arg(j, 0)))
    "Label" => Label(@base.identifier(@base.string(@base.tag_arg(j, 0))))
    "Memory" => Memory(@base.int(@base.tag_arg(j, 0)))
    _ => @base.error("Invalid core machine snapshot")
  }
}

///|
fn place_json(v : Place) -> Json {
  match v {
    Deref(v0) => @base.tag("Deref", [valueexpr_json(v0)])
  }
}

///|
fn decode_place(j : Json) -> Place raise @base.Fault {
  match @base.tag_name(j) {
    "Deref" => Deref(decode_valueexpr(@base.tag_arg(j, 0)))
    _ => @base.error("Invalid core machine snapshot")
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
    _ => @base.error("Invalid core machine snapshot")
  }
}

///|
fn rhs_json(v : Rhs) -> Json {
  match v {
    Value(v0) => @base.tag("Value", [valueexpr_json(v0)])
    Binop(v0, v1, v2) =>
      @base.tag("Binop", [
        valueexpr_json(v0),
        binop_json(v1),
        valueexpr_json(v2),
      ])
    Load(v0) => @base.tag("Load", [place_json(v0)])
    Slice(v0, v1, v2) =>
      @base.tag("Slice", [
        valueexpr_json(v0),
        valueexpr_json(v1),
        valueexpr_json(v2),
      ])
    Addr(v0, v1) => @base.tag("Addr", [valueexpr_json(v0), valueexpr_json(v1)])
    Len(v0) => @base.tag("Len", [valueexpr_json(v0)])
  }
}

///|
fn decode_rhs(j : Json) -> Rhs raise @base.Fault {
  match @base.tag_name(j) {
    "Value" => Value(decode_valueexpr(@base.tag_arg(j, 0)))
    "Binop" =>
      Binop(
        decode_valueexpr(@base.tag_arg(j, 0)),
        decode_binop(@base.tag_arg(j, 1)),
        decode_valueexpr(@base.tag_arg(j, 2)),
      )
    "Load" => Load(decode_place(@base.tag_arg(j, 0)))
    "Slice" =>
      Slice(
        decode_valueexpr(@base.tag_arg(j, 0)),
        decode_valueexpr(@base.tag_arg(j, 1)),
        decode_valueexpr(@base.tag_arg(j, 2)),
      )
    "Addr" =>
      Addr(
        decode_valueexpr(@base.tag_arg(j, 0)),
        decode_valueexpr(@base.tag_arg(j, 1)),
      )
    "Len" => Len(decode_valueexpr(@base.tag_arg(j, 0)))
    _ => @base.error("Invalid core machine snapshot")
  }
}

///|
fn stmt_json(v : Stmt) -> Json {
  match v {
    Assign(v0, v1) => @base.tag("Assign", [@base.jstr(v0), rhs_json(v1)])
    Store(v0, v1) => @base.tag("Store", [place_json(v0), valueexpr_json(v1)])
    Print(v0) => @base.tag("Print", [valueexpr_json(v0)])
    Input(v0) => @base.tag("Input", [place_json(v0)])
  }
}

///|
fn decode_stmt(j : Json) -> Stmt raise @base.Fault {
  match @base.tag_name(j) {
    "Assign" =>
      Assign(
        @base.identifier(@base.string(@base.tag_arg(j, 0))),
        decode_rhs(@base.tag_arg(j, 1)),
      )
    "Store" =>
      Store(
        decode_place(@base.tag_arg(j, 0)),
        decode_valueexpr(@base.tag_arg(j, 1)),
      )
    "Print" => Print(decode_valueexpr(@base.tag_arg(j, 0)))
    "Input" => Input(decode_place(@base.tag_arg(j, 0)))
    _ => @base.error("Invalid core machine snapshot")
  }
}

///|
fn rel_json(v : Rel) -> Json {
  match v {
    Eq => @base.tag("Eq", [])
    Lt => @base.tag("Lt", [])
    Gt => @base.tag("Gt", [])
  }
}

///|
fn decode_rel(j : Json) -> Rel raise @base.Fault {
  match @base.tag_name(j) {
    "Eq" => Eq
    "Lt" => Lt
    "Gt" => Gt
    _ => @base.error("Invalid core machine snapshot")
  }
}

///|
fn ending_json(v : Ending) -> Json {
  match v {
    Goto(v0) => @base.tag("Goto", [valueexpr_json(v0)])
    Halt => @base.tag("Halt", [])
  }
}

///|
fn decode_ending(j : Json) -> Ending raise @base.Fault {
  match @base.tag_name(j) {
    "Goto" => Goto(decode_valueexpr(@base.tag_arg(j, 0)))
    "Halt" => Halt
    _ => @base.error("Invalid core machine snapshot")
  }
}

///|
fn slicevalue_json(v : SliceValue) -> Json {
  match v {
    Whole(v0) => @base.tag("Whole", [@base.jint(v0)])
    Subslice(v0, v1, v2) =>
      @base.tag("Subslice", [
        slicevalue_json(v0),
        @base.jstr(v1.to_string()),
        @base.jstr(v2.to_string()),
      ])
  }
}

///|
fn decode_slicevalue(j : Json) -> SliceValue raise @base.Fault {
  match @base.tag_name(j) {
    "Whole" => Whole(@base.int(@base.tag_arg(j, 0)))
    "Subslice" =>
      Subslice(
        decode_slicevalue(@base.tag_arg(j, 0)),
        @base.json_nat(@base.tag_arg(j, 1)),
        @base.json_nat(@base.tag_arg(j, 2)),
      )
    _ => @base.error("Invalid core machine snapshot")
  }
}

///|
fn runtimevalue_json(v : RuntimeValue) -> Json {
  match v {
    Num(v0) => @base.tag("Num", [@base.jstr(v0.to_string())])
    CodeLabel(v0) => @base.tag("Code_label", [@base.jstr(v0)])
    SliceValue(v0) => @base.tag("Slice_value", [slicevalue_json(v0)])
    Address(v0, v1) =>
      @base.tag("Address", [slicevalue_json(v0), @base.jstr(v1.to_string())])
  }
}

///|
fn decode_runtimevalue(j : Json) -> RuntimeValue raise @base.Fault {
  match @base.tag_name(j) {
    "Num" => Num(@base.json_nat(@base.tag_arg(j, 0)))
    "Code_label" =>
      CodeLabel(@base.identifier(@base.string(@base.tag_arg(j, 0))))
    "Slice_value" => SliceValue(decode_slicevalue(@base.tag_arg(j, 0)))
    "Address" =>
      Address(
        decode_slicevalue(@base.tag_arg(j, 0)),
        @base.json_nat(@base.tag_arg(j, 1)),
      )
    _ => @base.error("Invalid core machine snapshot")
  }
}

///|
fn cond_json(c : Cond) -> Json {
  @base.jarray([valueexpr_json(c.0), rel_json(c.1), valueexpr_json(c.2)])
}

///|
fn decode_cond(j : Json) -> Cond raise @base.Fault {
  let a = @base.array(j)
  @base.require(a.length() == 3, "Invalid comparison")
  (decode_valueexpr(a[0]), decode_rel(a[1]), decode_valueexpr(a[2]))
}

///|
fn block_json(b : Block) -> Json {
  @base.object([
    ("label", @base.jstr(b.label)),
    ("stmts", @base.jarray(b.stmts.map(stmt_json))),
    (
      "ifs",
      @base.jarray(
        b.ifs.map(j => {
          @base.object([
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
fn decode_block(j : Json) -> Block raise @base.Fault {
  {
    label: @base.identifier(@base.string(@base.field(j, "label"))),
    stmts: @base.array(@base.field(j, "stmts")).map(decode_stmt),
    ifs: @base.array(@base.field(j, "ifs")).map(j => {
      cond: decode_cond(@base.field(j, "cond")),
      target: decode_valueexpr(@base.field(j, "target")),
    }),
    ending: decode_ending(@base.field(j, "ending")),
  }
}

///|
fn code_json(c : Code) -> Json {
  @base.object([
    ("memories", @base.jint(c.memories)),
    ("blocks", @base.jarray(c.blocks.map(block_json))),
  ])
}

///|
fn decode_code(j : Json) -> Code raise @base.Fault {
  {
    memories: @base.int(@base.field(j, "memories")),
    blocks: @base.array(@base.field(j, "blocks")).map(decode_block),
  }
}

///|
fn memory_json(mem : Memory) -> Json {
  @base.jarray(
    mem.map(p => {
      @base.jarray([@base.jstr(p.0.to_string()), runtimevalue_json(p.1)])
    }),
  )
}

///|
fn decode_memory(j : Json) -> Memory raise @base.Fault {
  @base.array(j).map(j => {
    let a = @base.array(j)
    @base.require(a.length() == 2, "Invalid memory entry")
    (@base.json_nat(a[0]), decode_runtimevalue(a[1]))
  })
}
