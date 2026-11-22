///|
priv enum Ty {
  Num
  U8
  Bool
  Unit
  Ptr
  Fn
  Product(Array[Ty])
  Sum(Array[Ty])
  Array(Ty, Int32)
}

///|
priv enum Binop {
  Add
  Sub
  Eq
  Lt
  And
  Or
}

///|
priv enum Unop {
  Not
}

///|
priv enum Place {
  Static(String)
  Local(String)
  Field(Place, Int32)
  Tag_place(Place, Int32)
  Index(Place, Expr)
  Deref(Expr)
}

///|
priv enum Expr {
  Number(@utils.Nat)
  Char(String)
  Boolean(Bool)
  Unit_value
  NullPtr
  NullFn
  BinOp(Expr, Binop, Expr)
  UnOp(Unop, Expr)
  Pair(Array[Expr])
  Tag(Int32, Expr)
  Load(Place)
  Addr(Place)
  Function(String)
}

///|
priv enum Stmt {
  Assign(Place, Expr)
  If(Expr, Stmt)
  Case(Int32, Expr, Stmt)
  HAlloc(Ty, Place)
  HFree(Expr)
  Loop(String, Stmt)
  Break(String)
  Continue(String)
  Call(Expr, Array[Expr], Array[Place])
  Return(Array[Expr])
  Block(Block)
}

///|
#derive(Eq)
priv enum Key {
  Heap(Int32)
  Static_key(String)
  Local_key(Int32)
}

///|
#derive(Eq)
priv enum Projection {
  Field_proj(Int32)
  Tag_proj(Int32)
  Index_proj(Int32)
}

///|
priv enum Value {
  Number_value(@utils.Nat)
  Char_value(String)
  Bool_value(Bool)
  Unit_v
  Ptr_value(Location?)
  FnPtr(String?)
  Aggregate(Array[Value])
  Sum_value(Int32, Value)
}

///|
priv enum Control {
  Next
  Break_ctrl(String)
  Continue_ctrl(String)
}

///|
priv enum Exec {
  Eval(Stmt)
  Ctrl(Control)
  Ret(Array[Value])
}

///|
priv enum InnerFrame {
  Stmt_frame(Stmt)
  Scope
  Loop_frame(String, Stmt)
}

///|
priv struct Block {
  bindings : Array[(String, Ty)]
  stmts : Array[Stmt]
}

///|
priv struct StaticDecl {
  name : String
  ty : Ty
}

///|
priv struct FunctionDecl {
  name : String
  block : Block
}

///|
struct Code {
  statics : Array[StaticDecl]
  functions : Array[FunctionDecl]
}

///|
#derive(Eq)
priv struct Location {
  key : Key
  projections : Array[Projection]
}

///|
priv struct Cell {
  ty : Ty
  value : Value
}

///|
priv struct Store {
  heap : Array[(Int32, Cell)]
  statics : Array[(String, Cell)]
  locals : Array[Array[Array[(Int32, Cell)]]]
}

///|
priv struct FnFrame {
  function_name : String
  env : Array[Array[(String, Int32)]]
  ret_locs : Array[Location]
  inner_k : Array[InnerFrame]
}

///|
struct State {
  code : Code
  current_function : String
  exec : Exec
  env : Array[Array[(String, Int32)]]
  store : Store
  inner_k : Array[InnerFrame]
  fn_k : Array[FnFrame]
  next_local_id : Int32
  next_heap_id : Int32
}
