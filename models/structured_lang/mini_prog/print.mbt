///|
fn type_text(t : Ty) -> String {
  match t {
    Num => "\\num"
    U8 => "\\u8"
    Bool => "\\bool"
    Unit => "\\unit"
    Ptr => "\\ptr"
    Fn => "\\fn"
    Product(ts) => @utils.join(ts.map(type_text), " * ")
    Sum(ts) => @utils.join(ts.map(type_text), " + ")
    Array(t, n) => "[" + type_text(t) + "; " + n.to_string() + "]"
  }
}

///|
fn op_text(op : Binop) -> String {
  match op {
    Add => "+"
    Sub => "-"
    Eq => "=="
    Lt => "<"
    And => "&&"
    Or => "||"
  }
}

///|
fn place_text(p : Place) -> String {
  match p {
    Static(n) => "\\static " + n
    Local(n) => "\\local " + n
    Field(p, i) => place_text(p) + "." + i.to_string()
    Tag_place(p, i) => place_text(p) + "?" + i.to_string()
    Index(p, v) => place_text(p) + "[" + expr_text(v) + "]"
    Deref(v) => "(" + expr_text(v) + ") \\loc"
  }
}

///|
fn expr_text(v : Expr) -> String {
  match v {
    Number(n) => n.to_string()
    Char(c) => @utils.char_text(c)
    Boolean(b) => if b { "\\true" } else { "\\false" }
    Unit_value => "\\unit"
    NullPtr => "\\null_ptr"
    NullFn => "\\null_fn"
    BinOp(a, op, b) => expr_text(a) + " " + expr_text(b) + " " + op_text(op)
    UnOp(Not, v) => expr_text(v) + " !"
    Pair(vs) =>
      @utils.join(vs.map(expr_text), ", ") +
      " \\pair(" +
      vs.length().to_string() +
      ")"
    Tag(t, v) => expr_text(v) + " \\tag(" + t.to_string() + ")"
    Load(p) => "\\ld " + place_text(p)
    Addr(p) => place_text(p) + " \\addr"
    Function(n) => "\\fn " + n
  }
}

///|
fn stmt_text(st : Stmt) -> String {
  match st {
    Assign(p, v) => "\\assign " + place_text(p) + " := " + expr_text(v)
    If(c, s) => "\\if " + expr_text(c) + " " + stmt_text(s)
    Case(t, v, s) =>
      "\\case " + t.to_string() + " \\of " + expr_text(v) + " " + stmt_text(s)
    HAlloc(t, p) => "\\halloc " + type_text(t) + " -> " + place_text(p)
    HFree(v) => "\\hfree " + expr_text(v)
    Loop(l, s) => "\\loop " + l + ": " + stmt_text(s)
    Break(l) => "\\break " + l
    Continue(l) => "\\continue " + l
    Call(f, args, rets) =>
      "\\call " +
      expr_text(f) +
      "(" +
      @utils.join(args.map(expr_text), ", ") +
      ") -> " +
      @utils.join(rets.map(place_text), ", ")
    Return(vs) =>
      "\\return" +
      (if vs.is_empty() {
        ""
      } else {
        " " + @utils.join(vs.map(expr_text), ", ")
      })
    Block(b) => "\\block " + block_text(b)
  }
}

///|
fn block_text(b : Block) -> String {
  "(" +
  @utils.join(
    b.bindings.map(binding => binding.0 + ": " + type_text(binding.1)),
    ", ",
  ) +
  ") { " +
  @utils.join(b.stmts.map(stmt_text), "; ") +
  " }"
}

///|
fn key_text(key : Key) -> String {
  match key {
    Heap(n) => "heap#" + n.to_string()
    Static_key(n) => "static " + n
    Local_key(n) => "local#" + n.to_string()
  }
}

///|
fn location_text(loc : Location) -> String {
  key_text(loc.key) +
  @utils.join(
    loc.projections.map(p => {
      match p {
        Field_proj(n) => "." + n.to_string()
        Tag_proj(n) => "?" + n.to_string()
        Index_proj(n) => "[" + n.to_string() + "]"
      }
    }),
    "",
  )
}

///|
fn value_text(v : Value) -> String {
  match v {
    Number_value(n) => n.to_string()
    Char_value(c) => "'" + c + "'"
    Bool_value(b) => if b { "\\true" } else { "\\false" }
    Unit_v => "\\unit"
    Ptr_value(None) => "\\null_ptr"
    Ptr_value(Some(l)) => location_text(l)
    FnPtr(None) => "\\null_fn"
    FnPtr(Some(n)) => "fn " + n
    Aggregate(vs) => "(" + @utils.join(vs.map(value_text), ", ") + ")"
    Sum_value(t, v) => "tag(" + t.to_string() + ", " + value_text(v) + ")"
  }
}

///|
fn exec_text(exec : Exec) -> String {
  match exec {
    Eval(s) => "eval " + stmt_text(s)
    Ctrl(Next) => "ctrl next"
    Ctrl(Break_ctrl(l)) => "ctrl break " + l
    Ctrl(Continue_ctrl(l)) => "ctrl continue " + l
    Ret(vs) => "ret " + @utils.join(vs.map(value_text), ", ")
  }
}

///|
fn inner_text(f : InnerFrame) -> String {
  match f {
    Stmt_frame(s) => "stmt " + stmt_text(s)
    Scope => "scope"
    Loop_frame(l, s) => "loop " + l + ": " + stmt_text(s)
  }
}

///|
fn cell_rows(cells : Array[(Int32, Cell)]) -> Array[Array[String]] {
  let cells = cells.copy()
  cells.sort_by((a, b) => a.0.compare(b.0))
  cells.map(entry => {
    [entry.0.to_string(), type_text(entry.1.ty), value_text(entry.1.value)]
  })
}

///|
fn render(s : State) -> Array[@definition.Block] {
  let env = []
  for index, scope in s.env {
    for entry in @utils.env_sorted(scope) {
      env.push([index.to_string(), entry.0, entry.1.to_string()])
    }
  }
  [
    @definition.text_table("status", ["field", "value"], [
      ["function", s.current_function],
      ["exec", exec_text(s.exec)],
      ["next_local_id", s.next_local_id.to_string()],
      ["next_heap_id", s.next_heap_id.to_string()],
    ]),
    @definition.text_table("env", ["scope", "name", "local"], env),
    @definition.text_table(
      "statics",
      ["name", "type", "value"],
      @utils.env_sorted(s.store.statics).map(entry => {
        [entry.0, type_text(entry.1.ty), value_text(entry.1.value)]
      }),
    ),
    @definition.text_table(
      "heap",
      ["heap", "type", "value"],
      cell_rows(s.store.heap),
    ),
    @definition.container(
      "locals",
      s.store.locals.mapi((i, scopes) => {
        @definition.container(
          "fn frame " + i.to_string(),
          scopes.mapi((j, cells) => {
            @definition.text_table(
              "scope " + j.to_string(),
              ["local", "type", "value"],
              cell_rows(cells),
            )
          }),
        )
      }),
    ),
    @definition.text_table(
      "inner_k",
      ["depth", "frame"],
      s.inner_k.mapi((i, f) => [i.to_string(), inner_text(f)]),
    ),
    @definition.text_table(
      "fn_k",
      ["depth", "function", "rets", "resume"],
      s.fn_k.mapi((i, f) => {
        [
          i.to_string(),
          f.function_name,
          f.ret_locs.length().to_string(),
          f.inner_k.length().to_string(),
        ]
      }),
    ),
  ]
}
