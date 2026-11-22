///|
pub struct Builder {
  regions : Array[@ir.Region]
  region_names : @utils.Supply
  block_names : @utils.Supply
  registers : @utils.Supply
  variables : Map[String, Bool]
}

///|
pub fn create() -> Builder {
  {
    regions: [],
    region_names: @utils.supply(reserved=["main"]),
    block_names: @utils.supply(),
    registers: @utils.supply(),
    variables: Map([]),
  }
}

///|
pub fn Builder::fresh_region(self : Builder, hint : String) -> String {
  self.region_names.fresh(hint)
}

///|
pub fn Builder::fresh_block(self : Builder, hint : String) -> String {
  self.block_names.fresh(hint)
}

///|
pub fn Builder::fresh_vreg(self : Builder, hint : String) -> String {
  self.registers.fresh(hint)
}

///|
pub fn Builder::variable(self : Builder, name : String) -> Unit {
  self.variables[name] = true
}

///|
pub fn enter(label : String) -> @ir.Cont {
  @ir.Enter([], @ir.CodeLabel(label))
}

///|
pub fn go(label : String) -> @ir.Cont {
  @ir.Go([], @ir.CodeLabel(label))
}

///|
pub fn block(
  label : String,
  stmts : Array[@ir.Stmt],
  cont : @ir.Cont,
) -> @ir.Block {
  { label, stmts, cont, }
}

///|
pub fn Builder::region(
  self : Builder,
  label : String,
  blocks : Array[@ir.Block],
) -> Unit {
  self.regions.push({ label, blocks, })
}

///|
pub fn Builder::single(
  self : Builder,
  label : String,
  stmts : Array[@ir.Stmt],
  cont : @ir.Cont,
) -> Unit {
  self.region(label, [block(self.fresh_block("entry"), stmts, cont)])
}

///|
pub fn Builder::finish(
  self : Builder,
  prefix~ : Array[@ir.Region] = [],
  statics~ : Array[String]? = None,
) -> @ir.Code {
  let names = match statics {
    Some(names) => names
    None => {
      let names = self.variables.keys().collect()
      names.sort()
      names
    }
  }
  {
    statics: names.map(label => { label, value: @utils.zero(), }),
    regions: prefix + self.regions,
  }
}

///|
pub fn emit_atom(c : Builder, dst : String, atom : @a.Atom) -> Array[@ir.Stmt] {
  match atom {
    @a.Var(v) => {
      c.variable(v)
      [@ir.Load(dst, @ir.Label(v))]
    }
    @a.Imm(n) => [@ir.Assign(dst, @ir.Imm(n))]
  }
}

///|
pub fn op(add : Bool) -> @ir.Binop {
  if add {
    @ir.Add
  } else {
    @ir.Sub
  }
}

///|
pub fn relation(rel : @a.Rel, a : @ir.ValueExpr, b : @ir.ValueExpr) -> @ir.Cond {
  match rel {
    @a.Lt => @ir.Lt(a, b)
    @a.Eq => @ir.Eq(a, b)
    @a.Gt => @ir.Gt(a, b)
  }
}

///|
pub fn emit_aexp(c : Builder, dst : String, a : @a.Aexp) -> Array[@ir.Stmt] {
  match a {
    @a.Atom(a) => emit_atom(c, dst, a)
    @a.Bin(a, add, b) => {
      let l = c.fresh_vreg("lhs")
      let r = c.fresh_vreg("rhs")
      emit_atom(c, l, a) +
      emit_atom(c, r, b) +
      [@ir.BinOp(dst, @ir.VReg(l), op(add), @ir.VReg(r))]
    }
  }
}

///|
pub fn branch_region(
  c : Builder,
  label : String,
  stmts : Array[@ir.Stmt],
  condition : @ir.Cond,
  yes : String,
  no : String,
) -> Unit {
  let entry = c.fresh_block("entry")
  let y = c.fresh_block("true")
  let n = c.fresh_block("false")
  c.region(label, [
    block(
      entry,
      stmts,
      @ir.Go([{ cond: condition, target: @ir.CodeLabel(y), }], @ir.CodeLabel(n)),
    ),
    block(y, [], enter(yes)),
    block(n, [], enter(no)),
  ])
}

///|
pub fn encode_env(values : Array[(String, @utils.Nat)]) -> @ir.Environment {
  { values: @utils.env_sorted(values).map(pair => (pair.0, @ir.Num(pair.1))), }
}

///|
pub fn decode_env(
  output : @ir.Environment,
) -> Array[(String, @utils.Nat)] raise @utils.Fault {
  @utils.env_sorted(output.values).map(pair => {
    let number = match pair.1 {
      @ir.Num(n) => n
      _ => @utils.error("Expected a natural number in compiler output")
    }
    (pair.0, number)
  })
}
