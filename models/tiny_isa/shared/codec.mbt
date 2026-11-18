///|
pub fn[T] instr_json(inst : Instr[T], value : (T) -> Json) -> Json {
  match inst {
    Nop => @base.tag("Nop", [])
    Halt => @base.tag("Halt", [])
    Rst => @base.tag("Rst", [])
    Ld(v0, v1) => @base.tag("Ld", [@base.jint(v0), @base.jint(v1)])
    St(v0, v1) => @base.tag("St", [@base.jint(v0), @base.jint(v1)])
    Mov(v0, v1, v2) =>
      @base.tag("Mov", [@base.jbool(v0), @base.jint(v1), @base.jint(v2)])
    Ldi(v0, v1, v2) =>
      @base.tag("Ldi", [@base.jbool(v0), @base.jint(v1), value(v2)])
    Add(v0, v1, v2) =>
      @base.tag("Add", [@base.jbool(v0), @base.jint(v1), @base.jint(v2)])
    Sub(v0, v1, v2) =>
      @base.tag("Sub", [@base.jbool(v0), @base.jint(v1), @base.jint(v2)])
    Addi(v0, v1, v2) =>
      @base.tag("Addi", [@base.jbool(v0), @base.jint(v1), value(v2)])
    Subi(v0, v1, v2) =>
      @base.tag("Subi", [@base.jbool(v0), @base.jint(v1), value(v2)])
    Eq(v0, v1) => @base.tag("Eq", [@base.jint(v0), @base.jint(v1)])
    Lt(v0, v1) => @base.tag("Lt", [@base.jint(v0), @base.jint(v1)])
    Gt(v0, v1) => @base.tag("Gt", [@base.jint(v0), @base.jint(v1)])
  }
}

///|
pub fn[T] decode_instr(
  j : Json,
  value : (Json) -> T raise @base.Fault,
) -> Instr[T] raise @base.Fault {
  match @base.tag_name(j) {
    "Nop" => Nop
    "Halt" => Halt
    "Rst" => Rst
    "Ld" => Ld(@base.int(@base.tag_arg(j, 0)), @base.int(@base.tag_arg(j, 1)))
    "St" => St(@base.int(@base.tag_arg(j, 0)), @base.int(@base.tag_arg(j, 1)))
    "Mov" =>
      Mov(
        @base.boolean(@base.tag_arg(j, 0)),
        @base.int(@base.tag_arg(j, 1)),
        @base.int(@base.tag_arg(j, 2)),
      )
    "Ldi" =>
      Ldi(
        @base.boolean(@base.tag_arg(j, 0)),
        @base.int(@base.tag_arg(j, 1)),
        value(@base.tag_arg(j, 2)),
      )
    "Add" =>
      Add(
        @base.boolean(@base.tag_arg(j, 0)),
        @base.int(@base.tag_arg(j, 1)),
        @base.int(@base.tag_arg(j, 2)),
      )
    "Sub" =>
      Sub(
        @base.boolean(@base.tag_arg(j, 0)),
        @base.int(@base.tag_arg(j, 1)),
        @base.int(@base.tag_arg(j, 2)),
      )
    "Addi" =>
      Addi(
        @base.boolean(@base.tag_arg(j, 0)),
        @base.int(@base.tag_arg(j, 1)),
        value(@base.tag_arg(j, 2)),
      )
    "Subi" =>
      Subi(
        @base.boolean(@base.tag_arg(j, 0)),
        @base.int(@base.tag_arg(j, 1)),
        value(@base.tag_arg(j, 2)),
      )
    "Eq" => Eq(@base.int(@base.tag_arg(j, 0)), @base.int(@base.tag_arg(j, 1)))
    "Lt" => Lt(@base.int(@base.tag_arg(j, 0)), @base.int(@base.tag_arg(j, 1)))
    "Gt" => Gt(@base.int(@base.tag_arg(j, 0)), @base.int(@base.tag_arg(j, 1)))
    _ => @base.error("Invalid tiny ISA instruction")
  }
}
