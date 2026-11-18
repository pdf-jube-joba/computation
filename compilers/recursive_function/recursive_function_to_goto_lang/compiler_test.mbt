///|
test "compiled primitive recursion and minimization match expected results" {
  let compiler = @recursive_function_to_goto_lang.compiler()
  for
    case in [
      ("\\ZERO", "()", "0"),
      (
        "\\SUCC", "(999999999999999999999999999999)", "1000000000000000000000000000000",
      ),
      ("\\PRIM[\\z:\\PROJ[1,0] \\s:\\COMP[\\SUCC: \\PROJ[3,0]]]", "(3,4)", "7"),
      ("\\MUOP[\\PROJ[1,0]]", "()", "0"),
    ] {
    let machine = @target.model()
    machine.make(
      (compiler.compile_code)(case.0),
      (compiler.encode_ainput)(case.1),
    )
    let mut output = None
    for _ in 0..<2000 {
      let result = @base.parse_json(machine.step(""))
      if @base.string(@base.field(result, "kind")) == "halt" {
        output = Some(
          (compiler.decode_foutput)(
            @base.string(@base.field(result, "foutput")),
          ),
        )
        break
      }
    }
    assert_eq(output, Some(case.2))
  }
}
