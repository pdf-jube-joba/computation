///|
fn atom_json(v : Atom) -> Json {
  match v {
    Var(v0) => @utils.tag("Var", [@utils.jstr(v0)])
    Lit(v0) => @utils.tag("Lit", [@utils.jstr(v0.to_string())])
  }
}

///|
fn decode_atom(j : Json) -> Atom raise @utils.Fault {
  match @utils.tag_name(j) {
    "Var" => Var(@utils.identifier(@utils.string(@utils.tag_arg(j, 0))))
    "Lit" => Lit(@utils.json_nat(@utils.tag_arg(j, 0)))
    _ => @utils.error("Invalid STG Atom")
  }
}

///|
fn updateflag_json(v : UpdateFlag) -> Json {
  match v {
    Updatable => @utils.tag("Updatable", [])
    Reentrant => @utils.tag("Reentrant", [])
  }
}

///|
fn decode_updateflag(j : Json) -> UpdateFlag raise @utils.Fault {
  match @utils.tag_name(j) {
    "Updatable" => Updatable
    "Reentrant" => Reentrant
    _ => @utils.error("Invalid STG UpdateFlag")
  }
}

///|
fn prim_json(v : Prim) -> Json {
  match v {
    Add => @utils.tag("Add", [])
    Sub => @utils.tag("Sub", [])
    Mul => @utils.tag("Mul", [])
    Eq => @utils.tag("Eq", [])
    Lt => @utils.tag("Lt", [])
  }
}

///|
fn decode_prim(j : Json) -> Prim raise @utils.Fault {
  match @utils.tag_name(j) {
    "Add" => Add
    "Sub" => Sub
    "Mul" => Mul
    "Eq" => Eq
    "Lt" => Lt
    _ => @utils.error("Invalid STG Prim")
  }
}

///|
fn expr_json(v : Expr) -> Json {
  match v {
    App(v0, v1) => @utils.tag("App", [@utils.jstr(v0), atoms_json(v1)])
    Con(v0, v1) => @utils.tag("Con", [@utils.jstr(v0), atoms_json(v1)])
    Nat(v0) => @utils.tag("Nat", [@utils.jstr(v0.to_string())])
    Prim(v0, v1, v2) =>
      @utils.tag("Prim", [prim_json(v0), atom_json(v1), atom_json(v2)])
    Let(v0, v1, v2) =>
      @utils.tag("Let", [@utils.jbool(v0), code_json(v1), expr_json(v2)])
    Case(v0, v1) => @utils.tag("Case", [expr_json(v0), alts_json(v1)])
  }
}

///|
fn decode_expr(j : Json) -> Expr raise @utils.Fault {
  match @utils.tag_name(j) {
    "App" =>
      App(
        @utils.identifier(@utils.string(@utils.tag_arg(j, 0))),
        decode_atoms(@utils.tag_arg(j, 1)),
      )
    "Con" =>
      Con(
        @utils.identifier(@utils.string(@utils.tag_arg(j, 0))),
        decode_atoms(@utils.tag_arg(j, 1)),
      )
    "Nat" => Nat(@utils.json_nat(@utils.tag_arg(j, 0)))
    "Prim" =>
      Prim(
        decode_prim(@utils.tag_arg(j, 0)),
        decode_atom(@utils.tag_arg(j, 1)),
        decode_atom(@utils.tag_arg(j, 2)),
      )
    "Let" =>
      Let(
        @utils.boolean(@utils.tag_arg(j, 0)),
        decode_code(@utils.tag_arg(j, 1)),
        decode_expr(@utils.tag_arg(j, 2)),
      )
    "Case" =>
      Case(decode_expr(@utils.tag_arg(j, 0)), decode_alts(@utils.tag_arg(j, 1)))
    _ => @utils.error("Invalid STG Expr")
  }
}

///|
fn pattern_json(v : Pattern) -> Json {
  match v {
    Constructor(v0, v1) =>
      @utils.tag("Constructor", [
        @utils.jstr(v0),
        @utils.jarray(v1.map(@utils.jstr)),
      ])
    Literal(v0) => @utils.tag("Literal", [@utils.jstr(v0.to_string())])
    Default(v0) =>
      @utils.tag("Default", [@utils.optional_json(v0, @utils.jstr)])
  }
}

///|
fn decode_pattern(j : Json) -> Pattern raise @utils.Fault {
  match @utils.tag_name(j) {
    "Constructor" =>
      Constructor(
        @utils.identifier(@utils.string(@utils.tag_arg(j, 0))),
        @utils.array(@utils.tag_arg(j, 1)).map(j => {
          @utils.identifier(@utils.string(j))
        }),
      )
    "Literal" => Literal(@utils.json_nat(@utils.tag_arg(j, 0)))
    "Default" =>
      Default(
        match @utils.tag_arg(j, 0) {
          Null => None
          other => Some(@utils.identifier(@utils.string(other)))
        },
      )
    _ => @utils.error("Invalid STG Pattern")
  }
}

///|
fn value_json(v : Value) -> Json {
  match v {
    Addr(v0) => @utils.tag("Addr", [@utils.jint(v0)])
    Integer(v0) => @utils.tag("Int", [@utils.jstr(v0.to_string())])
  }
}

///|
fn decode_value(j : Json) -> Value raise @utils.Fault {
  match @utils.tag_name(j) {
    "Addr" => Addr(@utils.int(@utils.tag_arg(j, 0)))
    "Int" => Integer(@utils.json_nat(@utils.tag_arg(j, 0)))
    _ => @utils.error("Invalid STG Value")
  }
}

///|
fn resultvalue_json(v : ResultValue) -> Json {
  match v {
    Number(v0) => @utils.tag("Number", [@utils.jstr(v0.to_string())])
    Data(v0, v1) => @utils.tag("Data", [@utils.jstr(v0), values_json(v1)])
    Function(v0) => @utils.tag("Function", [@utils.jint(v0)])
  }
}

///|
fn decode_resultvalue(j : Json) -> ResultValue raise @utils.Fault {
  match @utils.tag_name(j) {
    "Number" => Number(@utils.json_nat(@utils.tag_arg(j, 0)))
    "Data" =>
      Data(
        @utils.identifier(@utils.string(@utils.tag_arg(j, 0))),
        decode_values(@utils.tag_arg(j, 1)),
      )
    "Function" => Function(@utils.int(@utils.tag_arg(j, 0)))
    _ => @utils.error("Invalid STG ResultValue")
  }
}

///|
fn closure_json(v : Closure) -> Json {
  match v {
    Closure(v0, v1) => @utils.tag("Closure", [lambda_json(v0), env_json(v1)])
    Blackhole => @utils.tag("Blackhole", [])
    Evaluated(v0) => @utils.tag("Evaluated", [resultvalue_json(v0)])
    Pap(v0, v1) => @utils.tag("Pap", [@utils.jint(v0), values_json(v1)])
  }
}

///|
fn decode_closure(j : Json) -> Closure raise @utils.Fault {
  match @utils.tag_name(j) {
    "Closure" =>
      Closure(
        decode_lambda(@utils.tag_arg(j, 0)),
        decode_env(@utils.tag_arg(j, 1)),
      )
    "Blackhole" => Blackhole
    "Evaluated" => Evaluated(decode_resultvalue(@utils.tag_arg(j, 0)))
    "Pap" =>
      Pap(@utils.int(@utils.tag_arg(j, 0)), decode_values(@utils.tag_arg(j, 1)))
    _ => @utils.error("Invalid STG Closure")
  }
}

///|
fn control_json(v : Control) -> Json {
  match v {
    Eval(v0, v1) => @utils.tag("Eval", [expr_json(v0), env_json(v1)])
    Enter(v0) => @utils.tag("Enter", [value_json(v0)])
    Return(v0) => @utils.tag("Return", [resultvalue_json(v0)])
  }
}

///|
fn decode_control(j : Json) -> Control raise @utils.Fault {
  match @utils.tag_name(j) {
    "Eval" =>
      Eval(decode_expr(@utils.tag_arg(j, 0)), decode_env(@utils.tag_arg(j, 1)))
    "Enter" => Enter(decode_value(@utils.tag_arg(j, 0)))
    "Return" => Return(decode_resultvalue(@utils.tag_arg(j, 0)))
    _ => @utils.error("Invalid STG Control")
  }
}

///|
fn frame_json(v : Frame) -> Json {
  match v {
    Match(v0, v1, v2) =>
      @utils.tag("Match", [alts_json(v0), env_json(v1), values_json(v2)])
    Update(v0, v1) => @utils.tag("Update", [@utils.jint(v0), values_json(v1)])
  }
}

///|
fn decode_frame(j : Json) -> Frame raise @utils.Fault {
  match @utils.tag_name(j) {
    "Match" =>
      Match(
        decode_alts(@utils.tag_arg(j, 0)),
        decode_env(@utils.tag_arg(j, 1)),
        decode_values(@utils.tag_arg(j, 2)),
      )
    "Update" =>
      Update(
        @utils.int(@utils.tag_arg(j, 0)),
        decode_values(@utils.tag_arg(j, 1)),
      )
    _ => @utils.error("Invalid STG Frame")
  }
}

///|
fn atoms_json(v : Array[Atom]) -> Json {
  @utils.jarray(v.map(atom_json))
}

///|
fn decode_atoms(j : Json) -> Array[Atom] raise @utils.Fault {
  @utils.array(j).map(decode_atom)
}

///|
fn values_json(v : Array[Value]) -> Json {
  @utils.jarray(v.map(value_json))
}

///|
fn decode_values(j : Json) -> Array[Value] raise @utils.Fault {
  @utils.array(j).map(decode_value)
}

///|
fn code_json(code : Bindings) -> Json {
  @utils.jarray(
    code.map(b => @utils.jarray([@utils.jstr(b.0), lambda_json(b.1)])),
  )
}

///|
fn decode_code(j : Json) -> Bindings raise @utils.Fault {
  @utils.array(j).map(j => {
    let a = @utils.array(j)
    @utils.require(a.length() == 2, "Invalid STG binding")
    (@utils.identifier(@utils.string(a[0])), decode_lambda(a[1]))
  })
}

///|
fn alts_json(alts : Array[(Pattern, Expr)]) -> Json {
  @utils.jarray(
    alts.map(a => @utils.jarray([pattern_json(a.0), expr_json(a.1)])),
  )
}

///|
fn decode_alts(j : Json) -> Array[(Pattern, Expr)] raise @utils.Fault {
  @utils.array(j).map(j => {
    let a = @utils.array(j)
    @utils.require(a.length() == 2, "Invalid STG alternative")
    (decode_pattern(a[0]), decode_expr(a[1]))
  })
}

///|
fn lambda_json(lf : Lambda) -> Json {
  @utils.object([
    ("free", @utils.jarray(lf.free.map(@utils.jstr))),
    ("update", updateflag_json(lf.update)),
    ("args", @utils.jarray(lf.args.map(@utils.jstr))),
    ("body", expr_json(lf.body)),
  ])
}

///|
fn decode_lambda(j : Json) -> Lambda raise @utils.Fault {
  {
    free: @utils.array(@utils.field(j, "free")).map(j => {
      @utils.identifier(@utils.string(j))
    }),
    update: decode_updateflag(@utils.field(j, "update")),
    args: @utils.array(@utils.field(j, "args")).map(j => {
      @utils.identifier(@utils.string(j))
    }),
    body: decode_expr(@utils.field(j, "body")),
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

///|
fn encode(s : State) -> Json {
  @utils.object([
    ("code", code_json(s.code)),
    ("globals", env_json(s.globals)),
    (
      "heap",
      @utils.jarray(
        s.heap.map(p => @utils.jarray([@utils.jint(p.0), closure_json(p.1)])),
      ),
    ),
    ("next_addr", @utils.jint(s.next_addr)),
    ("control", control_json(s.control)),
    ("arguments", values_json(s.arguments)),
    ("stack", @utils.jarray(s.stack.map(frame_json))),
  ])
}

///|
fn decode(j : Json) -> State raise @utils.Fault {
  let s = {
    code: decode_code(@utils.field(j, "code")),
    globals: decode_env(@utils.field(j, "globals")),
    heap: @utils.array(@utils.field(j, "heap")).map(j => {
      let a = @utils.array(j)
      @utils.require(a.length() == 2, "Invalid heap cell")
      (@utils.int(a[0]), decode_closure(a[1]))
    }),
    next_addr: @utils.int(@utils.field(j, "next_addr")),
    control: decode_control(@utils.field(j, "control")),
    arguments: decode_values(@utils.field(j, "arguments")),
    stack: @utils.array(@utils.field(j, "stack")).map(decode_frame),
  }
  validate_state(s)
  s
}
