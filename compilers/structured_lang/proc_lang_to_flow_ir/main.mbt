///|
priv enum VarLocation {
  Global
  Param(Int32)
  Local(Int32)
}

///|
priv struct Scope {
  locals_len : Int32
  params_len : Int32
  vars : Array[(String, VarLocation)]
  entries : Array[(String, String)]
}

///|
fn index_from_top(
  c : @lower.Builder,
  offset : Int32,
  dst : String,
) -> Array[@ir.Stmt] {
  let len = c.fresh_vreg("len")
  [
    @ir.LGet(len),
    @ir.BinOp(
      dst,
      @ir.VReg(len),
      @ir.Sub,
      @ir.Imm(@utils.nint(offset.reinterpret_as_uint())),
    ),
  ]
}

///|
fn variable_place(
  c : @lower.Builder,
  scope : Scope,
  name : String,
) -> (Array[@ir.Stmt], @ir.PlaceExpr) raise @utils.Fault {
  match @utils.env_lookup(scope.vars, name) {
    Global => ([], @ir.Label(name))
    Local(i) => {
      let index = c.fresh_vreg("idx_local")
      (index_from_top(c, i + 1, index), @ir.SAcc(@ir.VReg(index)))
    }
    Param(i) => {
      let index = c.fresh_vreg("idx_param")
      (
        index_from_top(c, scope.locals_len + 2 + i, index),
        @ir.SAcc(@ir.VReg(index)),
      )
    }
  }
}

///|
fn load_var(
  c : @lower.Builder,
  scope : Scope,
  name : String,
  dst : String,
) -> Array[@ir.Stmt] raise @utils.Fault {
  let (stmts, place) = variable_place(c, scope, name)
  stmts + [@ir.Load(dst, place)]
}

///|
fn store_var(
  c : @lower.Builder,
  scope : Scope,
  name : String,
  src : @ir.ValueExpr,
) -> Array[@ir.Stmt] raise @utils.Fault {
  let (stmts, place) = variable_place(c, scope, name)
  stmts + [@ir.Store(place, src)]
}

///|
fn atom(
  c : @lower.Builder,
  scope : Scope,
  dst : String,
  a : @a.Atom,
) -> Array[@ir.Stmt] raise @utils.Fault {
  match a {
    @a.Imm(n) => [@ir.Assign(dst, @ir.Imm(n))]
    @a.Var(v) => load_var(c, scope, v, dst)
  }
}

///|
fn arithmetic(
  c : @lower.Builder,
  scope : Scope,
  dst : String,
  a : @a.Aexp,
) -> Array[@ir.Stmt] raise @utils.Fault {
  match a {
    @a.Atom(a) => atom(c, scope, dst, a)
    @a.Bin(a, add, b) => {
      let l = c.fresh_vreg("lhs")
      let r = c.fresh_vreg("rhs")
      atom(c, scope, l, a) +
      atom(c, scope, r, b) +
      [@ir.BinOp(dst, @ir.VReg(l), @lower.op(add), @ir.VReg(r))]
    }
  }
}

///|
fn statement(
  c : @lower.Builder,
  stmt : @s.Stmt,
  cont : String,
  scope : Scope,
) -> String raise @utils.Fault {
  match stmt {
    @s.Nop => {
      let label = c.fresh_region("nop")
      c.single(label, [@ir.Nop], @lower.enter(cont))
      label
    }
    @s.Assign(v, a) => {
      let label = c.fresh_region("assign")
      let r = c.fresh_vreg("assign")
      let ss = arithmetic(c, scope, r, a)
      c.single(
        label,
        ss + store_var(c, scope, v, @ir.VReg(r)),
        @lower.enter(cont),
      )
      label
    }
    @s.Seq(a, b) => {
      let right = statement(c, b, cont, scope)
      statement(c, a, right, scope)
    }
    @s.If(cond, body) => {
      let yes = statement(c, body, cont, scope)
      let label = c.fresh_region("if")
      let l = c.fresh_vreg("if_lhs")
      let r = c.fresh_vreg("if_rhs")
      @lower.branch_region(
        c,
        label,
        atom(c, scope, l, cond.left) + atom(c, scope, r, cond.right),
        @lower.relation(cond.op, @ir.VReg(l), @ir.VReg(r)),
        yes,
        cont,
      )
      label
    }
    @s.While(cond, body) => {
      let head = c.fresh_region("while_test")
      let yes = statement(c, body, head, scope)
      let l = c.fresh_vreg("while_lhs")
      let r = c.fresh_vreg("while_rhs")
      @lower.branch_region(
        c,
        head,
        atom(c, scope, l, cond.left) + atom(c, scope, r, cond.right),
        @lower.relation(cond.op, @ir.VReg(l), @ir.VReg(r)),
        yes,
        cont,
      )
      head
    }
    @s.Return(vars) => {
      let label = c.fresh_region("ret")
      let regs = []
      let ss = []
      for v in vars {
        let r = c.fresh_vreg("retv")
        ss.append(load_var(c, scope, v, r))
        regs.push(r)
      }
      let address = c.fresh_vreg("ret_addr")
      for _ in 0..<scope.locals_len {
        ss.push(@ir.Pop(c.fresh_vreg("drop_local")))
      }
      ss.push(@ir.Pop(address))
      for _ in 0..<scope.params_len {
        ss.push(@ir.Pop(c.fresh_vreg("drop_param")))
      }
      for r in regs.rev() {
        ss.push(@ir.Push(@ir.VReg(r)))
      }
      c.single(label, ss, @ir.Enter([], @ir.VReg(address)))
      label
    }
    @s.Call(name, args, rets) => {
      let ret_region = c.fresh_region("call_ret")
      let arg_regs = []
      let ss = []
      for arg in args.rev() {
        let r = c.fresh_vreg("arg")
        ss.append(load_var(c, scope, arg, r))
        arg_regs.push(r)
      }
      let target = @utils.env_lookup(scope.entries, name)
      let call_region = c.fresh_region("call")
      for r in arg_regs {
        ss.push(@ir.Push(@ir.VReg(r)))
      }
      ss.push(@ir.Push(@ir.CodeLabel(ret_region)))
      c.single(call_region, ss, @lower.enter(target))
      let ret_regs = rets.map(v => (v, c.fresh_vreg("ret")))
      let ss = ret_regs.map(entry => @ir.Pop(entry.1))
      for entry in ret_regs {
        ss.append(store_var(c, scope, entry.0, @ir.VReg(entry.1)))
      }
      c.single(ret_region, ss, @lower.enter(cont))
      call_region
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
      let halt = c.fresh_region("halt")
      c.single(halt, [@ir.Nop], @ir.Halt([]))
      let entries = source.procs.map(p => {
        (p.name, c.fresh_region("proc_" + p.name))
      })
      for p in source.procs {
        let mut vars = source.statics.map(v => (v, Global))
        for i, v in p.params {
          vars = @utils.env_set(vars, v, Param(i))
        }
        for i, v in p.locals {
          vars = @utils.env_set(vars, v, Local(i))
        }
        let scope = {
          vars,
          locals_len: p.locals.length(),
          params_len: p.params.length(),
          entries,
        }
        let body = statement(c, @s.with_return(p.body), halt, scope)
        c.single(
          @utils.env_lookup(entries, p.name),
          p.locals.map(_ => @ir.Push(@ir.Imm(@utils.zero()))),
          @lower.enter(body),
        )
      }
      let main = @utils.env_lookup(entries, "main")
      c.single(
        "main",
        [@ir.Push(@ir.CodeLabel(halt)), @ir.Nop],
        @lower.enter(main),
      )
      c.finish(statics=Some(source.statics))
    },
    encode_ainput=input => @lower.encode_env(input.values),
    encode_rinput=_ => "",
    decode_routput=output => output,
    decode_foutput=output => { values: @lower.decode_env(output), },
  )
}
