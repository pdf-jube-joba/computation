///|
pub fn compiler() -> @runtime.Compiler {
  {
    compile_code: text => {
      let count = if text.trim().is_empty() {
        0
      } else {
        @base.index(@base.nat(text))
      }
      count.to_string()
    },
    encode_ainput: text => {
      @base.unit_input(text)
      ""
    },
    encode_rinput: text => {
      let p = @p.create(text)
      let op = p.keyword()
      p.finish()
      @base.require(op == "inc" || op == "dec", "Expected \\inc or \\dec")
      "\\" + op
    },
    decode_routput: text => text,
    decode_foutput: text => text,
  }
}
