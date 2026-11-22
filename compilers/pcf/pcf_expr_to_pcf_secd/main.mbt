///|
pub fn compile(e : @s.Expr) -> Array[@target.Instr] raise @utils.Fault {
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
      @utils.error("SECD compiler accepts the direct expression language")
  }
}

///|
pub fn compiler() -> @definition.Compiler[
  @s.Expr,
  @s.Inputs,
  Unit,
  @s.Output,
  @utils.Nat,
  @target.Code,
  @target.Ainput,
  @target.Rinput,
  @target.Routput,
  @target.Foutput,
] {
  @definition.Compiler(
    compile_code=code => { instructions: compile(code), },
    encode_ainput=input => input,
    encode_rinput=input => input,
    decode_routput=output => output,
    decode_foutput=output => output,
  )
}
