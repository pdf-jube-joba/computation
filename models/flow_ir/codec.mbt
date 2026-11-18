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
    _ => @base.error("Invalid flow IR snapshot")
  }
}

///|
fn valueexpr_json(v : ValueExpr) -> Json {
  match v {
    VReg(v0) => @base.tag("VReg", [@base.jstr(v0)])
    Imm(v0) => @base.tag("Imm", [@base.jstr(v0.to_string())])
    CodeLabel(v0) => @base.tag("CodeLabel", [@base.jstr(v0)])
    Ref(v0) => @base.tag("Ref", [placeexpr_json(v0)])
  }
}

///|
fn decode_valueexpr(j : Json) -> ValueExpr raise @base.Fault {
  match @base.tag_name(j) {
    "VReg" => VReg(@base.identifier(@base.string(@base.tag_arg(j, 0))))
    "Imm" => Imm(@base.json_nat(@base.tag_arg(j, 0)))
    "CodeLabel" =>
      CodeLabel(@base.identifier(@base.string(@base.tag_arg(j, 0))))
    "Ref" => Ref(decode_placeexpr(@base.tag_arg(j, 0)))
    _ => @base.error("Invalid flow IR snapshot")
  }
}

///|
fn placeexpr_json(v : PlaceExpr) -> Json {
  match v {
    Label(v0) => @base.tag("Label", [@base.jstr(v0)])
    Deref(v0) => @base.tag("Deref", [@base.jstr(v0)])
    SAcc(v0) => @base.tag("SAcc", [valueexpr_json(v0)])
    HAcc(v0, v1) => @base.tag("HAcc", [valueexpr_json(v0), valueexpr_json(v1)])
  }
}

///|
fn decode_placeexpr(j : Json) -> PlaceExpr raise @base.Fault {
  match @base.tag_name(j) {
    "Label" => Label(@base.identifier(@base.string(@base.tag_arg(j, 0))))
    "Deref" => Deref(@base.identifier(@base.string(@base.tag_arg(j, 0))))
    "SAcc" => SAcc(decode_valueexpr(@base.tag_arg(j, 0)))
    "HAcc" =>
      HAcc(
        decode_valueexpr(@base.tag_arg(j, 0)),
        decode_valueexpr(@base.tag_arg(j, 1)),
      )
    _ => @base.error("Invalid flow IR snapshot")
  }
}

///|
fn cond_json(v : Cond) -> Json {
  match v {
    Lt(v0, v1) => @base.tag("Lt", [valueexpr_json(v0), valueexpr_json(v1)])
    Eq(v0, v1) => @base.tag("Eq", [valueexpr_json(v0), valueexpr_json(v1)])
    Gt(v0, v1) => @base.tag("Gt", [valueexpr_json(v0), valueexpr_json(v1)])
  }
}

///|
fn decode_cond(j : Json) -> Cond raise @base.Fault {
  match @base.tag_name(j) {
    "Lt" =>
      Lt(
        decode_valueexpr(@base.tag_arg(j, 0)),
        decode_valueexpr(@base.tag_arg(j, 1)),
      )
    "Eq" =>
      Eq(
        decode_valueexpr(@base.tag_arg(j, 0)),
        decode_valueexpr(@base.tag_arg(j, 1)),
      )
    "Gt" =>
      Gt(
        decode_valueexpr(@base.tag_arg(j, 0)),
        decode_valueexpr(@base.tag_arg(j, 1)),
      )
    _ => @base.error("Invalid flow IR snapshot")
  }
}

///|
fn stmt_json(v : Stmt) -> Json {
  match v {
    Nop => @base.tag("Nop", [])
    Load(v0, v1) => @base.tag("Load", [@base.jstr(v0), placeexpr_json(v1)])
    Assign(v0, v1) => @base.tag("Assign", [@base.jstr(v0), valueexpr_json(v1)])
    BinOp(v0, v1, v2, v3) =>
      @base.tag("BinOp", [
        @base.jstr(v0),
        valueexpr_json(v1),
        binop_json(v2),
        valueexpr_json(v3),
      ])
    Store(v0, v1) =>
      @base.tag("Store", [placeexpr_json(v0), valueexpr_json(v1)])
    Pop(v0) => @base.tag("Pop", [@base.jstr(v0)])
    Push(v0) => @base.tag("Push", [valueexpr_json(v0)])
    LGet(v0) => @base.tag("LGet", [@base.jstr(v0)])
    HAlloc(v0, v1) => @base.tag("HAlloc", [valueexpr_json(v0), @base.jstr(v1)])
    HFree(v0) => @base.tag("HFree", [valueexpr_json(v0)])
    Print(v0) => @base.tag("Print", [@base.jstr(v0)])
    Input(v0) => @base.tag("Input", [placeexpr_json(v0)])
  }
}

///|
fn decode_stmt(j : Json) -> Stmt raise @base.Fault {
  match @base.tag_name(j) {
    "Nop" => Nop
    "Load" =>
      Load(
        @base.identifier(@base.string(@base.tag_arg(j, 0))),
        decode_placeexpr(@base.tag_arg(j, 1)),
      )
    "Assign" =>
      Assign(
        @base.identifier(@base.string(@base.tag_arg(j, 0))),
        decode_valueexpr(@base.tag_arg(j, 1)),
      )
    "BinOp" =>
      BinOp(
        @base.identifier(@base.string(@base.tag_arg(j, 0))),
        decode_valueexpr(@base.tag_arg(j, 1)),
        decode_binop(@base.tag_arg(j, 2)),
        decode_valueexpr(@base.tag_arg(j, 3)),
      )
    "Store" =>
      Store(
        decode_placeexpr(@base.tag_arg(j, 0)),
        decode_valueexpr(@base.tag_arg(j, 1)),
      )
    "Pop" => Pop(@base.identifier(@base.string(@base.tag_arg(j, 0))))
    "Push" => Push(decode_valueexpr(@base.tag_arg(j, 0)))
    "LGet" => LGet(@base.identifier(@base.string(@base.tag_arg(j, 0))))
    "HAlloc" =>
      HAlloc(
        decode_valueexpr(@base.tag_arg(j, 0)),
        @base.identifier(@base.string(@base.tag_arg(j, 1))),
      )
    "HFree" => HFree(decode_valueexpr(@base.tag_arg(j, 0)))
    "Print" => Print(@base.identifier(@base.string(@base.tag_arg(j, 0))))
    "Input" => Input(decode_placeexpr(@base.tag_arg(j, 0)))
    _ => @base.error("Invalid flow IR snapshot")
  }
}

///|
fn cont_json(v : Cont) -> Json {
  match v {
    Go(v0, v1) => @base.tag("Go", [jumps_json(v0), valueexpr_json(v1)])
    Enter(v0, v1) => @base.tag("Enter", [jumps_json(v0), valueexpr_json(v1)])
    Halt(v0) => @base.tag("Halt", [jumps_json(v0)])
  }
}

///|
fn decode_cont(j : Json) -> Cont raise @base.Fault {
  match @base.tag_name(j) {
    "Go" =>
      Go(
        decode_jumps(@base.tag_arg(j, 0)),
        decode_valueexpr(@base.tag_arg(j, 1)),
      )
    "Enter" =>
      Enter(
        decode_jumps(@base.tag_arg(j, 0)),
        decode_valueexpr(@base.tag_arg(j, 1)),
      )
    "Halt" => Halt(decode_jumps(@base.tag_arg(j, 0)))
    _ => @base.error("Invalid flow IR snapshot")
  }
}

///|
fn target_json(v : Target) -> Json {
  match v {
    RegionTarget(v0) => @base.tag("Region_target", [@base.jint(v0)])
    BlockTarget(v0) => @base.tag("Block_target", [@base.jint(v0)])
  }
}

///|
fn decode_target(j : Json) -> Target raise @base.Fault {
  match @base.tag_name(j) {
    "Region_target" => RegionTarget(id(@base.tag_arg(j, 0)))
    "Block_target" => BlockTarget(id(@base.tag_arg(j, 0)))
    _ => @base.error("Invalid flow IR snapshot")
  }
}

///|
fn id(j : Json) -> Int32 raise @base.Fault {
  let id = @base.int(j)
  @base.require(id >= 0 && id < 2147483647, "Invalid resolved ID")
  id
}

///|
fn jumps_json(jumps : Array[JumpIf]) -> Json {
  @base.jarray(
    jumps.map(j => {
      @base.object([
        ("cond", cond_json(j.cond)),
        ("target", valueexpr_json(j.target)),
      ])
    }),
  )
}

///|
fn decode_jumps(j : Json) -> Array[JumpIf] raise @base.Fault {
  @base.array(j).map(j => {
    cond: decode_cond(@base.field(j, "cond")),
    target: decode_valueexpr(@base.field(j, "target")),
  })
}

///|
fn block_json(b : Block) -> Json {
  @base.object([
    ("label", @base.jstr(b.label)),
    ("stmts", @base.jarray(b.stmts.map(stmt_json))),
    ("cont", cont_json(b.cont)),
  ])
}

///|
fn decode_block(j : Json) -> Block raise @base.Fault {
  {
    label: @base.identifier(@base.string(@base.field(j, "label"))),
    stmts: @base.array(@base.field(j, "stmts")).map(decode_stmt),
    cont: decode_cont(@base.field(j, "cont")),
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
    (
      "regions",
      @base.jarray(
        c.regions.map(r => {
          @base.object([
            ("label", @base.jstr(r.label)),
            ("blocks", @base.jarray(r.blocks.map(block_json))),
          ])
        }),
      ),
    ),
  ])
}

///|
fn decode_code(j : Json) -> Code raise @base.Fault {
  {
    statics: @base.array(@base.field(j, "statics")).map(j => {
      label: @base.identifier(@base.string(@base.field(j, "label"))),
      value: @base.json_nat(@base.field(j, "value")),
    }),
    regions: @base.array(@base.field(j, "regions")).map(j => {
      label: @base.identifier(@base.string(@base.field(j, "label"))),
      blocks: @base.array(@base.field(j, "blocks")).map(decode_block),
    }),
  }
}

///|
fn[D] location_json(location : Location[D], data : (D) -> Json) -> Json {
  match location {
    Static(d) => @base.tag("Static", [data(d)])
    Stack(i) => @base.tag("Stack", [@base.jint(i)])
    Heap(h, i) => @base.tag("Heap", [@base.jint(h), @base.jint(i)])
  }
}

///|
fn[D] decode_location(
  j : Json,
  data : (Json) -> D raise @base.Fault,
) -> Location[D] raise @base.Fault {
  match @base.tag_name(j) {
    "Static" => Static(data(@base.tag_arg(j, 0)))
    "Stack" => Stack(@base.int(@base.tag_arg(j, 0)))
    "Heap" => Heap(id(@base.tag_arg(j, 0)), @base.int(@base.tag_arg(j, 1)))
    _ => @base.error("Invalid memory location")
  }
}

///|
fn runtime_json(v : RuntimeValue) -> Json {
  match v {
    Num(n) => @base.tag("Num", [@base.jstr(n.to_string())])
    Code_label(t) => @base.tag("Code_label", [target_json(t)])
    Key(k) => @base.tag("Key", [location_json(k, @base.jint)])
  }
}

///|
fn decode_runtime(j : Json) -> RuntimeValue raise @base.Fault {
  match @base.tag_name(j) {
    "Num" => Num(@base.json_nat(@base.tag_arg(j, 0)))
    "Code_label" => Code_label(decode_target(@base.tag_arg(j, 0)))
    "Key" => Key(decode_location(@base.tag_arg(j, 0), id))
    _ => @base.error("Invalid runtime value")
  }
}

///|
fn resolution_json(r : Resolution) -> Json {
  @base.object([
    ("registers", @data.env_json(r.registers, @base.jint)),
    ("data", @data.env_json(r.data, @base.jint)),
    ("regions", @data.env_json(r.regions, @base.jint)),
    ("blocks", @base.jarray(r.blocks.map(xs => @data.env_json(xs, @base.jint)))),
    (
      "label_names",
      @base.jarray(
        r.label_names.map(p => @base.jarray([target_json(p.0), @base.jstr(p.1)])),
      ),
    ),
  ])
}

///|
fn decode_resolution(j : Json) -> Resolution raise @base.Fault {
  {
    registers: @data.decode_env(@base.field(j, "registers"), id),
    data: @data.decode_env(@base.field(j, "data"), id),
    regions: @data.decode_env(@base.field(j, "regions"), id),
    blocks: @base.array(@base.field(j, "blocks")).map(j => {
      @data.decode_env(j, id)
    }),
    label_names: @base.array(@base.field(j, "label_names")).map(j => {
      let a = @base.array(j)
      @base.require(a.length() == 2, "Invalid label name")
      (decode_target(a[0]), @base.identifier(@base.string(a[1])))
    }),
  }
}

///|
fn[T] id_pairs_json(entries : Array[(Int32, T)], encode : (T) -> Json) -> Json {
  @base.jarray(entries.map(p => @base.jarray([@base.jint(p.0), encode(p.1)])))
}

///|
fn[T] decode_id_pairs(
  j : Json,
  decode : (Json) -> T raise @base.Fault,
) -> Array[(Int32, T)] raise @base.Fault {
  @base.array(j).map(j => {
    let a = @base.array(j)
    @base.require(a.length() == 2, "Invalid ID binding")
    (id(a[0]), decode(a[1]))
  })
}

///|
fn encode(s : State) -> Json {
  @base.object([
    ("code", code_json(s.code)),
    ("current_region", @base.jint(s.current_region)),
    ("current_block", @base.jint(s.current_block)),
    ("current_line", @base.jint(s.current_line)),
    ("resolution", resolution_json(s.resolution)),
    ("vregs", id_pairs_json(s.vregs, runtime_json)),
    ("static_mem", id_pairs_json(s.static_mem, runtime_json)),
    ("stack", @base.jarray(s.stack.map(runtime_json))),
    ("heap", id_pairs_json(s.heap, vs => @base.jarray(vs.map(runtime_json)))),
    ("next_handle", @base.jint(s.next_handle)),
    ("halted", @base.jbool(s.halted)),
  ])
}

///|
fn decode(j : Json) -> State raise @base.Fault {
  let code = decode_code(@base.field(j, "code"))
  validate(code)
  let resolution = decode_resolution(@base.field(j, "resolution"))
  @base.require(
    resolution == build_resolution(code),
    "Inconsistent name resolution in snapshot",
  )
  let s = {
    code,
    resolution,
    current_region: @base.int(@base.field(j, "current_region")),
    current_block: @base.int(@base.field(j, "current_block")),
    current_line: @base.int(@base.field(j, "current_line")),
    vregs: decode_id_pairs(@base.field(j, "vregs"), decode_runtime),
    static_mem: decode_id_pairs(@base.field(j, "static_mem"), decode_runtime),
    stack: @base.array(@base.field(j, "stack")).map(decode_runtime),
    heap: decode_id_pairs(@base.field(j, "heap"), j => {
      @base.array(j).map(decode_runtime)
    }),
    next_handle: @base.int(@base.field(j, "next_handle")),
    halted: @base.boolean(@base.field(j, "halted")),
  }
  @base.require(s.next_handle >= 0, "Expected nonnegative ID supply")
  fn check(v : RuntimeValue) -> Unit raise @base.Fault {
    match v {
      Num(_) => ()
      Code_label(t) => ignore(target_name(resolution, t))
      Key(Static(id)) => ignore(name_of_id(resolution.data, id))
      Key(Heap(h, _)) =>
        @base.require(h < s.next_handle, "ID supply would reuse an existing ID")
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
    @base.require(p.0 < s.next_handle, "ID supply would reuse an existing ID")
    for v in p.1 {
      check(v)
    }
  }
  s
}
