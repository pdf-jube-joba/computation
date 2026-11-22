///|
pub fn[T] instr_json(inst : Instr[T], value : (T) -> Json) -> Json {
  match inst {
    Nop => @utils.tag("Nop", [])
    Halt => @utils.tag("Halt", [])
    Rst => @utils.tag("Rst", [])
    Ld(v0, v1) => @utils.tag("Ld", [@utils.jint(v0), @utils.jint(v1)])
    St(v0, v1) => @utils.tag("St", [@utils.jint(v0), @utils.jint(v1)])
    Mov(v0, v1, v2) =>
      @utils.tag("Mov", [@utils.jbool(v0), @utils.jint(v1), @utils.jint(v2)])
    Ldi(v0, v1, v2) =>
      @utils.tag("Ldi", [@utils.jbool(v0), @utils.jint(v1), value(v2)])
    Add(v0, v1, v2) =>
      @utils.tag("Add", [@utils.jbool(v0), @utils.jint(v1), @utils.jint(v2)])
    Sub(v0, v1, v2) =>
      @utils.tag("Sub", [@utils.jbool(v0), @utils.jint(v1), @utils.jint(v2)])
    Addi(v0, v1, v2) =>
      @utils.tag("Addi", [@utils.jbool(v0), @utils.jint(v1), value(v2)])
    Subi(v0, v1, v2) =>
      @utils.tag("Subi", [@utils.jbool(v0), @utils.jint(v1), value(v2)])
    Eq(v0, v1) => @utils.tag("Eq", [@utils.jint(v0), @utils.jint(v1)])
    Lt(v0, v1) => @utils.tag("Lt", [@utils.jint(v0), @utils.jint(v1)])
    Gt(v0, v1) => @utils.tag("Gt", [@utils.jint(v0), @utils.jint(v1)])
  }
}

///|
pub fn[T] decode_instr(
  j : Json,
  value : (Json) -> T raise @utils.Fault,
) -> Instr[T] raise @utils.Fault {
  match @utils.tag_name(j) {
    "Nop" => Nop
    "Halt" => Halt
    "Rst" => Rst
    "Ld" =>
      Ld(@utils.int(@utils.tag_arg(j, 0)), @utils.int(@utils.tag_arg(j, 1)))
    "St" =>
      St(@utils.int(@utils.tag_arg(j, 0)), @utils.int(@utils.tag_arg(j, 1)))
    "Mov" =>
      Mov(
        @utils.boolean(@utils.tag_arg(j, 0)),
        @utils.int(@utils.tag_arg(j, 1)),
        @utils.int(@utils.tag_arg(j, 2)),
      )
    "Ldi" =>
      Ldi(
        @utils.boolean(@utils.tag_arg(j, 0)),
        @utils.int(@utils.tag_arg(j, 1)),
        value(@utils.tag_arg(j, 2)),
      )
    "Add" =>
      Add(
        @utils.boolean(@utils.tag_arg(j, 0)),
        @utils.int(@utils.tag_arg(j, 1)),
        @utils.int(@utils.tag_arg(j, 2)),
      )
    "Sub" =>
      Sub(
        @utils.boolean(@utils.tag_arg(j, 0)),
        @utils.int(@utils.tag_arg(j, 1)),
        @utils.int(@utils.tag_arg(j, 2)),
      )
    "Addi" =>
      Addi(
        @utils.boolean(@utils.tag_arg(j, 0)),
        @utils.int(@utils.tag_arg(j, 1)),
        value(@utils.tag_arg(j, 2)),
      )
    "Subi" =>
      Subi(
        @utils.boolean(@utils.tag_arg(j, 0)),
        @utils.int(@utils.tag_arg(j, 1)),
        value(@utils.tag_arg(j, 2)),
      )
    "Eq" =>
      Eq(@utils.int(@utils.tag_arg(j, 0)), @utils.int(@utils.tag_arg(j, 1)))
    "Lt" =>
      Lt(@utils.int(@utils.tag_arg(j, 0)), @utils.int(@utils.tag_arg(j, 1)))
    "Gt" =>
      Gt(@utils.int(@utils.tag_arg(j, 0)), @utils.int(@utils.tag_arg(j, 1)))
    _ => @utils.error("Invalid tiny ISA instruction")
  }
}
