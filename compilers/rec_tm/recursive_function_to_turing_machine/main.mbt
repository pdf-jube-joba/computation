///|
pub fn compiler() -> @runtime.Compiler {
  let first = @first.compiler()
  {
    ..first,
    compile_code: text => {
      @target.print_code(
        @third.compile(
          @second.compile(@first.compile(@source.parse_code(text))),
        ),
      )
    },
  }
}
