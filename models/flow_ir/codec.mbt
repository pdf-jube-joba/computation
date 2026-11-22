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
    _ => @utils.error("Invalid flow IR snapshot")
  }
}

///|
fn valueexpr_json(v : ValueExpr) -> Json {
  match v {
    VReg(v0) => @utils.tag("VReg", [@utils.jstr(v0)])
    Imm(v0) => @utils.tag("Imm", [@utils.jstr(v0.to_string())])
    CodeLabel(v0) => @utils.tag("CodeLabel", [@utils.jstr(v0)])
    Ref(v0) => @utils.tag("Ref", [placeexpr_json(v0)])
  }
}

///|
fn decode_valueexpr(j : Json) -> ValueExpr raise @utils.Fault {
  match @utils.tag_name(j) {
    "VReg" => VReg(@utils.identifier(@utils.string(@utils.tag_arg(j, 0))))
    "Imm" => Imm(@utils.json_nat(@utils.tag_arg(j, 0)))
    "CodeLabel" =>
      CodeLabel(@utils.identifier(@utils.string(@utils.tag_arg(j, 0))))
    "Ref" => Ref(decode_placeexpr(@utils.tag_arg(j, 0)))
    _ => @utils.error("Invalid flow IR snapshot")
  }
}

///|
fn placeexpr_json(v : PlaceExpr) -> Json {
  match v {
    Label(v0) => @utils.tag("Label", [@utils.jstr(v0)])
    Deref(v0) => @utils.tag("Deref", [@utils.jstr(v0)])
    SAcc(v0) => @utils.tag("SAcc", [valueexpr_json(v0)])
    HAcc(v0, v1) => @utils.tag("HAcc", [valueexpr_json(v0), valueexpr_json(v1)])
  }
}

///|
fn decode_placeexpr(j : Json) -> PlaceExpr raise @utils.Fault {
  match @utils.tag_name(j) {
    "Label" => Label(@utils.identifier(@utils.string(@utils.tag_arg(j, 0))))
    "Deref" => Deref(@utils.identifier(@utils.string(@utils.tag_arg(j, 0))))
    "SAcc" => SAcc(decode_valueexpr(@utils.tag_arg(j, 0)))
    "HAcc" =>
      HAcc(
        decode_valueexpr(@utils.tag_arg(j, 0)),
        decode_valueexpr(@utils.tag_arg(j, 1)),
      )
    _ => @utils.error("Invalid flow IR snapshot")
  }
}

///|
fn cond_json(v : Cond) -> Json {
  match v {
    Lt(v0, v1) => @utils.tag("Lt", [valueexpr_json(v0), valueexpr_json(v1)])
    Eq(v0, v1) => @utils.tag("Eq", [valueexpr_json(v0), valueexpr_json(v1)])
    Gt(v0, v1) => @utils.tag("Gt", [valueexpr_json(v0), valueexpr_json(v1)])
  }
}

///|
fn decode_cond(j : Json) -> Cond raise @utils.Fault {
  match @utils.tag_name(j) {
    "Lt" =>
      Lt(
        decode_valueexpr(@utils.tag_arg(j, 0)),
        decode_valueexpr(@utils.tag_arg(j, 1)),
      )
    "Eq" =>
      Eq(
        decode_valueexpr(@utils.tag_arg(j, 0)),
        decode_valueexpr(@utils.tag_arg(j, 1)),
      )
    "Gt" =>
      Gt(
        decode_valueexpr(@utils.tag_arg(j, 0)),
        decode_valueexpr(@utils.tag_arg(j, 1)),
      )
    _ => @utils.error("Invalid flow IR snapshot")
  }
}

///|
fn stmt_json(v : Stmt) -> Json {
  match v {
    Nop => @utils.tag("Nop", [])
    Load(v0, v1) => @utils.tag("Load", [@utils.jstr(v0), placeexpr_json(v1)])
    Assign(v0, v1) =>
      @utils.tag("Assign", [@utils.jstr(v0), valueexpr_json(v1)])
    BinOp(v0, v1, v2, v3) =>
      @utils.tag("BinOp", [
        @utils.jstr(v0),
        valueexpr_json(v1),
        binop_json(v2),
        valueexpr_json(v3),
      ])
    Store(v0, v1) =>
      @utils.tag("Store", [placeexpr_json(v0), valueexpr_json(v1)])
    Pop(v0) => @utils.tag("Pop", [@utils.jstr(v0)])
    Push(v0) => @utils.tag("Push", [valueexpr_json(v0)])
    LGet(v0) => @utils.tag("LGet", [@utils.jstr(v0)])
    HAlloc(v0, v1) =>
      @utils.tag("HAlloc", [valueexpr_json(v0), @utils.jstr(v1)])
    HFree(v0) => @utils.tag("HFree", [valueexpr_json(v0)])
    Print(v0) => @utils.tag("Print", [@utils.jstr(v0)])
    Input(v0) => @utils.tag("Input", [placeexpr_json(v0)])
  }
}

///|
fn decode_stmt(j : Json) -> Stmt raise @utils.Fault {
  match @utils.tag_name(j) {
    "Nop" => Nop
    "Load" =>
      Load(
        @utils.identifier(@utils.string(@utils.tag_arg(j, 0))),
        decode_placeexpr(@utils.tag_arg(j, 1)),
      )
    "Assign" =>
      Assign(
        @utils.identifier(@utils.string(@utils.tag_arg(j, 0))),
        decode_valueexpr(@utils.tag_arg(j, 1)),
      )
    "BinOp" =>
      BinOp(
        @utils.identifier(@utils.string(@utils.tag_arg(j, 0))),
        decode_valueexpr(@utils.tag_arg(j, 1)),
        decode_binop(@utils.tag_arg(j, 2)),
        decode_valueexpr(@utils.tag_arg(j, 3)),
      )
    "Store" =>
      Store(
        decode_placeexpr(@utils.tag_arg(j, 0)),
        decode_valueexpr(@utils.tag_arg(j, 1)),
      )
    "Pop" => Pop(@utils.identifier(@utils.string(@utils.tag_arg(j, 0))))
    "Push" => Push(decode_valueexpr(@utils.tag_arg(j, 0)))
    "LGet" => LGet(@utils.identifier(@utils.string(@utils.tag_arg(j, 0))))
    "HAlloc" =>
      HAlloc(
        decode_valueexpr(@utils.tag_arg(j, 0)),
        @utils.identifier(@utils.string(@utils.tag_arg(j, 1))),
      )
    "HFree" => HFree(decode_valueexpr(@utils.tag_arg(j, 0)))
    "Print" => Print(@utils.identifier(@utils.string(@utils.tag_arg(j, 0))))
    "Input" => Input(decode_placeexpr(@utils.tag_arg(j, 0)))
    _ => @utils.error("Invalid flow IR snapshot")
  }
}

///|
fn cont_json(v : Cont) -> Json {
  match v {
    Go(v0, v1) => @utils.tag("Go", [jumps_json(v0), valueexpr_json(v1)])
    Enter(v0, v1) => @utils.tag("Enter", [jumps_json(v0), valueexpr_json(v1)])
    Halt(v0) => @utils.tag("Halt", [jumps_json(v0)])
  }
}

///|
fn decode_cont(j : Json) -> Cont raise @utils.Fault {
  match @utils.tag_name(j) {
    "Go" =>
      Go(
        decode_jumps(@utils.tag_arg(j, 0)),
        decode_valueexpr(@utils.tag_arg(j, 1)),
      )
    "Enter" =>
      Enter(
        decode_jumps(@utils.tag_arg(j, 0)),
        decode_valueexpr(@utils.tag_arg(j, 1)),
      )
    "Halt" => Halt(decode_jumps(@utils.tag_arg(j, 0)))
    _ => @utils.error("Invalid flow IR snapshot")
  }
}

///|
fn target_json(v : Target) -> Json {
  match v {
    RegionTarget(v0) => @utils.tag("Region_target", [@utils.jint(v0)])
    BlockTarget(v0) => @utils.tag("Block_target", [@utils.jint(v0)])
  }
}

///|
fn decode_target(j : Json) -> Target raise @utils.Fault {
  match @utils.tag_name(j) {
    "Region_target" => RegionTarget(id(@utils.tag_arg(j, 0)))
    "Block_target" => BlockTarget(id(@utils.tag_arg(j, 0)))
    _ => @utils.error("Invalid flow IR snapshot")
  }
}

///|
fn id(j : Json) -> Int32 raise @utils.Fault {
  let id = @utils.int(j)
  @utils.require(id >= 0 && id < 2147483647, "Invalid resolved ID")
  id
}

///|
fn jumps_json(jumps : Array[JumpIf]) -> Json {
  @utils.jarray(
    jumps.map(j => {
      @utils.object([
        ("cond", cond_json(j.cond)),
        ("target", valueexpr_json(j.target)),
      ])
    }),
  )
}

///|
fn decode_jumps(j : Json) -> Array[JumpIf] raise @utils.Fault {
  @utils.array(j).map(j => {
    cond: decode_cond(@utils.field(j, "cond")),
    target: decode_valueexpr(@utils.field(j, "target")),
  })
}

///|
fn block_json(b : Block) -> Json {
  @utils.object([
    ("label", @utils.jstr(b.label)),
    ("stmts", @utils.jarray(b.stmts.map(stmt_json))),
    ("cont", cont_json(b.cont)),
  ])
}

///|
fn decode_block(j : Json) -> Block raise @utils.Fault {
  {
    label: @utils.identifier(@utils.string(@utils.field(j, "label"))),
    stmts: @utils.array(@utils.field(j, "stmts")).map(decode_stmt),
    cont: decode_cont(@utils.field(j, "cont")),
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
    (
      "regions",
      @utils.jarray(
        c.regions.map(r => {
          @utils.object([
            ("label", @utils.jstr(r.label)),
            ("blocks", @utils.jarray(r.blocks.map(block_json))),
          ])
        }),
      ),
    ),
  ])
}

///|
fn decode_code(j : Json) -> Code raise @utils.Fault {
  {
    statics: @utils.array(@utils.field(j, "statics")).map(j => {
      label: @utils.identifier(@utils.string(@utils.field(j, "label"))),
      value: @utils.json_nat(@utils.field(j, "value")),
    }),
    regions: @utils.array(@utils.field(j, "regions")).map(j => {
      label: @utils.identifier(@utils.string(@utils.field(j, "label"))),
      blocks: @utils.array(@utils.field(j, "blocks")).map(decode_block),
    }),
  }
}

///|
fn[D] location_json(location : Location[D], data : (D) -> Json) -> Json {
  match location {
    Static(d) => @utils.tag("Static", [data(d)])
    Stack(i) => @utils.tag("Stack", [@utils.jint(i)])
    Heap(h, i) => @utils.tag("Heap", [@utils.jint(h), @utils.jint(i)])
  }
}

///|
fn[D] decode_location(
  j : Json,
  data : (Json) -> D raise @utils.Fault,
) -> Location[D] raise @utils.Fault {
  match @utils.tag_name(j) {
    "Static" => Static(data(@utils.tag_arg(j, 0)))
    "Stack" => Stack(@utils.int(@utils.tag_arg(j, 0)))
    "Heap" => Heap(id(@utils.tag_arg(j, 0)), @utils.int(@utils.tag_arg(j, 1)))
    _ => @utils.error("Invalid memory location")
  }
}

///|
fn runtime_json(v : RuntimeValue) -> Json {
  match v {
    Num(n) => @utils.tag("Num", [@utils.jstr(n.to_string())])
    Code_label(t) => @utils.tag("Code_label", [target_json(t)])
    Key(k) => @utils.tag("Key", [location_json(k, @utils.jint)])
  }
}

///|
fn decode_runtime(j : Json) -> RuntimeValue raise @utils.Fault {
  match @utils.tag_name(j) {
    "Num" => Num(@utils.json_nat(@utils.tag_arg(j, 0)))
    "Code_label" => Code_label(decode_target(@utils.tag_arg(j, 0)))
    "Key" => Key(decode_location(@utils.tag_arg(j, 0), id))
    _ => @utils.error("Invalid runtime value")
  }
}

///|
fn resolution_json(r : Resolution) -> Json {
  @utils.object([
    ("registers", @utils.env_json(r.registers, @utils.jint)),
    ("data", @utils.env_json(r.data, @utils.jint)),
    ("regions", @utils.env_json(r.regions, @utils.jint)),
    (
      "blocks",
      @utils.jarray(r.blocks.map(xs => @utils.env_json(xs, @utils.jint))),
    ),
    (
      "label_names",
      @utils.jarray(
        r.label_names.map(p => {
          @utils.jarray([target_json(p.0), @utils.jstr(p.1)])
        }),
      ),
    ),
  ])
}

///|
fn decode_resolution(j : Json) -> Resolution raise @utils.Fault {
  {
    registers: @utils.decode_env(@utils.field(j, "registers"), id),
    data: @utils.decode_env(@utils.field(j, "data"), id),
    regions: @utils.decode_env(@utils.field(j, "regions"), id),
    blocks: @utils.array(@utils.field(j, "blocks")).map(j => {
      @utils.decode_env(j, id)
    }),
    label_names: @utils.array(@utils.field(j, "label_names")).map(j => {
      let a = @utils.array(j)
      @utils.require(a.length() == 2, "Invalid label name")
      (decode_target(a[0]), @utils.identifier(@utils.string(a[1])))
    }),
  }
}

///|
fn[T] id_pairs_json(entries : Array[(Int32, T)], encode : (T) -> Json) -> Json {
  @utils.jarray(
    entries.map(p => @utils.jarray([@utils.jint(p.0), encode(p.1)])),
  )
}

///|
fn[T] decode_id_pairs(
  j : Json,
  decode : (Json) -> T raise @utils.Fault,
) -> Array[(Int32, T)] raise @utils.Fault {
  @utils.array(j).map(j => {
    let a = @utils.array(j)
    @utils.require(a.length() == 2, "Invalid ID binding")
    (id(a[0]), decode(a[1]))
  })
}

///|
fn encode(s : State) -> Json {
  @utils.object([
    ("code", code_json(s.code)),
    ("current_region", @utils.jint(s.current_region)),
    ("current_block", @utils.jint(s.current_block)),
    ("current_line", @utils.jint(s.current_line)),
    ("resolution", resolution_json(s.resolution)),
    ("vregs", id_pairs_json(s.vregs, runtime_json)),
    ("static_mem", id_pairs_json(s.static_mem, runtime_json)),
    ("stack", @utils.jarray(s.stack.map(runtime_json))),
    ("heap", id_pairs_json(s.heap, vs => @utils.jarray(vs.map(runtime_json)))),
    ("next_handle", @utils.jint(s.next_handle)),
    ("halted", @utils.jbool(s.halted)),
  ])
}

///|
fn decode(j : Json) -> State raise @utils.Fault {
  let code = decode_code(@utils.field(j, "code"))
  validate(code)
  let resolution = decode_resolution(@utils.field(j, "resolution"))
  @utils.require(
    resolution == build_resolution(code),
    "Inconsistent name resolution in snapshot",
  )
  let s = {
    code,
    resolution,
    current_region: @utils.int(@utils.field(j, "current_region")),
    current_block: @utils.int(@utils.field(j, "current_block")),
    current_line: @utils.int(@utils.field(j, "current_line")),
    vregs: decode_id_pairs(@utils.field(j, "vregs"), decode_runtime),
    static_mem: decode_id_pairs(@utils.field(j, "static_mem"), decode_runtime),
    stack: @utils.array(@utils.field(j, "stack")).map(decode_runtime),
    heap: decode_id_pairs(@utils.field(j, "heap"), j => {
      @utils.array(j).map(decode_runtime)
    }),
    next_handle: @utils.int(@utils.field(j, "next_handle")),
    halted: @utils.boolean(@utils.field(j, "halted")),
  }
  @utils.require(s.next_handle >= 0, "Expected nonnegative ID supply")
  fn check(v : RuntimeValue) -> Unit raise @utils.Fault {
    match v {
      Num(_) => ()
      Code_label(t) => ignore(target_name(resolution, t))
      Key(Static(id)) => ignore(name_of_id(resolution.data, id))
      Key(Heap(h, _)) =>
        @utils.require(
          h < s.next_handle,
          "ID supply would reuse an existing ID",
        )
      Key(Stack(_)) => ()
    }
  }
  for p in s.vregs {
    ignore(name_of_id(resolution.registers, p.0))
    check(p.1)
  }
  for p in s.static_mem {
    ignore(name_of_id(resolution.data, p.0))
    check(p.1)
  }
  for v in s.stack {
    check(v)
  }
  for p in s.heap {
    @utils.require(p.0 < s.next_handle, "ID supply would reuse an existing ID")
    for v in p.1 {
      check(v)
    }
  }
  s
}
