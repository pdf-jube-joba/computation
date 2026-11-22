///|
pub fn compiler() -> @definition.Compiler[
  @s.Expr,
  @s.Inputs,
  Unit,
  @s.Output,
  @utils.Nat,
  @s.Expr,
  @s.Inputs,
  Unit,
  @s.Output,
  @utils.Nat,
] {
  @definition.Compiler(
    compile_code=code => code,
    encode_ainput=input => input,
    encode_rinput=input => input,
    decode_routput=output => output,
    decode_foutput=output => output,
  )
}
