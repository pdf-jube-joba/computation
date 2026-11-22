///|
fn int64_json(n : Int64) -> Json {
  Json::number(n.to_double(), repr=n.to_string())
}

///|
fn read_int64(j : Json) -> Int64 raise @utils.Fault {
  @string.parse_int64(j.stringify()) catch {
    _ => @utils.error("Invalid int64")
  }
}

///|
fn json_binop(v : Binop) -> Json {
  match v {
    Add => @utils.tag("Add", [])
    Sub => @utils.tag("Sub", [])
  }
}

///|
fn read_binop(j : Json) -> Binop raise @utils.Fault {
  let tag = @utils.tag_name(j)
  let a = @utils.array(j)[1:].to_owned()
  match tag {
    "Add" => {
      @utils.require(a.length() == 0, "Invalid Add")
      Add
    }
    "Sub" => {
      @utils.require(a.length() == 0, "Invalid Sub")
      Sub
    }
    _ => @utils.error("Invalid Binop tag: " + tag)
  }
}

///|
fn json_place(v : Place) -> Json {
  match v {
    Var(v0) => @utils.tag("Var", [@utils.jstr(v0)])
    Deref(v0) => @utils.tag("Deref", [json_expr(v0)])
  }
}

///|
fn read_place(j : Json) -> Place raise @utils.Fault {
  let tag = @utils.tag_name(j)
  let a = @utils.array(j)[1:].to_owned()
  match tag {
    "Var" => {
      @utils.require(a.length() == 1, "Invalid Var")
      Var(@utils.string(a[0]))
    }
    "Deref" => {
      @utils.require(a.length() == 1, "Invalid Deref")
      Deref(read_expr(a[0]))
    }
    _ => @utils.error("Invalid Place tag: " + tag)
  }
}

///|
fn json_expr(v : Expr) -> Json {
  match v {
    Number(v0) => @utils.tag("Number", [int64_json(v0)])
    NullPtr => @utils.tag("NullPtr", [])
    BinOp(v0, v1, v2) =>
      @utils.tag("BinOp", [json_expr(v0), json_binop(v1), json_expr(v2)])
    Load(v0) => @utils.tag("Load", [json_place(v0)])
    Addr(v0) => @utils.tag("Addr", [json_place(v0)])
  }
}

///|
fn read_expr(j : Json) -> Expr raise @utils.Fault {
  let tag = @utils.tag_name(j)
  let a = @utils.array(j)[1:].to_owned()
  match tag {
    "Number" => {
      @utils.require(a.length() == 1, "Invalid Number")
      Number(read_int64(a[0]))
    }
    "NullPtr" => {
      @utils.require(a.length() == 0, "Invalid NullPtr")
      NullPtr
    }
    "BinOp" => {
      @utils.require(a.length() == 3, "Invalid BinOp")
      BinOp(read_expr(a[0]), read_binop(a[1]), read_expr(a[2]))
    }
    "Load" => {
      @utils.require(a.length() == 1, "Invalid Load")
      Load(read_place(a[0]))
    }
    "Addr" => {
      @utils.require(a.length() == 1, "Invalid Addr")
      Addr(read_place(a[0]))
    }
    _ => @utils.error("Invalid Expr tag: " + tag)
  }
}

///|
fn json_stmt(v : Stmt) -> Json {
  match v {
    Assign(v0, v1) => @utils.tag("Assign", [json_place(v0), json_expr(v1)])
    Ifz(v0, v1, v2) =>
      @utils.tag("Ifz", [json_expr(v0), json_stmt(v1), json_stmt(v2)])
    Call(v0, v1) =>
      @utils.tag("Call", [
        @utils.jstr(v0),
        @utils.jarray(v1.map(x => json_expr(x))),
      ])
    Return => @utils.tag("Return", [])
  }
}

///|
fn read_stmt(j : Json) -> Stmt raise @utils.Fault {
  let tag = @utils.tag_name(j)
  let a = @utils.array(j)[1:].to_owned()
  match tag {
    "Assign" => {
      @utils.require(a.length() == 2, "Invalid Assign")
      Assign(read_place(a[0]), read_expr(a[1]))
    }
    "Ifz" => {
      @utils.require(a.length() == 3, "Invalid Ifz")
      Ifz(read_expr(a[0]), read_stmt(a[1]), read_stmt(a[2]))
    }
    "Call" => {
      @utils.require(a.length() == 2, "Invalid Call")
      Call(@utils.string(a[0]), @utils.array(a[1]).map(x => read_expr(x)))
    }
    "Return" => {
      @utils.require(a.length() == 0, "Invalid Return")
      Return
    }
    _ => @utils.error("Invalid Stmt tag: " + tag)
  }
}

///|
fn json_value(v : Value) -> Json {
  match v {
    Num(v0) => @utils.tag("Num", [int64_json(v0)])
    Location(v0) => @utils.tag("Location", [json_location(v0)])
    Null_ptr => @utils.tag("Null_ptr", [])
  }
}

///|
fn read_value(j : Json) -> Value raise @utils.Fault {
  let tag = @utils.tag_name(j)
  let a = @utils.array(j)[1:].to_owned()
  match tag {
    "Num" => {
      @utils.require(a.length() == 1, "Invalid Num")
      Num(read_int64(a[0]))
    }
    "Location" => {
      @utils.require(a.length() == 1, "Invalid Location")
      Location(read_location(a[0]))
    }
    "Null_ptr" => {
      @utils.require(a.length() == 0, "Invalid Null_ptr")
      Null_ptr
    }
    _ => @utils.error("Invalid Value tag: " + tag)
  }
}

///|
fn json_function(v : Function) -> Json {
  @utils.object([
    ("name", @utils.jstr(v.name)),
    ("params", @utils.jarray(v.params.map(x => @utils.jstr(x)))),
    ("body", @utils.jarray(v.body.map(x => json_stmt(x)))),
  ])
}

///|
fn read_function(j : Json) -> Function raise @utils.Fault {
  {
    name: @utils.string(@utils.field(j, "name")),
    params: @utils.array(@utils.field(j, "params")).map(x => @utils.string(x)),
    body: @utils.array(@utils.field(j, "body")).map(x => read_stmt(x)),
  }
}

///|
fn json_code(v : Code) -> Json {
  @utils.object([
    ("functions", @utils.jarray(v.functions.map(x => json_function(x)))),
  ])
}

///|
fn read_code(j : Json) -> Code raise @utils.Fault {
  {
    functions: @utils.array(@utils.field(j, "functions")).map(x => {
      read_function(x)
    }),
  }
}

///|
fn json_location(v : Location) -> Json {
  @utils.object([
    ("frame_id", @utils.jint(v.frame_id)),
    ("slot", @utils.jint(v.slot)),
  ])
}

///|
fn read_location(j : Json) -> Location raise @utils.Fault {
  {
    frame_id: @utils.int(@utils.field(j, "frame_id")),
    slot: @utils.int(@utils.field(j, "slot")),
  }
}

///|
fn json_framestore(v : FrameStore) -> Json {
  @utils.object([
    ("frame_id", @utils.jint(v.frame_id)),
    (
      "values",
      @utils.jarray(
        v.values.map(x => @utils.jarray([json_location(x.0), json_value(x.1)])),
      ),
    ),
  ])
}

///|
fn read_framestore(j : Json) -> FrameStore raise @utils.Fault {
  {
    frame_id: @utils.int(@utils.field(j, "frame_id")),
    values: @utils.array(@utils.field(j, "values")).map(x => {
      let items = @utils.array(x)
      @utils.require(items.length() == 2, "Invalid tuple")
      (read_location(items[0]), read_value(items[1]))
    }),
  }
}

///|
fn json_frame(v : Frame) -> Json {
  @utils.object([
    (
      "env",
      @utils.jarray(
        v.env.map(x => @utils.jarray([@utils.jstr(x.0), json_location(x.1)])),
      ),
    ),
    ("inner_k", @utils.jarray(v.inner_k.map(x => json_stmt(x)))),
    ("function_name", @utils.jstr(v.function_name)),
  ])
}

///|
fn read_frame(j : Json) -> Frame raise @utils.Fault {
  {
    env: @utils.array(@utils.field(j, "env")).map(x => {
      let items = @utils.array(x)
      @utils.require(items.length() == 2, "Invalid tuple")
      (@utils.string(items[0]), read_location(items[1]))
    }),
    inner_k: @utils.array(@utils.field(j, "inner_k")).map(x => read_stmt(x)),
    function_name: @utils.string(@utils.field(j, "function_name")),
  }
}

///|
fn json_state(v : State) -> Json {
  @utils.object([
    ("code", json_code(v.code)),
    ("current_function", @utils.jstr(v.current_function)),
    (
      "current_stmt",
      match v.current_stmt {
        Some(x) => json_stmt(x)
        None => Json::null()
      },
    ),
    (
      "env",
      @utils.jarray(
        v.env.map(x => @utils.jarray([@utils.jstr(x.0), json_location(x.1)])),
      ),
    ),
    ("store", @utils.jarray(v.store.map(x => json_framestore(x)))),
    ("inner_k", @utils.jarray(v.inner_k.map(x => json_stmt(x)))),
    ("fn_k", @utils.jarray(v.fn_k.map(x => json_frame(x)))),
    ("next_frame_id", @utils.jint(v.next_frame_id)),
  ])
}

///|
fn read_state(j : Json) -> State raise @utils.Fault {
  {
    code: read_code(@utils.field(j, "code")),
    current_function: @utils.string(@utils.field(j, "current_function")),
    current_stmt: match @utils.field(j, "current_stmt") {
      Null => None
      _ => Some(read_stmt(@utils.field(j, "current_stmt")))
    },
    env: @utils.array(@utils.field(j, "env")).map(x => {
      let items = @utils.array(x)
      @utils.require(items.length() == 2, "Invalid tuple")
      (@utils.string(items[0]), read_location(items[1]))
    }),
    store: @utils.array(@utils.field(j, "store")).map(x => read_framestore(x)),
    inner_k: @utils.array(@utils.field(j, "inner_k")).map(x => read_stmt(x)),
    fn_k: @utils.array(@utils.field(j, "fn_k")).map(x => read_frame(x)),
    next_frame_id: @utils.int(@utils.field(j, "next_frame_id")),
  }
}

///|
fn decode(j : Json) -> State raise @utils.Fault {
  let s = read_state(j)
  validate(s.code)
  @utils.require(s.next_frame_id >= 0, "Invalid frame ID supply")
  fn location(l : Location) -> Unit raise @utils.Fault {
    @utils.require(
      l.frame_id >= 0 && l.frame_id < s.next_frame_id,
      "Invalid frame ID",
    )
  }
  for f in s.store {
    @utils.require(
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
