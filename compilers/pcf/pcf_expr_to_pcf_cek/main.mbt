///|
pub fn compiler() -> @runtime.Compiler {
  {
    compile_code: text => @s.print(@s.parse(text)),
    encode_ainput: text => {
      "(" + @base.join(@s.inputs(text).map(n => n.to_string()), ", ") + ")"
    },
    encode_rinput: text => {
      @base.unit_input(text)
      ""
    },
    decode_routput: text => {
      if text.trim().is_empty() {
        ""
      } else {
        @base.nat(text).to_string()
      }
    },
    decode_foutput: text => @base.nat(text).to_string(),
  }
}
