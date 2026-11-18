///|
fn int64_json(n : Int64) -> Json {
  Json::number(n.to_double(), repr=n.to_string())
}

///|
fn read_int64(j : Json) -> Int64 raise @base.Fault {
  @string.parse_int64(j.stringify()) catch {
    _ => @base.error("Invalid int64")
  }
}

///|
fn json_binop(v : Binop) -> Json {
  match v {
    Add => @base.tag("Add", [])
    Sub => @base.tag("Sub", [])
  }
}

///|
fn read_binop(j : Json) -> Binop raise @base.Fault {
  let tag = @base.tag_name(j)
  let a = @base.array(j)[1:].to_owned()
  match tag {
    "Add" => {
      @base.require(a.length() == 0, "Invalid Add")
      Add
    }
    "Sub" => {
      @base.require(a.length() == 0, "Invalid Sub")
      Sub
    }
    _ => @base.error("Invalid Binop tag: " + tag)
  }
}

///|
fn json_place(v : Place) -> Json {
  match v {
    Var(v0) => @base.tag("Var", [@base.jstr(v0)])
    Deref(v0) => @base.tag("Deref", [json_expr(v0)])
  }
}

///|
fn read_place(j : Json) -> Place raise @base.Fault {
  let tag = @base.tag_name(j)
  let a = @base.array(j)[1:].to_owned()
  match tag {
    "Var" => {
      @base.require(a.length() == 1, "Invalid Var")
      Var(@base.string(a[0]))
    }
    "Deref" => {
      @base.require(a.length() == 1, "Invalid Deref")
      Deref(read_expr(a[0]))
    }
    _ => @base.error("Invalid Place tag: " + tag)
  }
}

///|
fn json_expr(v : Expr) -> Json {
  match v {
    Number(v0) => @base.tag("Number", [int64_json(v0)])
    NullPtr => @base.tag("NullPtr", [])
    BinOp(v0, v1, v2) =>
      @base.tag("BinOp", [json_expr(v0), json_binop(v1), json_expr(v2)])
    Load(v0) => @base.tag("Load", [json_place(v0)])
    Addr(v0) => @base.tag("Addr", [json_place(v0)])
  }
}

///|
fn read_expr(j : Json) -> Expr raise @base.Fault {
  let tag = @base.tag_name(j)
  let a = @base.array(j)[1:].to_owned()
  match tag {
    "Number" => {
      @base.require(a.length() == 1, "Invalid Number")
      Number(read_int64(a[0]))
    }
    "NullPtr" => {
      @base.require(a.length() == 0, "Invalid NullPtr")
      NullPtr
    }
    "BinOp" => {
      @base.require(a.length() == 3, "Invalid BinOp")
      BinOp(read_expr(a[0]), read_binop(a[1]), read_expr(a[2]))
    }
    "Load" => {
      @base.require(a.length() == 1, "Invalid Load")
      Load(read_place(a[0]))
    }
    "Addr" => {
      @base.require(a.length() == 1, "Invalid Addr")
      Addr(read_place(a[0]))
    }
    _ => @base.error("Invalid Expr tag: " + tag)
  }
}

///|
fn json_stmt(v : Stmt) -> Json {
  match v {
    Assign(v0, v1) => @base.tag("Assign", [json_place(v0), json_expr(v1)])
    Ifz(v0, v1, v2) =>
      @base.tag("Ifz", [json_expr(v0), json_stmt(v1), json_stmt(v2)])
    Call(v0, v1) =>
      @base.tag("Call", [
        @base.jstr(v0),
        @base.jarray(v1.map(x => json_expr(x))),
      ])
    Return => @base.tag("Return", [])
  }
}

///|
fn read_stmt(j : Json) -> Stmt raise @base.Fault {
  let tag = @base.tag_name(j)
  let a = @base.array(j)[1:].to_owned()
  match tag {
    "Assign" => {
      @base.require(a.length() == 2, "Invalid Assign")
      Assign(read_place(a[0]), read_expr(a[1]))
    }
    "Ifz" => {
      @base.require(a.length() == 3, "Invalid Ifz")
      Ifz(read_expr(a[0]), read_stmt(a[1]), read_stmt(a[2]))
    }
    "Call" => {
      @base.require(a.length() == 2, "Invalid Call")
      Call(@base.string(a[0]), @base.array(a[1]).map(x => read_expr(x)))
    }
    "Return" => {
      @base.require(a.length() == 0, "Invalid Return")
      Return
    }
    _ => @base.error("Invalid Stmt tag: " + tag)
  }
}

///|
fn json_value(v : Value) -> Json {
  match v {
    Num(v0) => @base.tag("Num", [int64_json(v0)])
    Location(v0) => @base.tag("Location", [json_location(v0)])
    Null_ptr => @base.tag("Null_ptr", [])
  }
}

///|
fn read_value(j : Json) -> Value raise @base.Fault {
  let tag = @base.tag_name(j)
  let a = @base.array(j)[1:].to_owned()
  match tag {
    "Num" => {
      @base.require(a.length() == 1, "Invalid Num")
      Num(read_int64(a[0]))
    }
    "Location" => {
      @base.require(a.length() == 1, "Invalid Location")
      Location(read_location(a[0]))
    }
    "Null_ptr" => {
      @base.require(a.length() == 0, "Invalid Null_ptr")
      Null_ptr
    }
    _ => @base.error("Invalid Value tag: " + tag)
  }
}

///|
fn json_function(v : Function) -> Json {
  @base.object([
    ("name", @base.jstr(v.name)),
    ("params", @base.jarray(v.params.map(x => @base.jstr(x)))),
    ("body", @base.jarray(v.body.map(x => json_stmt(x)))),
  ])
}

///|
fn read_function(j : Json) -> Function raise @base.Fault {
  {
    name: @base.string(@base.field(j, "name")),
    params: @base.array(@base.field(j, "params")).map(x => @base.string(x)),
    body: @base.array(@base.field(j, "body")).map(x => read_stmt(x)),
  }
}

///|
fn json_code(v : Code) -> Json {
  @base.object([
    ("functions", @base.jarray(v.functions.map(x => json_function(x)))),
  ])
}

///|
fn read_code(j : Json) -> Code raise @base.Fault {
  {
    functions: @base.array(@base.field(j, "functions")).map(x => {
      read_function(x)
    }),
  }
}

///|
fn json_location(v : Location) -> Json {
  @base.object([
    ("frame_id", @base.jint(v.frame_id)),
    ("slot", @base.jint(v.slot)),
  ])
}

///|
fn read_location(j : Json) -> Location raise @base.Fault {
  {
    frame_id: @base.int(@base.field(j, "frame_id")),
    slot: @base.int(@base.field(j, "slot")),
  }
}

///|
fn json_framestore(v : FrameStore) -> Json {
  @base.object([
    ("frame_id", @base.jint(v.frame_id)),
    (
      "values",
      @base.jarray(
        v.values.map(x => @base.jarray([json_location(x.0), json_value(x.1)])),
      ),
    ),
  ])
}

///|
fn read_framestore(j : Json) -> FrameStore raise @base.Fault {
  {
    frame_id: @base.int(@base.field(j, "frame_id")),
    values: @base.array(@base.field(j, "values")).map(x => {
      let items = @base.array(x)
      @base.require(items.length() == 2, "Invalid tuple")
      (read_location(items[0]), read_value(items[1]))
    }),
  }
}

///|
fn json_frame(v : Frame) -> Json {
  @base.object([
    (
      "env",
      @base.jarray(
        v.env.map(x => @base.jarray([@base.jstr(x.0), json_location(x.1)])),
      ),
    ),
    ("inner_k", @base.jarray(v.inner_k.map(x => json_stmt(x)))),
    ("function_name", @base.jstr(v.function_name)),
  ])
}

///|
fn read_frame(j : Json) -> Frame raise @base.Fault {
  {
    env: @base.array(@base.field(j, "env")).map(x => {
      let items = @base.array(x)
      @base.require(items.length() == 2, "Invalid tuple")
      (@base.string(items[0]), read_location(items[1]))
    }),
    inner_k: @base.array(@base.field(j, "inner_k")).map(x => read_stmt(x)),
    function_name: @base.string(@base.field(j, "function_name")),
  }
}

///|
fn json_state(v : State) -> Json {
  @base.object([
    ("code", json_code(v.code)),
    ("current_function", @base.jstr(v.current_function)),
    (
      "current_stmt",
      match v.current_stmt {
        Some(x) => json_stmt(x)
        None => Json::null()
      },
    ),
    (
      "env",
      @base.jarray(
        v.env.map(x => @base.jarray([@base.jstr(x.0), json_location(x.1)])),
      ),
    ),
    ("store", @base.jarray(v.store.map(x => json_framestore(x)))),
    ("inner_k", @base.jarray(v.inner_k.map(x => json_stmt(x)))),
    ("fn_k", @base.jarray(v.fn_k.map(x => json_frame(x)))),
    ("next_frame_id", @base.jint(v.next_frame_id)),
  ])
}

///|
fn read_state(j : Json) -> State raise @base.Fault {
  {
    code: read_code(@base.field(j, "code")),
    current_function: @base.string(@base.field(j, "current_function")),
    current_stmt: match @base.field(j, "current_stmt") {
      Null => None
      _ => Some(read_stmt(@base.field(j, "current_stmt")))
    },
    env: @base.array(@base.field(j, "env")).map(x => {
      let items = @base.array(x)
      @base.require(items.length() == 2, "Invalid tuple")
      (@base.string(items[0]), read_location(items[1]))
    }),
    store: @base.array(@base.field(j, "store")).map(x => read_framestore(x)),
    inner_k: @base.array(@base.field(j, "inner_k")).map(x => read_stmt(x)),
    fn_k: @base.array(@base.field(j, "fn_k")).map(x => read_frame(x)),
    next_frame_id: @base.int(@base.field(j, "next_frame_id")),
  }
}

///|
fn decode(j : Json) -> State raise @base.Fault {
  let s = read_state(j)
  validate(s.code)
  @base.require(s.next_frame_id >= 0, "Invalid frame ID supply")
  fn location(l : Location) -> Unit raise @base.Fault {
    @base.require(
      l.frame_id >= 0 && l.frame_id < s.next_frame_id,
      "Invalid frame ID",
    )
  }
  for f in s.store {
    @base.require(
      f.frame_id >= 0 && f.frame_id < s.next_frame_id,
      "Invalid frame ID",
    )
    for pair in f.values {
      location(pair.0)
      if pair.1 is Location(l) {
        location(l)
      }
    }
  }
  for pair in s.env {
    location(pair.1)
  }
  for f in s.fn_k {
    for pair in f.env {
      location(pair.1)
    }
  }
  s
}
