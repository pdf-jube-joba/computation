///|
pub fn compile(e : @s.Expr) -> Array[@target.Instr] raise @base.Fault {
  match e {
    @s.Nat(n) => [@target.PushNat(n)]
    @s.Bool(b) => [@target.PushBool(b)]
    @s.Unit => [@target.PushUnit]
    @s.Print(n) => [@target.Print(n)]
    @s.Var(x) => [@target.Acc(x)]
    @s.BinOp(a, o, b) => {
      let c = compile(a)
      c.append(compile(b))
      c.push(@target.BinOp(o))
      c
    }
    @s.UnOp(o, e) => {
      let c = compile(e)
      c.push(@target.UnOp(o))
      c
    }
    @s.Fun(x, b) => {
      let c = compile(b)
      c.push(@target.Ret)
      [@target.MkCls(x, c)]
    }
    @s.Rec(f, x, b) => {
      let c = compile(b)
      c.push(@target.Ret)
      [@target.MkRec(f, x, c)]
    }
    @s.App(f, a) => {
      let c = compile(f)
      c.append(compile(a))
      c.push(@target.Ap)
      c
    }
    @s.If(c, t, e) => {
      let c = compile(c)
      c.push(@target.If(compile(t), compile(e)))
      c
    }
    @s.Block(_, _) =>
      @base.error("SECD compiler accepts the direct expression language")
  }
}

///|
fn input(text : String) -> String raise @base.Fault {
  "(" + @base.join(@s.inputs(text).map(n => n.to_string()), ", ") + ")"
}

///|
fn unit(text : String) -> String raise @base.Fault {
  @base.unit_input(text)
  ""
}

///|
fn effect(text : String) -> String raise @base.Fault {
  if text.trim().is_empty() {
    ""
  } else {
    @base.nat(text).to_string()
  }
}

///|
pub fn compiler() -> @runtime.Compiler {
  {
    compile_code: text => @target.print_code(compile(@s.parse(text))),
    encode_ainput: input,
    encode_rinput: unit,
    decode_routput: effect,
    decode_foutput: text => @base.nat(text).to_string(),
  }
}

///|
test "compiled recursion, closures and arbitrary precision" {
  let c = compiler()
  let m = @target.model()
  m.make(
    (c.compile_code)(
      "(\\rec f x => \\if \\not (x && \\false) \\then 4 \\else 2 \\fi)(\\true)",
    ),
    "()",
  )
  let mut out = None
  for _ in 0..<100 {
    let j = @base.parse_json(m.step(""))
    if @base.string(@base.field(j, "kind")) == "halt" {
      out = Some(@base.string(@base.field(j, "foutput")))
      break
    }
  }
  assert_eq(out, Some("4"))
  m.make((c.compile_code)("\\inc 999999999999999999999999999999"), "()")
  ignore(m.step(""))
  ignore(m.step(""))
  assert_eq(
    @base.string(@base.field(@base.parse_json(m.step("")), "foutput")),
    "1000000000000000000000000000000",
  )
}
