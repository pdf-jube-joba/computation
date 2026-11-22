///|
pub fn compiler() -> @definition.Compiler[
  @counter.Code,
  @counter.Ainput,
  @counter.Rinput,
  @counter.Routput,
  @counter.Foutput,
  @counter.Code,
  @counter.Ainput,
  @counter.Rinput,
  @counter.Routput,
  @counter.Foutput,
] {
  @definition.Compiler(
    compile_code=code => code,
    encode_ainput=input => input,
    encode_rinput=input => input,
    decode_routput=output => output,
    decode_foutput=output => output,
  )
}
