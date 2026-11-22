///|
fn statement(
  c : @lower.Builder,
  stmt : @s.Stmt,
  cont : String,
  loops : Array[(String, (String, String))],
) -> String raise @utils.Fault {
  match stmt {
    @s.Nop => {
      let label = c.fresh_region("nop")
      c.single(label, [@ir.Nop], @lower.enter(cont))
      label
    }
    @s.Seq(a, b) => {
      let right = statement(c, b, cont, loops)
      statement(c, a, right, loops)
    }
    @s.Assign(v, a) => {
      c.variable(v)
      let label = c.fresh_region("assign")
      let dst = c.fresh_vreg("assign")
      c.single(
        label,
        @lower.emit_aexp(c, dst, a) + [@ir.Store(@ir.Label(v), @ir.VReg(dst))],
        @lower.enter(cont),
      )
      label
    }
    @s.If(cond, body) => {
      let yes = statement(c, body, cont, loops)
      let label = c.fresh_region("if")
      let l = c.fresh_vreg("if_lhs")
      let r = c.fresh_vreg("if_rhs")
      @lower.branch_region(
        c,
        label,
        @lower.emit_atom(c, l, cond.left) + @lower.emit_atom(c, r, cond.right),
        @lower.relation(cond.op, @ir.VReg(l), @ir.VReg(r)),
        yes,
        cont,
      )
      label
    }
    @s.Break(l, v) => {
      c.variable(v)
      let (_, exit) = @utils.env_lookup(loops, l)
      let label = c.fresh_region("break")
      let r = c.fresh_vreg("break")
      c.single(
        label,
        [@ir.Load(r, @ir.Label(v)), @ir.Push(@ir.VReg(r))],
        @lower.enter(exit),
      )
      label
    }
    @s.Continue(l) => {
      let (head, _) = @utils.env_lookup(loops, l)
      let label = c.fresh_region("continue")
      c.single(label, [@ir.Nop], @lower.enter(head))
      label
    }
    @s.Loop(l, b, out) => {
      c.variable(out)
      let head = c.fresh_region("loop_head")
      let exit = c.fresh_region("loop_exit")
      let body = statement(c, b, head, @utils.env_set(loops, l, (head, exit)))
      c.single(head, [@ir.Nop], @lower.enter(body))
      let r = c.fresh_vreg("loop_break")
      c.single(
        exit,
        [@ir.Pop(r), @ir.Store(@ir.Label(out), @ir.VReg(r))],
        @lower.enter(cont),
      )
      head
    }
  }
}

///|
pub fn compiler() -> @definition.Compiler[
  @s.Code,
  @s.Ainput,
  @s.Rinput,
  @s.Routput,
  @s.Foutput,
  @ir.Code,
  @ir.Ainput,
  @ir.Rinput,
  @ir.Routput,
  @ir.Foutput,
] {
  @definition.Compiler(
    compile_code=source => {
      let c = @lower.create()
      for v in source.statics {
        c.variable(v)
      }
      let halt = c.fresh_region("halt")
      c.region(halt, [
        @lower.block(c.fresh_block("halt_entry"), [@ir.Nop], @ir.Halt([])),
      ])
      let entry = statement(c, source.body, halt, [])
      c.finish(prefix=[
        {
          label: "main",
          blocks: [@lower.block("entry", [@ir.Nop], @lower.enter(entry))],
        },
      ])
    },
    encode_ainput=input => @lower.encode_env(input.values),
    encode_rinput=_ => "",
    decode_routput=_ => "",
    decode_foutput=output => { values: @lower.decode_env(output), },
  )
}
