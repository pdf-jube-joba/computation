///|
fn nat_json(n : @base.Nat) -> Json {
  @base.jstr(n.to_string())
}

///|
fn json_ty(v : Ty) -> Json {
  match v {
    Num => @base.tag("Num", [])
    U8 => @base.tag("U8", [])
    Bool => @base.tag("Bool", [])
    Unit => @base.tag("Unit", [])
    Ptr => @base.tag("Ptr", [])
    Fn => @base.tag("Fn", [])
    Product(v0) => @base.tag("Product", [@base.jarray(v0.map(x => json_ty(x)))])
    Sum(v0) => @base.tag("Sum", [@base.jarray(v0.map(x => json_ty(x)))])
    Array(v0, v1) => @base.tag("Array", [json_ty(v0), @base.jint(v1)])
  }
}

///|
fn read_ty(j : Json) -> Ty raise @base.Fault {
  let tag = @base.tag_name(j)
  let a = @base.array(j)[1:].to_owned()
  match tag {
    "Num" => {
      @base.require(a.length() == 0, "Invalid Num")
      Num
    }
    "U8" => {
      @base.require(a.length() == 0, "Invalid U8")
      U8
    }
    "Bool" => {
      @base.require(a.length() == 0, "Invalid Bool")
      Bool
    }
    "Unit" => {
      @base.require(a.length() == 0, "Invalid Unit")
      Unit
    }
    "Ptr" => {
      @base.require(a.length() == 0, "Invalid Ptr")
      Ptr
    }
    "Fn" => {
      @base.require(a.length() == 0, "Invalid Fn")
      Fn
    }
    "Product" => {
      @base.require(a.length() == 1, "Invalid Product")
      Product(@base.array(a[0]).map(x => read_ty(x)))
    }
    "Sum" => {
      @base.require(a.length() == 1, "Invalid Sum")
      Sum(@base.array(a[0]).map(x => read_ty(x)))
    }
    "Array" => {
      @base.require(a.length() == 2, "Invalid Array")
      Array(read_ty(a[0]), @base.int(a[1]))
    }
    _ => @base.error("Invalid Ty tag: " + tag)
  }
}

///|
fn json_binop(v : Binop) -> Json {
  match v {
    Add => @base.tag("Add", [])
    Sub => @base.tag("Sub", [])
    Eq => @base.tag("Eq", [])
    Lt => @base.tag("Lt", [])
    And => @base.tag("And", [])
    Or => @base.tag("Or", [])
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
    "Eq" => {
      @base.require(a.length() == 0, "Invalid Eq")
      Eq
    }
    "Lt" => {
      @base.require(a.length() == 0, "Invalid Lt")
      Lt
    }
    "And" => {
      @base.require(a.length() == 0, "Invalid And")
      And
    }
    "Or" => {
      @base.require(a.length() == 0, "Invalid Or")
      Or
    }
    _ => @base.error("Invalid Binop tag: " + tag)
  }
}

///|
fn json_unop(v : Unop) -> Json {
  match v {
    Not => @base.tag("Not", [])
  }
}

///|
fn read_unop(j : Json) -> Unop raise @base.Fault {
  let tag = @base.tag_name(j)
  let a = @base.array(j)[1:].to_owned()
  match tag {
    "Not" => {
      @base.require(a.length() == 0, "Invalid Not")
      Not
    }
    _ => @base.error("Invalid Unop tag: " + tag)
  }
}

///|
fn json_place(v : Place) -> Json {
  match v {
    Static(v0) => @base.tag("Static", [@base.jstr(v0)])
    Local(v0) => @base.tag("Local", [@base.jstr(v0)])
    Field(v0, v1) => @base.tag("Field", [json_place(v0), @base.jint(v1)])
    Tag_place(v0, v1) =>
      @base.tag("Tag_place", [json_place(v0), @base.jint(v1)])
    Index(v0, v1) => @base.tag("Index", [json_place(v0), json_expr(v1)])
    Deref(v0) => @base.tag("Deref", [json_expr(v0)])
  }
}

///|
fn read_place(j : Json) -> Place raise @base.Fault {
  let tag = @base.tag_name(j)
  let a = @base.array(j)[1:].to_owned()
  match tag {
    "Static" => {
      @base.require(a.length() == 1, "Invalid Static")
      Static(@base.string(a[0]))
    }
    "Local" => {
      @base.require(a.length() == 1, "Invalid Local")
      Local(@base.string(a[0]))
    }
    "Field" => {
      @base.require(a.length() == 2, "Invalid Field")
      Field(read_place(a[0]), @base.int(a[1]))
    }
    "Tag_place" => {
      @base.require(a.length() == 2, "Invalid Tag_place")
      Tag_place(read_place(a[0]), @base.int(a[1]))
    }
    "Index" => {
      @base.require(a.length() == 2, "Invalid Index")
      Index(read_place(a[0]), read_expr(a[1]))
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
    Number(v0) => @base.tag("Number", [nat_json(v0)])
    Char(v0) => @base.tag("Char", [@base.jstr(v0)])
    Boolean(v0) => @base.tag("Boolean", [@base.jbool(v0)])
    Unit_value => @base.tag("Unit_value", [])
    NullPtr => @base.tag("NullPtr", [])
    NullFn => @base.tag("NullFn", [])
    BinOp(v0, v1, v2) =>
      @base.tag("BinOp", [json_expr(v0), json_binop(v1), json_expr(v2)])
    UnOp(v0, v1) => @base.tag("UnOp", [json_unop(v0), json_expr(v1)])
    Pair(v0) => @base.tag("Pair", [@base.jarray(v0.map(x => json_expr(x)))])
    Tag(v0, v1) => @base.tag("Tag", [@base.jint(v0), json_expr(v1)])
    Load(v0) => @base.tag("Load", [json_place(v0)])
    Addr(v0) => @base.tag("Addr", [json_place(v0)])
    Function(v0) => @base.tag("Function", [@base.jstr(v0)])
  }
}

///|
fn read_expr(j : Json) -> Expr raise @base.Fault {
  let tag = @base.tag_name(j)
  let a = @base.array(j)[1:].to_owned()
  match tag {
    "Number" => {
      @base.require(a.length() == 1, "Invalid Number")
      Number(@base.json_nat(a[0]))
    }
    "Char" => {
      @base.require(a.length() == 1, "Invalid Char")
      Char(@base.string(a[0]))
    }
    "Boolean" => {
      @base.require(a.length() == 1, "Invalid Boolean")
      Boolean(@base.boolean(a[0]))
    }
    "Unit_value" => {
      @base.require(a.length() == 0, "Invalid Unit_value")
      Unit_value
    }
    "NullPtr" => {
      @base.require(a.length() == 0, "Invalid NullPtr")
      NullPtr
    }
    "NullFn" => {
      @base.require(a.length() == 0, "Invalid NullFn")
      NullFn
    }
    "BinOp" => {
      @base.require(a.length() == 3, "Invalid BinOp")
      BinOp(read_expr(a[0]), read_binop(a[1]), read_expr(a[2]))
    }
    "UnOp" => {
      @base.require(a.length() == 2, "Invalid UnOp")
      UnOp(read_unop(a[0]), read_expr(a[1]))
    }
    "Pair" => {
      @base.require(a.length() == 1, "Invalid Pair")
      Pair(@base.array(a[0]).map(x => read_expr(x)))
    }
    "Tag" => {
      @base.require(a.length() == 2, "Invalid Tag")
      Tag(@base.int(a[0]), read_expr(a[1]))
    }
    "Load" => {
      @base.require(a.length() == 1, "Invalid Load")
      Load(read_place(a[0]))
    }
    "Addr" => {
      @base.require(a.length() == 1, "Invalid Addr")
      Addr(read_place(a[0]))
    }
    "Function" => {
      @base.require(a.length() == 1, "Invalid Function")
      Function(@base.string(a[0]))
    }
    _ => @base.error("Invalid Expr tag: " + tag)
  }
}

///|
fn json_stmt(v : Stmt) -> Json {
  match v {
    Assign(v0, v1) => @base.tag("Assign", [json_place(v0), json_expr(v1)])
    If(v0, v1) => @base.tag("If", [json_expr(v0), json_stmt(v1)])
    Case(v0, v1, v2) =>
      @base.tag("Case", [@base.jint(v0), json_expr(v1), json_stmt(v2)])
    HAlloc(v0, v1) => @base.tag("HAlloc", [json_ty(v0), json_place(v1)])
    HFree(v0) => @base.tag("HFree", [json_expr(v0)])
    Loop(v0, v1) => @base.tag("Loop", [@base.jstr(v0), json_stmt(v1)])
    Break(v0) => @base.tag("Break", [@base.jstr(v0)])
    Continue(v0) => @base.tag("Continue", [@base.jstr(v0)])
    Call(v0, v1, v2) =>
      @base.tag("Call", [
        json_expr(v0),
        @base.jarray(v1.map(x => json_expr(x))),
        @base.jarray(v2.map(x => json_place(x))),
      ])
    Return(v0) => @base.tag("Return", [@base.jarray(v0.map(x => json_expr(x)))])
    Block(v0) => @base.tag("Block", [json_block(v0)])
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
    "If" => {
      @base.require(a.length() == 2, "Invalid If")
      If(read_expr(a[0]), read_stmt(a[1]))
    }
    "Case" => {
      @base.require(a.length() == 3, "Invalid Case")
      Case(@base.int(a[0]), read_expr(a[1]), read_stmt(a[2]))
    }
    "HAlloc" => {
      @base.require(a.length() == 2, "Invalid HAlloc")
      HAlloc(read_ty(a[0]), read_place(a[1]))
    }
    "HFree" => {
      @base.require(a.length() == 1, "Invalid HFree")
      HFree(read_expr(a[0]))
    }
    "Loop" => {
      @base.require(a.length() == 2, "Invalid Loop")
      Loop(@base.string(a[0]), read_stmt(a[1]))
    }
    "Break" => {
      @base.require(a.length() == 1, "Invalid Break")
      Break(@base.string(a[0]))
    }
    "Continue" => {
      @base.require(a.length() == 1, "Invalid Continue")
      Continue(@base.string(a[0]))
    }
    "Call" => {
      @base.require(a.length() == 3, "Invalid Call")
      Call(
        read_expr(a[0]),
        @base.array(a[1]).map(x => read_expr(x)),
        @base.array(a[2]).map(x => read_place(x)),
      )
    }
    "Return" => {
      @base.require(a.length() == 1, "Invalid Return")
      Return(@base.array(a[0]).map(x => read_expr(x)))
    }
    "Block" => {
      @base.require(a.length() == 1, "Invalid Block")
      Block(read_block(a[0]))
    }
    _ => @base.error("Invalid Stmt tag: " + tag)
  }
}

///|
fn json_key(v : Key) -> Json {
  match v {
    Heap(v0) => @base.tag("Heap", [@base.jint(v0)])
    Static_key(v0) => @base.tag("Static_key", [@base.jstr(v0)])
    Local_key(v0) => @base.tag("Local_key", [@base.jint(v0)])
  }
}

///|
fn read_key(j : Json) -> Key raise @base.Fault {
  let tag = @base.tag_name(j)
  let a = @base.array(j)[1:].to_owned()
  match tag {
    "Heap" => {
      @base.require(a.length() == 1, "Invalid Heap")
      Heap(@base.int(a[0]))
    }
    "Static_key" => {
      @base.require(a.length() == 1, "Invalid Static_key")
      Static_key(@base.string(a[0]))
    }
    "Local_key" => {
      @base.require(a.length() == 1, "Invalid Local_key")
      Local_key(@base.int(a[0]))
    }
    _ => @base.error("Invalid Key tag: " + tag)
  }
}

///|
fn json_projection(v : Projection) -> Json {
  match v {
    Field_proj(v0) => @base.tag("Field_proj", [@base.jint(v0)])
    Tag_proj(v0) => @base.tag("Tag_proj", [@base.jint(v0)])
    Index_proj(v0) => @base.tag("Index_proj", [@base.jint(v0)])
  }
}

///|
fn read_projection(j : Json) -> Projection raise @base.Fault {
  let tag = @base.tag_name(j)
  let a = @base.array(j)[1:].to_owned()
  match tag {
    "Field_proj" => {
      @base.require(a.length() == 1, "Invalid Field_proj")
      Field_proj(@base.int(a[0]))
    }
    "Tag_proj" => {
      @base.require(a.length() == 1, "Invalid Tag_proj")
      Tag_proj(@base.int(a[0]))
    }
    "Index_proj" => {
      @base.require(a.length() == 1, "Invalid Index_proj")
      Index_proj(@base.int(a[0]))
    }
    _ => @base.error("Invalid Projection tag: " + tag)
  }
}

///|
fn json_value(v : Value) -> Json {
  match v {
    Number_value(v0) => @base.tag("Number_value", [nat_json(v0)])
    Char_value(v0) => @base.tag("Char_value", [@base.jstr(v0)])
    Bool_value(v0) => @base.tag("Bool_value", [@base.jbool(v0)])
    Unit_v => @base.tag("Unit_v", [])
    Ptr_value(v0) =>
      @base.tag("Ptr_value", [
        match v0 {
          Some(x) => json_location(x)
          None => Json::null()
        },
      ])
    FnPtr(v0) =>
      @base.tag("FnPtr", [
        match v0 {
          Some(x) => @base.jstr(x)
          None => Json::null()
        },
      ])
    Aggregate(v0) =>
      @base.tag("Aggregate", [@base.jarray(v0.map(x => json_value(x)))])
    Sum_value(v0, v1) =>
      @base.tag("Sum_value", [@base.jint(v0), json_value(v1)])
  }
}

///|
fn read_value(j : Json) -> Value raise @base.Fault {
  let tag = @base.tag_name(j)
  let a = @base.array(j)[1:].to_owned()
  match tag {
    "Number_value" => {
      @base.require(a.length() == 1, "Invalid Number_value")
      Number_value(@base.json_nat(a[0]))
    }
    "Char_value" => {
      @base.require(a.length() == 1, "Invalid Char_value")
      Char_value(@base.string(a[0]))
    }
    "Bool_value" => {
      @base.require(a.length() == 1, "Invalid Bool_value")
      Bool_value(@base.boolean(a[0]))
    }
    "Unit_v" => {
      @base.require(a.length() == 0, "Invalid Unit_v")
      Unit_v
    }
    "Ptr_value" => {
      @base.require(a.length() == 1, "Invalid Ptr_value")
      Ptr_value(
        match a[0] {
          Null => None
          _ => Some(read_location(a[0]))
        },
      )
    }
    "FnPtr" => {
      @base.require(a.length() == 1, "Invalid FnPtr")
      FnPtr(
        match a[0] {
          Null => None
          _ => Some(@base.string(a[0]))
        },
      )
    }
    "Aggregate" => {
      @base.require(a.length() == 1, "Invalid Aggregate")
      Aggregate(@base.array(a[0]).map(x => read_value(x)))
    }
    "Sum_value" => {
      @base.require(a.length() == 2, "Invalid Sum_value")
      Sum_value(@base.int(a[0]), read_value(a[1]))
    }
    _ => @base.error("Invalid Value tag: " + tag)
  }
}

///|
fn json_control(v : Control) -> Json {
  match v {
    Next => @base.tag("Next", [])
    Break_ctrl(v0) => @base.tag("Break_ctrl", [@base.jstr(v0)])
    Continue_ctrl(v0) => @base.tag("Continue_ctrl", [@base.jstr(v0)])
  }
}

///|
fn read_control(j : Json) -> Control raise @base.Fault {
  let tag = @base.tag_name(j)
  let a = @base.array(j)[1:].to_owned()
  match tag {
    "Next" => {
      @base.require(a.length() == 0, "Invalid Next")
      Next
    }
    "Break_ctrl" => {
      @base.require(a.length() == 1, "Invalid Break_ctrl")
      Break_ctrl(@base.string(a[0]))
    }
    "Continue_ctrl" => {
      @base.require(a.length() == 1, "Invalid Continue_ctrl")
      Continue_ctrl(@base.string(a[0]))
    }
    _ => @base.error("Invalid Control tag: " + tag)
  }
}

///|
fn json_exec(v : Exec) -> Json {
  match v {
    Eval(v0) => @base.tag("Eval", [json_stmt(v0)])
    Ctrl(v0) => @base.tag("Ctrl", [json_control(v0)])
    Ret(v0) => @base.tag("Ret", [@base.jarray(v0.map(x => json_value(x)))])
  }
}

///|
fn read_exec(j : Json) -> Exec raise @base.Fault {
  let tag = @base.tag_name(j)
  let a = @base.array(j)[1:].to_owned()
  match tag {
    "Eval" => {
      @base.require(a.length() == 1, "Invalid Eval")
      Eval(read_stmt(a[0]))
    }
    "Ctrl" => {
      @base.require(a.length() == 1, "Invalid Ctrl")
      Ctrl(read_control(a[0]))
    }
    "Ret" => {
      @base.require(a.length() == 1, "Invalid Ret")
      Ret(@base.array(a[0]).map(x => read_value(x)))
    }
    _ => @base.error("Invalid Exec tag: " + tag)
  }
}

///|
fn json_innerframe(v : InnerFrame) -> Json {
  match v {
    Stmt_frame(v0) => @base.tag("Stmt_frame", [json_stmt(v0)])
    Scope => @base.tag("Scope", [])
    Loop_frame(v0, v1) =>
      @base.tag("Loop_frame", [@base.jstr(v0), json_stmt(v1)])
  }
}

///|
fn read_innerframe(j : Json) -> InnerFrame raise @base.Fault {
  let tag = @base.tag_name(j)
  let a = @base.array(j)[1:].to_owned()
  match tag {
    "Stmt_frame" => {
      @base.require(a.length() == 1, "Invalid Stmt_frame")
      Stmt_frame(read_stmt(a[0]))
    }
    "Scope" => {
      @base.require(a.length() == 0, "Invalid Scope")
      Scope
    }
    "Loop_frame" => {
      @base.require(a.length() == 2, "Invalid Loop_frame")
      Loop_frame(@base.string(a[0]), read_stmt(a[1]))
    }
    _ => @base.error("Invalid InnerFrame tag: " + tag)
  }
}

///|
fn json_block(v : Block) -> Json {
  @base.object([
    (
      "bindings",
      @base.jarray(
        v.bindings.map(x => @base.jarray([@base.jstr(x.0), json_ty(x.1)])),
      ),
    ),
    ("stmts", @base.jarray(v.stmts.map(x => json_stmt(x)))),
  ])
}

///|
fn read_block(j : Json) -> Block raise @base.Fault {
  {
    bindings: @base.array(@base.field(j, "bindings")).map(x => {
      let items = @base.array(x)
      @base.require(items.length() == 2, "Invalid tuple")
      (@base.string(items[0]), read_ty(items[1]))
    }),
    stmts: @base.array(@base.field(j, "stmts")).map(x => read_stmt(x)),
  }
}

///|
fn json_staticdecl(v : StaticDecl) -> Json {
  @base.object([("name", @base.jstr(v.name)), ("ty", json_ty(v.ty))])
}

///|
fn read_staticdecl(j : Json) -> StaticDecl raise @base.Fault {
  {
    name: @base.string(@base.field(j, "name")),
    ty: read_ty(@base.field(j, "ty")),
  }
}

///|
fn json_functiondecl(v : FunctionDecl) -> Json {
  @base.object([("name", @base.jstr(v.name)), ("block", json_block(v.block))])
}

///|
fn read_functiondecl(j : Json) -> FunctionDecl raise @base.Fault {
  {
    name: @base.string(@base.field(j, "name")),
    block: read_block(@base.field(j, "block")),
  }
}

///|
fn json_code(v : Code) -> Json {
  @base.object([
    ("statics", @base.jarray(v.statics.map(x => json_staticdecl(x)))),
    ("functions", @base.jarray(v.functions.map(x => json_functiondecl(x)))),
  ])
}

///|
fn read_code(j : Json) -> Code raise @base.Fault {
  {
    statics: @base.array(@base.field(j, "statics")).map(x => read_staticdecl(x)),
    functions: @base.array(@base.field(j, "functions")).map(x => {
      read_functiondecl(x)
    }),
  }
}

///|
fn json_location(v : Location) -> Json {
  @base.object([
    ("key", json_key(v.key)),
    ("projections", @base.jarray(v.projections.map(x => json_projection(x)))),
  ])
}

///|
fn read_location(j : Json) -> Location raise @base.Fault {
  {
    key: read_key(@base.field(j, "key")),
    projections: @base.array(@base.field(j, "projections")).map(x => {
      read_projection(x)
    }),
  }
}

///|
fn json_cell(v : Cell) -> Json {
  @base.object([("ty", json_ty(v.ty)), ("value", json_value(v.value))])
}

///|
fn read_cell(j : Json) -> Cell raise @base.Fault {
  {
    ty: read_ty(@base.field(j, "ty")),
    value: read_value(@base.field(j, "value")),
  }
}

///|
fn json_store(v : Store) -> Json {
  @base.object([
    (
      "heap",
      @base.jarray(
        v.heap.map(x => @base.jarray([@base.jint(x.0), json_cell(x.1)])),
      ),
    ),
    (
      "statics",
      @base.jarray(
        v.statics.map(x => @base.jarray([@base.jstr(x.0), json_cell(x.1)])),
      ),
    ),
    (
      "locals",
      @base.jarray(
        v.locals.map(x => {
          @base.jarray(
            x.map(x => {
              @base.jarray(
                x.map(x => @base.jarray([@base.jint(x.0), json_cell(x.1)])),
              )
            }),
          )
        }),
      ),
    ),
  ])
}

///|
fn read_store(j : Json) -> Store raise @base.Fault {
  {
    heap: @base.array(@base.field(j, "heap")).map(x => {
      let items = @base.array(x)
      @base.require(items.length() == 2, "Invalid tuple")
      (@base.int(items[0]), read_cell(items[1]))
    }),
    statics: @base.array(@base.field(j, "statics")).map(x => {
      let items = @base.array(x)
      @base.require(items.length() == 2, "Invalid tuple")
      (@base.string(items[0]), read_cell(items[1]))
    }),
    locals: @base.array(@base.field(j, "locals")).map(x => {
      @base.array(x).map(x => {
        @base.array(x).map(x => {
          let items = @base.array(x)
          @base.require(items.length() == 2, "Invalid tuple")
          (@base.int(items[0]), read_cell(items[1]))
        })
      })
    }),
  }
}

///|
fn json_fnframe(v : FnFrame) -> Json {
  @base.object([
    ("function_name", @base.jstr(v.function_name)),
    (
      "env",
      @base.jarray(
        v.env.map(x => {
          @base.jarray(
            x.map(x => @base.jarray([@base.jstr(x.0), @base.jint(x.1)])),
          )
        }),
      ),
    ),
    ("ret_locs", @base.jarray(v.ret_locs.map(x => json_location(x)))),
    ("inner_k", @base.jarray(v.inner_k.map(x => json_innerframe(x)))),
  ])
}

///|
fn read_fnframe(j : Json) -> FnFrame raise @base.Fault {
  {
    function_name: @base.string(@base.field(j, "function_name")),
    env: @base.array(@base.field(j, "env")).map(x => {
      @base.array(x).map(x => {
        let items = @base.array(x)
        @base.require(items.length() == 2, "Invalid tuple")
        (@base.string(items[0]), @base.int(items[1]))
      })
    }),
    ret_locs: @base.array(@base.field(j, "ret_locs")).map(x => read_location(x)),
    inner_k: @base.array(@base.field(j, "inner_k")).map(x => read_innerframe(x)),
  }
}

///|
fn json_state(v : State) -> Json {
  @base.object([
    ("code", json_code(v.code)),
    ("current_function", @base.jstr(v.current_function)),
    ("exec", json_exec(v.exec)),
    (
      "env",
      @base.jarray(
        v.env.map(x => {
          @base.jarray(
            x.map(x => @base.jarray([@base.jstr(x.0), @base.jint(x.1)])),
          )
        }),
      ),
    ),
    ("store", json_store(v.store)),
    ("inner_k", @base.jarray(v.inner_k.map(x => json_innerframe(x)))),
    ("fn_k", @base.jarray(v.fn_k.map(x => json_fnframe(x)))),
    ("next_local_id", @base.jint(v.next_local_id)),
    ("next_heap_id", @base.jint(v.next_heap_id)),
  ])
}

///|
fn read_state(j : Json) -> State raise @base.Fault {
  {
    code: read_code(@base.field(j, "code")),
    current_function: @base.string(@base.field(j, "current_function")),
    exec: read_exec(@base.field(j, "exec")),
    env: @base.array(@base.field(j, "env")).map(x => {
      @base.array(x).map(x => {
        let items = @base.array(x)
        @base.require(items.length() == 2, "Invalid tuple")
        (@base.string(items[0]), @base.int(items[1]))
      })
    }),
    store: read_store(@base.field(j, "store")),
    inner_k: @base.array(@base.field(j, "inner_k")).map(x => read_innerframe(x)),
    fn_k: @base.array(@base.field(j, "fn_k")).map(x => read_fnframe(x)),
    next_local_id: @base.int(@base.field(j, "next_local_id")),
    next_heap_id: @base.int(@base.field(j, "next_heap_id")),
  }
}
