///|
fn builtin_json(v : Builtin) -> Json {
  match v {
    This => @utils.tag("This", [])
    CodeLen => @utils.tag("Code_len", [])
    DataLen => @utils.tag("Data_len", [])
  }
}

///|
fn decode_builtin(j : Json) -> Builtin raise @utils.Fault {
  match @utils.tag_name(j) {
    "This" => This
    "Code_len" => CodeLen
    "Data_len" => DataLen
    _ => @utils.error("Invalid symbolic assembly snapshot")
  }
}

///|
fn rawvalue_json(v : RawValue) -> Json {
  match v {
    Imm(v0) => @utils.tag("Imm", [@utils.jstr(v0.to_string())])
    Label(v0) => @utils.tag("Label", [@utils.jstr(v0)])
    Const(v0) => @utils.tag("Const", [@utils.jstr(v0)])
    Builtin(v0) => @utils.tag("Builtin", [builtin_json(v0)])
  }
}

///|
fn decode_rawvalue(j : Json) -> RawValue raise @utils.Fault {
  match @utils.tag_name(j) {
    "Imm" => Imm(@utils.json_nat(@utils.tag_arg(j, 0)))
    "Label" => Label(@utils.identifier(@utils.string(@utils.tag_arg(j, 0))))
    "Const" => Const(@utils.identifier(@utils.string(@utils.tag_arg(j, 0))))
    "Builtin" => Builtin(decode_builtin(@utils.tag_arg(j, 0)))
    _ => @utils.error("Invalid symbolic assembly snapshot")
  }
}

///|
fn operand_json(v : Operand) -> Json {
  match v {
    Reg(v0) => @utils.tag("Reg", [@utils.jint(v0)])
    Value(v0) => @utils.tag("Value", [rawvalue_json(v0)])
  }
}

///|
fn decode_operand(j : Json) -> Operand raise @utils.Fault {
  match @utils.tag_name(j) {
    "Reg" => Reg(@utils.int(@utils.tag_arg(j, 0)))
    "Value" => Value(decode_rawvalue(@utils.tag_arg(j, 0)))
    _ => @utils.error("Invalid symbolic assembly snapshot")
  }
}

///|
fn directive_json(v : Directive) -> Json {
  match v {
    Text => @utils.tag("Text", [])
    Data => @utils.tag("Data", [])
    Equ(v0, v1) =>
      @utils.tag("Equ", [@utils.jstr(v0), @utils.jstr(v1.to_string())])
  }
}

///|
fn decode_directive(j : Json) -> Directive raise @utils.Fault {
  match @utils.tag_name(j) {
    "Text" => Text
    "Data" => Data
    "Equ" =>
      Equ(
        @utils.identifier(@utils.string(@utils.tag_arg(j, 0))),
        @utils.json_nat(@utils.tag_arg(j, 1)),
      )
    _ => @utils.error("Invalid symbolic assembly snapshot")
  }
}

///|
fn item_json(v : Item) -> Json {
  match v {
    Directive(v0) => @utils.tag("Directive", [directive_json(v0)])
    LabelDef(v0) => @utils.tag("LabelDef", [@utils.jstr(v0)])
    Inst(v0) => @utils.tag("Inst", [rawinst_json(v0)])
    DataValue(v0) => @utils.tag("DataValue", [rawvalue_json(v0)])
  }
}

///|
fn decode_item(j : Json) -> Item raise @utils.Fault {
  match @utils.tag_name(j) {
    "Directive" => Directive(decode_directive(@utils.tag_arg(j, 0)))
    "LabelDef" =>
      LabelDef(@utils.identifier(@utils.string(@utils.tag_arg(j, 0))))
    "Inst" => Inst(decode_rawinst(@utils.tag_arg(j, 0)))
    "DataValue" => DataValue(decode_rawvalue(@utils.tag_arg(j, 0)))
    _ => @utils.error("Invalid symbolic assembly snapshot")
  }
}

///|
fn rawinst_json(i : RawInst) -> Json {
  @utils.object([
    ("mnemonic", @utils.jstr(i.mnemonic)),
    ("cond_flag", @utils.jbool(i.cond_flag)),
    ("operands", @utils.jarray(i.operands.map(operand_json))),
  ])
}

///|
fn decode_rawinst(j : Json) -> RawInst raise @utils.Fault {
  {
    mnemonic: @utils.string(@utils.field(j, "mnemonic")),
    cond_flag: @utils.boolean(@utils.field(j, "cond_flag")),
    operands: @utils.array(@utils.field(j, "operands")).map(decode_operand),
  }
}

///|
fn labels_json(labels : Array[(String, Int32)]) -> Json {
  @utils.env_json(labels, @utils.jint)
}

///|
fn symbols_json(s : Symbols) -> Json {
  @utils.object([
    ("consts", @utils.env_json(s.consts, n => @utils.jstr(n.to_string()))),
    ("text_labels", labels_json(s.text_labels)),
    ("data_labels", labels_json(s.data_labels)),
    ("code_len", @utils.jint(s.code_len)),
    ("data_len", @utils.jint(s.data_len)),
  ])
}

///|
fn decode_symbols(j : Json) -> Symbols raise @utils.Fault {
  {
    consts: @utils.decode_env(@utils.field(j, "consts"), @utils.json_nat),
    text_labels: @utils.decode_env(@utils.field(j, "text_labels"), @utils.int),
    data_labels: @utils.decode_env(@utils.field(j, "data_labels"), @utils.int),
    code_len: @utils.int(@utils.field(j, "code_len")),
    data_len: @utils.int(@utils.field(j, "data_len")),
  }
}

///|
fn debug_json(d : DebugInfo) -> Json {
  fn labels(xs : Array[(Int32, Array[String])]) -> Json {
    @utils.jarray(
      xs.map(p => {
        @utils.jarray([@utils.jint(p.0), @utils.jarray(p.1.map(@utils.jstr))])
      }),
    )
  }
  @utils.object([
    ("text_labels_by_addr", labels(d.text_labels_by_addr)),
    ("data_labels_by_addr", labels(d.data_labels_by_addr)),
  ])
}

///|
fn decode_debug(j : Json) -> DebugInfo raise @utils.Fault {
  fn labels(j : Json) -> Array[(Int32, Array[String])] raise @utils.Fault {
    @utils.array(j).map(j => {
      let a = @utils.array(j)
      @utils.require(a.length() == 2, "Invalid debug labels")
      (
        @utils.int(a[0]),
        @utils.array(a[1]).map(j => @utils.identifier(@utils.string(j))),
      )
    })
  }
  {
    text_labels_by_addr: labels(@utils.field(j, "text_labels_by_addr")),
    data_labels_by_addr: labels(@utils.field(j, "data_labels_by_addr")),
  }
}
