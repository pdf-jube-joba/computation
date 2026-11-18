///|
fn builtin_json(v : Builtin) -> Json {
  match v {
    This => @base.tag("This", [])
    CodeLen => @base.tag("Code_len", [])
    DataLen => @base.tag("Data_len", [])
  }
}

///|
fn decode_builtin(j : Json) -> Builtin raise @base.Fault {
  match @base.tag_name(j) {
    "This" => This
    "Code_len" => CodeLen
    "Data_len" => DataLen
    _ => @base.error("Invalid symbolic assembly snapshot")
  }
}

///|
fn rawvalue_json(v : RawValue) -> Json {
  match v {
    Imm(v0) => @base.tag("Imm", [@base.jstr(v0.to_string())])
    Label(v0) => @base.tag("Label", [@base.jstr(v0)])
    Const(v0) => @base.tag("Const", [@base.jstr(v0)])
    Builtin(v0) => @base.tag("Builtin", [builtin_json(v0)])
  }
}

///|
fn decode_rawvalue(j : Json) -> RawValue raise @base.Fault {
  match @base.tag_name(j) {
    "Imm" => Imm(@base.json_nat(@base.tag_arg(j, 0)))
    "Label" => Label(@base.identifier(@base.string(@base.tag_arg(j, 0))))
    "Const" => Const(@base.identifier(@base.string(@base.tag_arg(j, 0))))
    "Builtin" => Builtin(decode_builtin(@base.tag_arg(j, 0)))
    _ => @base.error("Invalid symbolic assembly snapshot")
  }
}

///|
fn operand_json(v : Operand) -> Json {
  match v {
    Reg(v0) => @base.tag("Reg", [@base.jint(v0)])
    Value(v0) => @base.tag("Value", [rawvalue_json(v0)])
  }
}

///|
fn decode_operand(j : Json) -> Operand raise @base.Fault {
  match @base.tag_name(j) {
    "Reg" => Reg(@base.int(@base.tag_arg(j, 0)))
    "Value" => Value(decode_rawvalue(@base.tag_arg(j, 0)))
    _ => @base.error("Invalid symbolic assembly snapshot")
  }
}

///|
fn directive_json(v : Directive) -> Json {
  match v {
    Text => @base.tag("Text", [])
    Data => @base.tag("Data", [])
    Equ(v0, v1) =>
      @base.tag("Equ", [@base.jstr(v0), @base.jstr(v1.to_string())])
  }
}

///|
fn decode_directive(j : Json) -> Directive raise @base.Fault {
  match @base.tag_name(j) {
    "Text" => Text
    "Data" => Data
    "Equ" =>
      Equ(
        @base.identifier(@base.string(@base.tag_arg(j, 0))),
        @base.json_nat(@base.tag_arg(j, 1)),
      )
    _ => @base.error("Invalid symbolic assembly snapshot")
  }
}

///|
fn item_json(v : Item) -> Json {
  match v {
    Directive(v0) => @base.tag("Directive", [directive_json(v0)])
    LabelDef(v0) => @base.tag("LabelDef", [@base.jstr(v0)])
    Inst(v0) => @base.tag("Inst", [rawinst_json(v0)])
    DataValue(v0) => @base.tag("DataValue", [rawvalue_json(v0)])
  }
}

///|
fn decode_item(j : Json) -> Item raise @base.Fault {
  match @base.tag_name(j) {
    "Directive" => Directive(decode_directive(@base.tag_arg(j, 0)))
    "LabelDef" => LabelDef(@base.identifier(@base.string(@base.tag_arg(j, 0))))
    "Inst" => Inst(decode_rawinst(@base.tag_arg(j, 0)))
    "DataValue" => DataValue(decode_rawvalue(@base.tag_arg(j, 0)))
    _ => @base.error("Invalid symbolic assembly snapshot")
  }
}

///|
fn rawinst_json(i : RawInst) -> Json {
  @base.object([
    ("mnemonic", @base.jstr(i.mnemonic)),
    ("cond_flag", @base.jbool(i.cond_flag)),
    ("operands", @base.jarray(i.operands.map(operand_json))),
  ])
}

///|
fn decode_rawinst(j : Json) -> RawInst raise @base.Fault {
  {
    mnemonic: @base.string(@base.field(j, "mnemonic")),
    cond_flag: @base.boolean(@base.field(j, "cond_flag")),
    operands: @base.array(@base.field(j, "operands")).map(decode_operand),
  }
}

///|
fn labels_json(labels : Array[(String, Int32)]) -> Json {
  @data.env_json(labels, @base.jint)
}

///|
fn symbols_json(s : Symbols) -> Json {
  @base.object([
    ("consts", @data.env_json(s.consts, n => @base.jstr(n.to_string()))),
    ("text_labels", labels_json(s.text_labels)),
    ("data_labels", labels_json(s.data_labels)),
    ("code_len", @base.jint(s.code_len)),
    ("data_len", @base.jint(s.data_len)),
  ])
}

///|
fn decode_symbols(j : Json) -> Symbols raise @base.Fault {
  {
    consts: @data.decode_env(@base.field(j, "consts"), @base.json_nat),
    text_labels: @data.decode_env(@base.field(j, "text_labels"), @base.int),
    data_labels: @data.decode_env(@base.field(j, "data_labels"), @base.int),
    code_len: @base.int(@base.field(j, "code_len")),
    data_len: @base.int(@base.field(j, "data_len")),
  }
}

///|
fn debug_json(d : DebugInfo) -> Json {
  fn labels(xs : Array[(Int32, Array[String])]) -> Json {
    @base.jarray(
      xs.map(p => {
        @base.jarray([@base.jint(p.0), @base.jarray(p.1.map(@base.jstr))])
      }),
    )
  }
  @base.object([
    ("text_labels_by_addr", labels(d.text_labels_by_addr)),
    ("data_labels_by_addr", labels(d.data_labels_by_addr)),
  ])
}

///|
fn decode_debug(j : Json) -> DebugInfo raise @base.Fault {
  fn labels(j : Json) -> Array[(Int32, Array[String])] raise @base.Fault {
    @base.array(j).map(j => {
      let a = @base.array(j)
      @base.require(a.length() == 2, "Invalid debug labels")
      (
        @base.int(a[0]),
        @base.array(a[1]).map(j => @base.identifier(@base.string(j))),
      )
    })
  }
  {
    text_labels_by_addr: labels(@base.field(j, "text_labels_by_addr")),
    data_labels_by_addr: labels(@base.field(j, "data_labels_by_addr")),
  }
}
