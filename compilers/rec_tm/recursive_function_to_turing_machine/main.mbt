///|
pub fn compiler() -> @definition.Compiler[
  @source.Code,
  @source.Ainput,
  @source.Rinput,
  @source.Routput,
  @source.Foutput,
  @target.Code,
  @target.Ainput,
  @target.Rinput,
  @target.Routput,
  @target.Foutput,
] {
  @definition.compose(
    @definition.compose(@first.compiler(), @second.compiler()),
    @third.compiler(),
  )
}
