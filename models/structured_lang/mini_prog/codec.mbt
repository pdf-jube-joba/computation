///|
fn nat_json(n : @utils.Nat) -> Json {
  @utils.jstr(n.to_string())
}

///|
fn json_ty(v : Ty) -> Json {
  match v {
    Num => @utils.tag("Num", [])
    U8 => @utils.tag("U8", [])
    Bool => @utils.tag("Bool", [])
    Unit => @utils.tag("Unit", [])
    Ptr => @utils.tag("Ptr", [])
    Fn => @utils.tag("Fn", [])
    Product(v0) =>
      @utils.tag("Product", [@utils.jarray(v0.map(x => json_ty(x)))])
    Sum(v0) => @utils.tag("Sum", [@utils.jarray(v0.map(x => json_ty(x)))])
    Array(v0, v1) => @utils.tag("Array", [json_ty(v0), @utils.jint(v1)])
  }
}

///|
fn read_ty(j : Json) -> Ty raise @utils.Fault {
  let tag = @utils.tag_name(j)
  let a = @utils.array(j)[1:].to_owned()
  match tag {
    "Num" => {
      @utils.require(a.length() == 0, "Invalid Num")
      Num
    }
    "U8" => {
      @utils.require(a.length() == 0, "Invalid U8")
      U8
    }
    "Bool" => {
      @utils.require(a.length() == 0, "Invalid Bool")
      Bool
    }
    "Unit" => {
      @utils.require(a.length() == 0, "Invalid Unit")
      Unit
    }
    "Ptr" => {
      @utils.require(a.length() == 0, "Invalid Ptr")
      Ptr
    }
    "Fn" => {
      @utils.require(a.length() == 0, "Invalid Fn")
      Fn
    }
    "Product" => {
      @utils.require(a.length() == 1, "Invalid Product")
      Product(@utils.array(a[0]).map(x => read_ty(x)))
    }
    "Sum" => {
      @utils.require(a.length() == 1, "Invalid Sum")
      Sum(@utils.array(a[0]).map(x => read_ty(x)))
    }
    "Array" => {
      @utils.require(a.length() == 2, "Invalid Array")
      Array(read_ty(a[0]), @utils.int(a[1]))
    }
    _ => @utils.error("Invalid Ty tag: " + tag)
  }
}

///|
fn json_binop(v : Binop) -> Json {
  match v {
    Add => @utils.tag("Add", [])
    Sub => @utils.tag("Sub", [])
    Eq => @utils.tag("Eq", [])
    Lt => @utils.tag("Lt", [])
    And => @utils.tag("And", [])
    Or => @utils.tag("Or", [])
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
    "Eq" => {
      @utils.require(a.length() == 0, "Invalid Eq")
      Eq
    }
    "Lt" => {
      @utils.require(a.length() == 0, "Invalid Lt")
      Lt
    }
    "And" => {
      @utils.require(a.length() == 0, "Invalid And")
      And
    }
    "Or" => {
      @utils.require(a.length() == 0, "Invalid Or")
      Or
    }
    _ => @utils.error("Invalid Binop tag: " + tag)
  }
}

///|
fn json_unop(v : Unop) -> Json {
  match v {
    Not => @utils.tag("Not", [])
  }
}

///|
fn read_unop(j : Json) -> Unop raise @utils.Fault {
  let tag = @utils.tag_name(j)
  let a = @utils.array(j)[1:].to_owned()
  match tag {
    "Not" => {
      @utils.require(a.length() == 0, "Invalid Not")
      Not
    }
    _ => @utils.error("Invalid Unop tag: " + tag)
  }
}

///|
fn json_place(v : Place) -> Json {
  match v {
    Static(v0) => @utils.tag("Static", [@utils.jstr(v0)])
    Local(v0) => @utils.tag("Local", [@utils.jstr(v0)])
    Field(v0, v1) => @utils.tag("Field", [json_place(v0), @utils.jint(v1)])
    Tag_place(v0, v1) =>
      @utils.tag("Tag_place", [json_place(v0), @utils.jint(v1)])
    Index(v0, v1) => @utils.tag("Index", [json_place(v0), json_expr(v1)])
    Deref(v0) => @utils.tag("Deref", [json_expr(v0)])
  }
}

///|
fn read_place(j : Json) -> Place raise @utils.Fault {
  let tag = @utils.tag_name(j)
  let a = @utils.array(j)[1:].to_owned()
  match tag {
    "Static" => {
      @utils.require(a.length() == 1, "Invalid Static")
      Static(@utils.string(a[0]))
    }
    "Local" => {
      @utils.require(a.length() == 1, "Invalid Local")
      Local(@utils.string(a[0]))
    }
    "Field" => {
      @utils.require(a.length() == 2, "Invalid Field")
      Field(read_place(a[0]), @utils.int(a[1]))
    }
    "Tag_place" => {
      @utils.require(a.length() == 2, "Invalid Tag_place")
      Tag_place(read_place(a[0]), @utils.int(a[1]))
    }
    "Index" => {
      @utils.require(a.length() == 2, "Invalid Index")
      Index(read_place(a[0]), read_expr(a[1]))
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
    Number(v0) => @utils.tag("Number", [nat_json(v0)])
    Char(v0) => @utils.tag("Char", [@utils.jstr(v0)])
    Boolean(v0) => @utils.tag("Boolean", [@utils.jbool(v0)])
    Unit_value => @utils.tag("Unit_value", [])
    NullPtr => @utils.tag("NullPtr", [])
    NullFn => @utils.tag("NullFn", [])
    BinOp(v0, v1, v2) =>
      @utils.tag("BinOp", [json_expr(v0), json_binop(v1), json_expr(v2)])
    UnOp(v0, v1) => @utils.tag("UnOp", [json_unop(v0), json_expr(v1)])
    Pair(v0) => @utils.tag("Pair", [@utils.jarray(v0.map(x => json_expr(x)))])
    Tag(v0, v1) => @utils.tag("Tag", [@utils.jint(v0), json_expr(v1)])
    Load(v0) => @utils.tag("Load", [json_place(v0)])
    Addr(v0) => @utils.tag("Addr", [json_place(v0)])
    Function(v0) => @utils.tag("Function", [@utils.jstr(v0)])
  }
}

///|
fn read_expr(j : Json) -> Expr raise @utils.Fault {
  let tag = @utils.tag_name(j)
  let a = @utils.array(j)[1:].to_owned()
  match tag {
    "Number" => {
      @utils.require(a.length() == 1, "Invalid Number")
      Number(@utils.json_nat(a[0]))
    }
    "Char" => {
      @utils.require(a.length() == 1, "Invalid Char")
      Char(@utils.string(a[0]))
    }
    "Boolean" => {
      @utils.require(a.length() == 1, "Invalid Boolean")
      Boolean(@utils.boolean(a[0]))
    }
    "Unit_value" => {
      @utils.require(a.length() == 0, "Invalid Unit_value")
      Unit_value
    }
    "NullPtr" => {
      @utils.require(a.length() == 0, "Invalid NullPtr")
      NullPtr
    }
    "NullFn" => {
      @utils.require(a.length() == 0, "Invalid NullFn")
      NullFn
    }
    "BinOp" => {
      @utils.require(a.length() == 3, "Invalid BinOp")
      BinOp(read_expr(a[0]), read_binop(a[1]), read_expr(a[2]))
    }
    "UnOp" => {
      @utils.require(a.length() == 2, "Invalid UnOp")
      UnOp(read_unop(a[0]), read_expr(a[1]))
    }
    "Pair" => {
      @utils.require(a.length() == 1, "Invalid Pair")
      Pair(@utils.array(a[0]).map(x => read_expr(x)))
    }
    "Tag" => {
      @utils.require(a.length() == 2, "Invalid Tag")
      Tag(@utils.int(a[0]), read_expr(a[1]))
    }
    "Load" => {
      @utils.require(a.length() == 1, "Invalid Load")
      Load(read_place(a[0]))
    }
    "Addr" => {
      @utils.require(a.length() == 1, "Invalid Addr")
      Addr(read_place(a[0]))
    }
    "Function" => {
      @utils.require(a.length() == 1, "Invalid Function")
      Function(@utils.string(a[0]))
    }
    _ => @utils.error("Invalid Expr tag: " + tag)
  }
}

///|
fn json_stmt(v : Stmt) -> Json {
  match v {
    Assign(v0, v1) => @utils.tag("Assign", [json_place(v0), json_expr(v1)])
    If(v0, v1) => @utils.tag("If", [json_expr(v0), json_stmt(v1)])
    Case(v0, v1, v2) =>
      @utils.tag("Case", [@utils.jint(v0), json_expr(v1), json_stmt(v2)])
    HAlloc(v0, v1) => @utils.tag("HAlloc", [json_ty(v0), json_place(v1)])
    HFree(v0) => @utils.tag("HFree", [json_expr(v0)])
    Loop(v0, v1) => @utils.tag("Loop", [@utils.jstr(v0), json_stmt(v1)])
    Break(v0) => @utils.tag("Break", [@utils.jstr(v0)])
    Continue(v0) => @utils.tag("Continue", [@utils.jstr(v0)])
    Call(v0, v1, v2) =>
      @utils.tag("Call", [
        json_expr(v0),
        @utils.jarray(v1.map(x => json_expr(x))),
        @utils.jarray(v2.map(x => json_place(x))),
      ])
    Return(v0) =>
      @utils.tag("Return", [@utils.jarray(v0.map(x => json_expr(x)))])
    Block(v0) => @utils.tag("Block", [json_block(v0)])
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
    "If" => {
      @utils.require(a.length() == 2, "Invalid If")
      If(read_expr(a[0]), read_stmt(a[1]))
    }
    "Case" => {
      @utils.require(a.length() == 3, "Invalid Case")
      Case(@utils.int(a[0]), read_expr(a[1]), read_stmt(a[2]))
    }
    "HAlloc" => {
      @utils.require(a.length() == 2, "Invalid HAlloc")
      HAlloc(read_ty(a[0]), read_place(a[1]))
    }
    "HFree" => {
      @utils.require(a.length() == 1, "Invalid HFree")
      HFree(read_expr(a[0]))
    }
    "Loop" => {
      @utils.require(a.length() == 2, "Invalid Loop")
      Loop(@utils.string(a[0]), read_stmt(a[1]))
    }
    "Break" => {
      @utils.require(a.length() == 1, "Invalid Break")
      Break(@utils.string(a[0]))
    }
    "Continue" => {
      @utils.require(a.length() == 1, "Invalid Continue")
      Continue(@utils.string(a[0]))
    }
    "Call" => {
      @utils.require(a.length() == 3, "Invalid Call")
      Call(
        read_expr(a[0]),
        @utils.array(a[1]).map(x => read_expr(x)),
        @utils.array(a[2]).map(x => read_place(x)),
      )
    }
    "Return" => {
      @utils.require(a.length() == 1, "Invalid Return")
      Return(@utils.array(a[0]).map(x => read_expr(x)))
    }
    "Block" => {
      @utils.require(a.length() == 1, "Invalid Block")
      Block(read_block(a[0]))
    }
    _ => @utils.error("Invalid Stmt tag: " + tag)
  }
}

///|
fn json_key(v : Key) -> Json {
  match v {
    Heap(v0) => @utils.tag("Heap", [@utils.jint(v0)])
    Static_key(v0) => @utils.tag("Static_key", [@utils.jstr(v0)])
    Local_key(v0) => @utils.tag("Local_key", [@utils.jint(v0)])
  }
}

///|
fn read_key(j : Json) -> Key raise @utils.Fault {
  let tag = @utils.tag_name(j)
  let a = @utils.array(j)[1:].to_owned()
  match tag {
    "Heap" => {
      @utils.require(a.length() == 1, "Invalid Heap")
      Heap(@utils.int(a[0]))
    }
    "Static_key" => {
      @utils.require(a.length() == 1, "Invalid Static_key")
      Static_key(@utils.string(a[0]))
    }
    "Local_key" => {
      @utils.require(a.length() == 1, "Invalid Local_key")
      Local_key(@utils.int(a[0]))
    }
    _ => @utils.error("Invalid Key tag: " + tag)
  }
}

///|
fn json_projection(v : Projection) -> Json {
  match v {
    Field_proj(v0) => @utils.tag("Field_proj", [@utils.jint(v0)])
    Tag_proj(v0) => @utils.tag("Tag_proj", [@utils.jint(v0)])
    Index_proj(v0) => @utils.tag("Index_proj", [@utils.jint(v0)])
  }
}

///|
fn read_projection(j : Json) -> Projection raise @utils.Fault {
  let tag = @utils.tag_name(j)
  let a = @utils.array(j)[1:].to_owned()
  match tag {
    "Field_proj" => {
      @utils.require(a.length() == 1, "Invalid Field_proj")
      Field_proj(@utils.int(a[0]))
    }
    "Tag_proj" => {
      @utils.require(a.length() == 1, "Invalid Tag_proj")
      Tag_proj(@utils.int(a[0]))
    }
    "Index_proj" => {
      @utils.require(a.length() == 1, "Invalid Index_proj")
      Index_proj(@utils.int(a[0]))
    }
    _ => @utils.error("Invalid Projection tag: " + tag)
  }
}

///|
fn json_value(v : Value) -> Json {
  match v {
    Number_value(v0) => @utils.tag("Number_value", [nat_json(v0)])
    Char_value(v0) => @utils.tag("Char_value", [@utils.jstr(v0)])
    Bool_value(v0) => @utils.tag("Bool_value", [@utils.jbool(v0)])
    Unit_v => @utils.tag("Unit_v", [])
    Ptr_value(v0) =>
      @utils.tag("Ptr_value", [
        match v0 {
          Some(x) => json_location(x)
          None => Json::null()
        },
      ])
    FnPtr(v0) =>
      @utils.tag("FnPtr", [
        match v0 {
          Some(x) => @utils.jstr(x)
          None => Json::null()
        },
      ])
    Aggregate(v0) =>
      @utils.tag("Aggregate", [@utils.jarray(v0.map(x => json_value(x)))])
    Sum_value(v0, v1) =>
      @utils.tag("Sum_value", [@utils.jint(v0), json_value(v1)])
  }
}

///|
fn read_value(j : Json) -> Value raise @utils.Fault {
  let tag = @utils.tag_name(j)
  let a = @utils.array(j)[1:].to_owned()
  match tag {
    "Number_value" => {
      @utils.require(a.length() == 1, "Invalid Number_value")
      Number_value(@utils.json_nat(a[0]))
    }
    "Char_value" => {
      @utils.require(a.length() == 1, "Invalid Char_value")
      Char_value(@utils.string(a[0]))
    }
    "Bool_value" => {
      @utils.require(a.length() == 1, "Invalid Bool_value")
      Bool_value(@utils.boolean(a[0]))
    }
    "Unit_v" => {
      @utils.require(a.length() == 0, "Invalid Unit_v")
      Unit_v
    }
    "Ptr_value" => {
      @utils.require(a.length() == 1, "Invalid Ptr_value")
      Ptr_value(
        match a[0] {
          Null => None
          _ => Some(read_location(a[0]))
        },
      )
    }
    "FnPtr" => {
      @utils.require(a.length() == 1, "Invalid FnPtr")
      FnPtr(
        match a[0] {
          Null => None
          _ => Some(@utils.string(a[0]))
        },
      )
    }
    "Aggregate" => {
      @utils.require(a.length() == 1, "Invalid Aggregate")
      Aggregate(@utils.array(a[0]).map(x => read_value(x)))
    }
    "Sum_value" => {
      @utils.require(a.length() == 2, "Invalid Sum_value")
      Sum_value(@utils.int(a[0]), read_value(a[1]))
    }
    _ => @utils.error("Invalid Value tag: " + tag)
  }
}

///|
fn json_control(v : Control) -> Json {
  match v {
    Next => @utils.tag("Next", [])
    Break_ctrl(v0) => @utils.tag("Break_ctrl", [@utils.jstr(v0)])
    Continue_ctrl(v0) => @utils.tag("Continue_ctrl", [@utils.jstr(v0)])
  }
}

///|
fn read_control(j : Json) -> Control raise @utils.Fault {
  let tag = @utils.tag_name(j)
  let a = @utils.array(j)[1:].to_owned()
  match tag {
    "Next" => {
      @utils.require(a.length() == 0, "Invalid Next")
      Next
    }
    "Break_ctrl" => {
      @utils.require(a.length() == 1, "Invalid Break_ctrl")
      Break_ctrl(@utils.string(a[0]))
    }
    "Continue_ctrl" => {
      @utils.require(a.length() == 1, "Invalid Continue_ctrl")
      Continue_ctrl(@utils.string(a[0]))
    }
    _ => @utils.error("Invalid Control tag: " + tag)
  }
}

///|
fn json_exec(v : Exec) -> Json {
  match v {
    Eval(v0) => @utils.tag("Eval", [json_stmt(v0)])
    Ctrl(v0) => @utils.tag("Ctrl", [json_control(v0)])
    Ret(v0) => @utils.tag("Ret", [@utils.jarray(v0.map(x => json_value(x)))])
  }
}

///|
fn read_exec(j : Json) -> Exec raise @utils.Fault {
  let tag = @utils.tag_name(j)
  let a = @utils.array(j)[1:].to_owned()
  match tag {
    "Eval" => {
      @utils.require(a.length() == 1, "Invalid Eval")
      Eval(read_stmt(a[0]))
    }
    "Ctrl" => {
      @utils.require(a.length() == 1, "Invalid Ctrl")
      Ctrl(read_control(a[0]))
    }
    "Ret" => {
      @utils.require(a.length() == 1, "Invalid Ret")
      Ret(@utils.array(a[0]).map(x => read_value(x)))
    }
    _ => @utils.error("Invalid Exec tag: " + tag)
  }
}

///|
fn json_innerframe(v : InnerFrame) -> Json {
  match v {
    Stmt_frame(v0) => @utils.tag("Stmt_frame", [json_stmt(v0)])
    Scope => @utils.tag("Scope", [])
    Loop_frame(v0, v1) =>
      @utils.tag("Loop_frame", [@utils.jstr(v0), json_stmt(v1)])
  }
}

///|
fn read_innerframe(j : Json) -> InnerFrame raise @utils.Fault {
  let tag = @utils.tag_name(j)
  let a = @utils.array(j)[1:].to_owned()
  match tag {
    "Stmt_frame" => {
      @utils.require(a.length() == 1, "Invalid Stmt_frame")
      Stmt_frame(read_stmt(a[0]))
    }
    "Scope" => {
      @utils.require(a.length() == 0, "Invalid Scope")
      Scope
    }
    "Loop_frame" => {
      @utils.require(a.length() == 2, "Invalid Loop_frame")
      Loop_frame(@utils.string(a[0]), read_stmt(a[1]))
    }
    _ => @utils.error("Invalid InnerFrame tag: " + tag)
  }
}

///|
fn json_block(v : Block) -> Json {
  @utils.object([
    (
      "bindings",
      @utils.jarray(
        v.bindings.map(x => @utils.jarray([@utils.jstr(x.0), json_ty(x.1)])),
      ),
    ),
    ("stmts", @utils.jarray(v.stmts.map(x => json_stmt(x)))),
  ])
}

///|
fn read_block(j : Json) -> Block raise @utils.Fault {
  {
    bindings: @utils.array(@utils.field(j, "bindings")).map(x => {
      let items = @utils.array(x)
      @utils.require(items.length() == 2, "Invalid tuple")
      (@utils.string(items[0]), read_ty(items[1]))
    }),
    stmts: @utils.array(@utils.field(j, "stmts")).map(x => read_stmt(x)),
  }
}

///|
fn json_staticdecl(v : StaticDecl) -> Json {
  @utils.object([("name", @utils.jstr(v.name)), ("ty", json_ty(v.ty))])
}

///|
fn read_staticdecl(j : Json) -> StaticDecl raise @utils.Fault {
  {
    name: @utils.string(@utils.field(j, "name")),
    ty: read_ty(@utils.field(j, "ty")),
  }
}

///|
fn json_functiondecl(v : FunctionDecl) -> Json {
  @utils.object([("name", @utils.jstr(v.name)), ("block", json_block(v.block))])
}

///|
fn read_functiondecl(j : Json) -> FunctionDecl raise @utils.Fault {
  {
    name: @utils.string(@utils.field(j, "name")),
    block: read_block(@utils.field(j, "block")),
  }
}

///|
fn json_code(v : Code) -> Json {
  @utils.object([
    ("statics", @utils.jarray(v.statics.map(x => json_staticdecl(x)))),
    ("functions", @utils.jarray(v.functions.map(x => json_functiondecl(x)))),
  ])
}

///|
fn read_code(j : Json) -> Code raise @utils.Fault {
  {
    statics: @utils.array(@utils.field(j, "statics")).map(x => {
      read_staticdecl(x)
    }),
    functions: @utils.array(@utils.field(j, "functions")).map(x => {
      read_functiondecl(x)
    }),
  }
}

///|
fn json_location(v : Location) -> Json {
  @utils.object([
    ("key", json_key(v.key)),
    ("projections", @utils.jarray(v.projections.map(x => json_projection(x)))),
  ])
}

///|
fn read_location(j : Json) -> Location raise @utils.Fault {
  {
    key: read_key(@utils.field(j, "key")),
    projections: @utils.array(@utils.field(j, "projections")).map(x => {
      read_projection(x)
    }),
  }
}

///|
fn json_cell(v : Cell) -> Json {
  @utils.object([("ty", json_ty(v.ty)), ("value", json_value(v.value))])
}

///|
fn read_cell(j : Json) -> Cell raise @utils.Fault {
  {
    ty: read_ty(@utils.field(j, "ty")),
    value: read_value(@utils.field(j, "value")),
  }
}

///|
fn json_store(v : Store) -> Json {
  @utils.object([
    (
      "heap",
      @utils.jarray(
        v.heap.map(x => @utils.jarray([@utils.jint(x.0), json_cell(x.1)])),
      ),
    ),
    (
      "statics",
      @utils.jarray(
        v.statics.map(x => @utils.jarray([@utils.jstr(x.0), json_cell(x.1)])),
      ),
    ),
    (
      "locals",
      @utils.jarray(
        v.locals.map(x => {
          @utils.jarray(
            x.map(x => {
              @utils.jarray(
                x.map(x => @utils.jarray([@utils.jint(x.0), json_cell(x.1)])),
              )
            }),
          )
        }),
      ),
    ),
  ])
}

///|
fn read_store(j : Json) -> Store raise @utils.Fault {
  {
    heap: @utils.array(@utils.field(j, "heap")).map(x => {
      let items = @utils.array(x)
      @utils.require(items.length() == 2, "Invalid tuple")
      (@utils.int(items[0]), read_cell(items[1]))
    }),
    statics: @utils.array(@utils.field(j, "statics")).map(x => {
      let items = @utils.array(x)
      @utils.require(items.length() == 2, "Invalid tuple")
      (@utils.string(items[0]), read_cell(items[1]))
    }),
    locals: @utils.array(@utils.field(j, "locals")).map(x => {
      @utils.array(x).map(x => {
        @utils.array(x).map(x => {
          let items = @utils.array(x)
          @utils.require(items.length() == 2, "Invalid tuple")
          (@utils.int(items[0]), read_cell(items[1]))
        })
      })
    }),
  }
}

///|
fn json_fnframe(v : FnFrame) -> Json {
  @utils.object([
    ("function_name", @utils.jstr(v.function_name)),
    (
      "env",
      @utils.jarray(
        v.env.map(x => {
          @utils.jarray(
            x.map(x => @utils.jarray([@utils.jstr(x.0), @utils.jint(x.1)])),
          )
        }),
      ),
    ),
    ("ret_locs", @utils.jarray(v.ret_locs.map(x => json_location(x)))),
    ("inner_k", @utils.jarray(v.inner_k.map(x => json_innerframe(x)))),
  ])
}

///|
fn read_fnframe(j : Json) -> FnFrame raise @utils.Fault {
  {
    function_name: @utils.string(@utils.field(j, "function_name")),
    env: @utils.array(@utils.field(j, "env")).map(x => {
      @utils.array(x).map(x => {
        let items = @utils.array(x)
        @utils.require(items.length() == 2, "Invalid tuple")
        (@utils.string(items[0]), @utils.int(items[1]))
      })
    }),
    ret_locs: @utils.array(@utils.field(j, "ret_locs")).map(x => {
      read_location(x)
    }),
    inner_k: @utils.array(@utils.field(j, "inner_k")).map(x => {
      read_innerframe(x)
    }),
  }
}

///|
fn json_state(v : State) -> Json {
  @utils.object([
    ("code", json_code(v.code)),
    ("current_function", @utils.jstr(v.current_function)),
    ("exec", json_exec(v.exec)),
    (
      "env",
      @utils.jarray(
        v.env.map(x => {
          @utils.jarray(
            x.map(x => @utils.jarray([@utils.jstr(x.0), @utils.jint(x.1)])),
          )
        }),
      ),
    ),
    ("store", json_store(v.store)),
    ("inner_k", @utils.jarray(v.inner_k.map(x => json_innerframe(x)))),
    ("fn_k", @utils.jarray(v.fn_k.map(x => json_fnframe(x)))),
    ("next_local_id", @utils.jint(v.next_local_id)),
    ("next_heap_id", @utils.jint(v.next_heap_id)),
  ])
}

///|
fn read_state(j : Json) -> State raise @utils.Fault {
  {
    code: read_code(@utils.field(j, "code")),
    current_function: @utils.string(@utils.field(j, "current_function")),
    exec: read_exec(@utils.field(j, "exec")),
    env: @utils.array(@utils.field(j, "env")).map(x => {
      @utils.array(x).map(x => {
        let items = @utils.array(x)
        @utils.require(items.length() == 2, "Invalid tuple")
        (@utils.string(items[0]), @utils.int(items[1]))
      })
    }),
    store: read_store(@utils.field(j, "store")),
    inner_k: @utils.array(@utils.field(j, "inner_k")).map(x => {
      read_innerframe(x)
    }),
    fn_k: @utils.array(@utils.field(j, "fn_k")).map(x => read_fnframe(x)),
    next_local_id: @utils.int(@utils.field(j, "next_local_id")),
    next_heap_id: @utils.int(@utils.field(j, "next_heap_id")),
  }
}
