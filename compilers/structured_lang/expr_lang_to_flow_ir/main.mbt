///|
fn expression(
  c : @lower.Builder,
  blocks : Array[@ir.Block],
  exp : @s.Aexp,
  dst : String,
  end_block : String,
) -> String {
  match exp {
    @s.Num(n) => {
      let label = c.fresh_block("a_num")
      blocks.push(
        @lower.block(label, [@ir.Assign(dst, @ir.Imm(n))], @lower.go(end_block)),
      )
      label
    }
    @s.Var(v) => {
      c.variable(v)
      let label = c.fresh_block("a_var")
      blocks.push(
        @lower.block(label, [@ir.Load(dst, @ir.Label(v))], @lower.go(end_block)),
      )
      label
    }
    @s.Bin(a, add, b) => {
      let l = c.fresh_vreg("lhs")
      let r = c.fresh_vreg("rhs")
      let label = c.fresh_block("a_bin")
      blocks.push(
        @lower.block(
          label,
          [@ir.BinOp(dst, @ir.VReg(l), @lower.op(add), @ir.VReg(r))],
          @lower.go(end_block),
        ),
      )
      let right = expression(c, blocks, b, r, label)
      expression(c, blocks, a, l, right)
    }
    @s.Conditional(cond, a, b) => {
      let yes = expression(c, blocks, a, dst, end_block)
      let no = expression(c, blocks, b, dst, end_block)
      condition(c, blocks, cond, yes, no)
    }
  }
}

///|
fn condition(
  c : @lower.Builder,
  blocks : Array[@ir.Block],
  exp : @s.Bexp,
  yes : String,
  no : String,
) -> String {
  match exp {
    @s.Rel(a, rel, b) => {
      let l = c.fresh_vreg("c_lhs")
      let r = c.fresh_vreg("c_rhs")
      let label = c.fresh_block("cond")
      let cond = match rel {
        "<" => @ir.Lt(@ir.VReg(l), @ir.VReg(r))
        ">" => @ir.Gt(@ir.VReg(l), @ir.VReg(r))
        _ => @ir.Eq(@ir.VReg(l), @ir.VReg(r))
      }
      blocks.push(
        @lower.block(
          label,
          [],
          @ir.Go([{ cond, target: @ir.CodeLabel(yes), }], @ir.CodeLabel(no)),
        ),
      )
      let right = expression(c, blocks, b, r, label)
      expression(c, blocks, a, l, right)
    }
    @s.Or(a, b) => {
      let right = condition(c, blocks, b, yes, no)
      condition(c, blocks, a, yes, right)
    }
    @s.Not(b) => condition(c, blocks, b, no, yes)
  }
}

///|
fn entry_first(blocks : Array[@ir.Block], entry : String) -> Array[@ir.Block] {
  let mut index = 0
  for i, block in blocks {
    if block.label == entry {
      index = i
      break
    }
  }
  let result = blocks.copy()
  result[0] = blocks[index]
  result[index] = blocks[0]
  result
}

///|
fn condition_region(
  c : @lower.Builder,
  exp : @s.Bexp,
  yes : String,
  no : String,
) -> String {
  let label = c.fresh_region("stmt_cond")
  let y = c.fresh_block("true")
  let n = c.fresh_block("false")
  let blocks = [
    @lower.block(y, [@ir.Nop], @lower.enter(yes)),
    @lower.block(n, [@ir.Nop], @lower.enter(no)),
  ]
  let entry = condition(c, blocks, exp, y, n)
  c.region(label, entry_first(blocks, entry))
  label
}

///|
fn statement(c : @lower.Builder, stmt : @s.Stmt, cont : String) -> String {
  match stmt {
    @s.Nop => {
      let label = c.fresh_region("stmt_nop")
      c.single(label, [@ir.Nop], @lower.enter(cont))
      label
    }
    @s.Assign(v, a) => {
      c.variable(v)
      let label = c.fresh_region("stmt_assign")
      let dst = c.fresh_vreg("assign")
      let last = c.fresh_block("assign_end")
      let blocks = [
        @lower.block(
          last,
          [@ir.Store(@ir.Label(v), @ir.VReg(dst))],
          @lower.enter(cont),
        ),
      ]
      let entry = expression(c, blocks, a, dst, last)
      c.region(label, entry_first(blocks, entry))
      label
    }
    @s.Seq(a, b) => {
      let right = statement(c, b, cont)
      statement(c, a, right)
    }
    @s.If(cond, b) => {
      let yes = statement(c, b, cont)
      condition_region(c, cond, yes, cont)
    }
    @s.While(cond, b) => {
      let head = c.fresh_region("while_head")
      let body = statement(c, b, head)
      let test_region = condition_region(c, cond, body, cont)
      c.region(head, [
        @lower.block(
          c.fresh_block("head_entry"),
          [@ir.Nop],
          @lower.enter(test_region),
        ),
      ])
      head
    }
  }
}

///|
pub fn compiler() -> @runtime.Compiler {
  @lower.compiler(text => {
    let source = @s.parse_code(text)
    let c = @lower.create()
    let halt = c.fresh_region("halt")
    c.region(halt, [
      @lower.block(c.fresh_block("halt_entry"), [@ir.Nop], @ir.Halt([])),
    ])
    let entry = statement(c, source, halt)
    c.finish(prefix=[
      {
        label: "main",
        blocks: [@lower.block("entry", [@ir.Nop], @lower.enter(entry))],
      },
    ])
  })
}
