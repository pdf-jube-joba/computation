///|
fn atom_json(v : Atom) -> Json {
  match v {
    Var(v0) => @base.tag("Var", [@base.jstr(v0)])
    Lit(v0) => @base.tag("Lit", [@base.jstr(v0.to_string())])
  }
}

///|
fn decode_atom(j : Json) -> Atom raise @base.Fault {
  match @base.tag_name(j) {
    "Var" => Var(@base.identifier(@base.string(@base.tag_arg(j, 0))))
    "Lit" => Lit(@base.json_nat(@base.tag_arg(j, 0)))
    _ => @base.error("Invalid STG Atom")
  }
}

///|
fn updateflag_json(v : UpdateFlag) -> Json {
  match v {
    Updatable => @base.tag("Updatable", [])
    Reentrant => @base.tag("Reentrant", [])
  }
}

///|
fn decode_updateflag(j : Json) -> UpdateFlag raise @base.Fault {
  match @base.tag_name(j) {
    "Updatable" => Updatable
    "Reentrant" => Reentrant
    _ => @base.error("Invalid STG UpdateFlag")
  }
}

///|
fn prim_json(v : Prim) -> Json {
  match v {
    Add => @base.tag("Add", [])
    Sub => @base.tag("Sub", [])
    Mul => @base.tag("Mul", [])
    Eq => @base.tag("Eq", [])
    Lt => @base.tag("Lt", [])
  }
}

///|
fn decode_prim(j : Json) -> Prim raise @base.Fault {
  match @base.tag_name(j) {
    "Add" => Add
    "Sub" => Sub
    "Mul" => Mul
    "Eq" => Eq
    "Lt" => Lt
    _ => @base.error("Invalid STG Prim")
  }
}

///|
fn expr_json(v : Expr) -> Json {
  match v {
    App(v0, v1) => @base.tag("App", [@base.jstr(v0), atoms_json(v1)])
    Con(v0, v1) => @base.tag("Con", [@base.jstr(v0), atoms_json(v1)])
    Nat(v0) => @base.tag("Nat", [@base.jstr(v0.to_string())])
    Prim(v0, v1, v2) =>
      @base.tag("Prim", [prim_json(v0), atom_json(v1), atom_json(v2)])
    Let(v0, v1, v2) =>
      @base.tag("Let", [@base.jbool(v0), code_json(v1), expr_json(v2)])
    Case(v0, v1) => @base.tag("Case", [expr_json(v0), alts_json(v1)])
  }
}

///|
fn decode_expr(j : Json) -> Expr raise @base.Fault {
  match @base.tag_name(j) {
    "App" =>
      App(
        @base.identifier(@base.string(@base.tag_arg(j, 0))),
        decode_atoms(@base.tag_arg(j, 1)),
      )
    "Con" =>
      Con(
        @base.identifier(@base.string(@base.tag_arg(j, 0))),
        decode_atoms(@base.tag_arg(j, 1)),
      )
    "Nat" => Nat(@base.json_nat(@base.tag_arg(j, 0)))
    "Prim" =>
      Prim(
        decode_prim(@base.tag_arg(j, 0)),
        decode_atom(@base.tag_arg(j, 1)),
        decode_atom(@base.tag_arg(j, 2)),
      )
    "Let" =>
      Let(
        @base.boolean(@base.tag_arg(j, 0)),
        decode_code(@base.tag_arg(j, 1)),
        decode_expr(@base.tag_arg(j, 2)),
      )
    "Case" =>
      Case(decode_expr(@base.tag_arg(j, 0)), decode_alts(@base.tag_arg(j, 1)))
    _ => @base.error("Invalid STG Expr")
  }
}

///|
fn pattern_json(v : Pattern) -> Json {
  match v {
    Constructor(v0, v1) =>
      @base.tag("Constructor", [
        @base.jstr(v0),
        @base.jarray(v1.map(@base.jstr)),
      ])
    Literal(v0) => @base.tag("Literal", [@base.jstr(v0.to_string())])
    Default(v0) => @base.tag("Default", [@base.optional_json(v0, @base.jstr)])
  }
}

///|
fn decode_pattern(j : Json) -> Pattern raise @base.Fault {
  match @base.tag_name(j) {
    "Constructor" =>
      Constructor(
        @base.identifier(@base.string(@base.tag_arg(j, 0))),
        @base.array(@base.tag_arg(j, 1)).map(j => {
          @base.identifier(@base.string(j))
        }),
      )
    "Literal" => Literal(@base.json_nat(@base.tag_arg(j, 0)))
    "Default" =>
      Default(
        match @base.tag_arg(j, 0) {
          Null => None
          other => Some(@base.identifier(@base.string(other)))
        },
      )
    _ => @base.error("Invalid STG Pattern")
  }
}

///|
fn value_json(v : Value) -> Json {
  match v {
    Addr(v0) => @base.tag("Addr", [@base.jint(v0)])
    Integer(v0) => @base.tag("Int", [@base.jstr(v0.to_string())])
  }
}

///|
fn decode_value(j : Json) -> Value raise @base.Fault {
  match @base.tag_name(j) {
    "Addr" => Addr(@base.int(@base.tag_arg(j, 0)))
    "Int" => Integer(@base.json_nat(@base.tag_arg(j, 0)))
    _ => @base.error("Invalid STG Value")
  }
}

///|
fn resultvalue_json(v : ResultValue) -> Json {
  match v {
    Number(v0) => @base.tag("Number", [@base.jstr(v0.to_string())])
    Data(v0, v1) => @base.tag("Data", [@base.jstr(v0), values_json(v1)])
    Function(v0) => @base.tag("Function", [@base.jint(v0)])
  }
}

///|
fn decode_resultvalue(j : Json) -> ResultValue raise @base.Fault {
  match @base.tag_name(j) {
    "Number" => Number(@base.json_nat(@base.tag_arg(j, 0)))
    "Data" =>
      Data(
        @base.identifier(@base.string(@base.tag_arg(j, 0))),
        decode_values(@base.tag_arg(j, 1)),
      )
    "Function" => Function(@base.int(@base.tag_arg(j, 0)))
    _ => @base.error("Invalid STG ResultValue")
  }
}

///|
fn closure_json(v : Closure) -> Json {
  match v {
    Closure(v0, v1) => @base.tag("Closure", [lambda_json(v0), env_json(v1)])
    Blackhole => @base.tag("Blackhole", [])
    Evaluated(v0) => @base.tag("Evaluated", [resultvalue_json(v0)])
    Pap(v0, v1) => @base.tag("Pap", [@base.jint(v0), values_json(v1)])
  }
}

///|
fn decode_closure(j : Json) -> Closure raise @base.Fault {
  match @base.tag_name(j) {
    "Closure" =>
      Closure(
        decode_lambda(@base.tag_arg(j, 0)),
        decode_env(@base.tag_arg(j, 1)),
      )
    "Blackhole" => Blackhole
    "Evaluated" => Evaluated(decode_resultvalue(@base.tag_arg(j, 0)))
    "Pap" =>
      Pap(@base.int(@base.tag_arg(j, 0)), decode_values(@base.tag_arg(j, 1)))
    _ => @base.error("Invalid STG Closure")
  }
}

///|
fn control_json(v : Control) -> Json {
  match v {
    Eval(v0, v1) => @base.tag("Eval", [expr_json(v0), env_json(v1)])
    Enter(v0) => @base.tag("Enter", [value_json(v0)])
    Return(v0) => @base.tag("Return", [resultvalue_json(v0)])
  }
}

///|
fn decode_control(j : Json) -> Control raise @base.Fault {
  match @base.tag_name(j) {
    "Eval" =>
      Eval(decode_expr(@base.tag_arg(j, 0)), decode_env(@base.tag_arg(j, 1)))
    "Enter" => Enter(decode_value(@base.tag_arg(j, 0)))
    "Return" => Return(decode_resultvalue(@base.tag_arg(j, 0)))
    _ => @base.error("Invalid STG Control")
  }
}

///|
fn frame_json(v : Frame) -> Json {
  match v {
    Match(v0, v1, v2) =>
      @base.tag("Match", [alts_json(v0), env_json(v1), values_json(v2)])
    Update(v0, v1) => @base.tag("Update", [@base.jint(v0), values_json(v1)])
  }
}

///|
fn decode_frame(j : Json) -> Frame raise @base.Fault {
  match @base.tag_name(j) {
    "Match" =>
      Match(
        decode_alts(@base.tag_arg(j, 0)),
        decode_env(@base.tag_arg(j, 1)),
        decode_values(@base.tag_arg(j, 2)),
      )
    "Update" =>
      Update(@base.int(@base.tag_arg(j, 0)), decode_values(@base.tag_arg(j, 1)))
    _ => @base.error("Invalid STG Frame")
  }
}

///|
fn atoms_json(v : Array[Atom]) -> Json {
  @base.jarray(v.map(atom_json))
}

///|
fn decode_atoms(j : Json) -> Array[Atom] raise @base.Fault {
  @base.array(j).map(decode_atom)
}

///|
fn values_json(v : Array[Value]) -> Json {
  @base.jarray(v.map(value_json))
}

///|
fn decode_values(j : Json) -> Array[Value] raise @base.Fault {
  @base.array(j).map(decode_value)
}

///|
fn code_json(code : Code) -> Json {
  @base.jarray(code.map(b => @base.jarray([@base.jstr(b.0), lambda_json(b.1)])))
}

///|
fn decode_code(j : Json) -> Code raise @base.Fault {
  @base.array(j).map(j => {
    let a = @base.array(j)
    @base.require(a.length() == 2, "Invalid STG binding")
    (@base.identifier(@base.string(a[0])), decode_lambda(a[1]))
  })
}

///|
fn alts_json(alts : Array[(Pattern, Expr)]) -> Json {
  @base.jarray(alts.map(a => @base.jarray([pattern_json(a.0), expr_json(a.1)])))
}

///|
fn decode_alts(j : Json) -> Array[(Pattern, Expr)] raise @base.Fault {
  @base.array(j).map(j => {
    let a = @base.array(j)
    @base.require(a.length() == 2, "Invalid STG alternative")
    (decode_pattern(a[0]), decode_expr(a[1]))
  })
}

///|
fn lambda_json(lf : Lambda) -> Json {
  @base.object([
    ("free", @base.jarray(lf.free.map(@base.jstr))),
    ("update", updateflag_json(lf.update)),
    ("args", @base.jarray(lf.args.map(@base.jstr))),
    ("body", expr_json(lf.body)),
  ])
}

///|
fn decode_lambda(j : Json) -> Lambda raise @base.Fault {
  {
    free: @base.array(@base.field(j, "free")).map(j => {
      @base.identifier(@base.string(j))
    }),
    update: decode_updateflag(@base.field(j, "update")),
    args: @base.array(@base.field(j, "args")).map(j => {
      @base.identifier(@base.string(j))
    }),
    body: decode_expr(@base.field(j, "body")),
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

///|
fn encode(s : State) -> Json {
  @base.object([
    ("code", code_json(s.code)),
    ("globals", env_json(s.globals)),
    (
      "heap",
      @base.jarray(
        s.heap.map(p => @base.jarray([@base.jint(p.0), closure_json(p.1)])),
      ),
    ),
    ("next_addr", @base.jint(s.next_addr)),
    ("control", control_json(s.control)),
    ("arguments", values_json(s.arguments)),
    ("stack", @base.jarray(s.stack.map(frame_json))),
  ])
}

///|
fn decode(j : Json) -> State raise @base.Fault {
  let s = {
    code: decode_code(@base.field(j, "code")),
    globals: decode_env(@base.field(j, "globals")),
    heap: @base.array(@base.field(j, "heap")).map(j => {
      let a = @base.array(j)
      @base.require(a.length() == 2, "Invalid heap cell")
      (@base.int(a[0]), decode_closure(a[1]))
    }),
    next_addr: @base.int(@base.field(j, "next_addr")),
    control: decode_control(@base.field(j, "control")),
    arguments: decode_values(@base.field(j, "arguments")),
    stack: @base.array(@base.field(j, "stack")).map(decode_frame),
  }
  validate_state(s)
  s
}
