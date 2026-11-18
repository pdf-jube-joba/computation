///|
fn syntax_value_text(v : ValueExpr) -> String {
  match v {
    Reg(r) => "%" + r
    Imm(n) => n.to_string()
    Label(l) => ":" + l
    Memory(i) => "MEM_" + i.to_string()
  }
}

///|
fn syntax_place_text(p : Place) -> String {
  match p {
    Deref(v) => "\\deref " + syntax_value_text(v)
  }
}

///|
fn syntax_rhs_text(r : Rhs) -> String {
  match r {
    Value(v) => syntax_value_text(v)
    Binop(a, op, b) =>
      syntax_value_text(a) +
      (match op {
        Add => " + "
        Sub => " - "
      }) +
      syntax_value_text(b)
    Load(p) => "\\ld " + syntax_place_text(p)
    Slice(s, b, e) =>
      "\\slice " +
      syntax_value_text(s) +
      " " +
      syntax_value_text(b) +
      " " +
      syntax_value_text(e)
    Addr(s, i) => "\\addr " + syntax_value_text(s) + " " + syntax_value_text(i)
    Len(s) => "\\len " + syntax_value_text(s)
  }
}

///|
fn syntax_stmt_text(s : Stmt) -> String {
  match s {
    Assign(r, rhs) => "%" + r + " := " + syntax_rhs_text(rhs)
    Store(p, v) => syntax_place_text(p) + " := \\st " + syntax_value_text(v)
    Print(v) => "\\print " + syntax_value_text(v)
    Input(p) => "\\input " + syntax_place_text(p)
  }
}

///|
pub fn print_code(code : Code) -> String {
  "\\memories " +
  code.memories.to_string() +
  ";\n" +
  @base.join(
    code.blocks.map(b => {
      ":" +
      b.label +
      " {\n" +
      @base.join(b.stmts.map(s => "  " + syntax_stmt_text(s) + ";\n"), "") +
      @base.join(
        b.ifs.map(j => {
          "  \\if " +
          syntax_value_text(j.cond.0) +
          (match j.cond.1 {
            Eq => " == "
            Lt => " < "
            Gt => " > "
          }) +
          syntax_value_text(j.cond.2) +
          " \\then " +
          syntax_value_text(j.target) +
          ";\n"
        }),
        "",
      ) +
      "  " +
      (match b.ending {
        Goto(v) => "\\goto " + syntax_value_text(v)
        Halt => "\\halt"
      }) +
      ";\n}\n"
    }),
    "\n",
  )
}
