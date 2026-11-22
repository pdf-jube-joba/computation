///|
priv struct Lowering {
  builder : Builder
  resolution : @f.Resolution
  statics : Array[(@c.ValueExpr, Int32)]
}

///|
fn canonical(
  ctx : Lowering,
  target : @f.Target,
) -> @c.ValueExpr raise @utils.Fault {
  name(@f.target_name(ctx.resolution, target))
}

///|
fn value(
  ctx : Lowering,
  region : Int32,
  v : @f.ValueExpr,
) -> Triple raise @utils.Fault {
  let c = ctx.builder
  match v {
    @f.Imm(n) => (imm(0), @c.Imm(n), imm(0))
    @f.VReg(r) => {
      let (t, a, b) = reg_value(r)
      // Copy every operand before a later destination can overwrite it.
      let t = bind(c, @c.Value(t))
      let a = bind(c, @c.Value(a))
      let b = bind(c, @c.Value(b))
      (t, a, b)
    }
    @f.CodeLabel(label) =>
      (
        imm(1),
        canonical(ctx, @f.resolve_target(ctx.resolution, region, label)),
        imm(0),
      )
    @f.Ref(p) => place(ctx, region, p)
  }
}

///|
fn place(
  ctx : Lowering,
  region : Int32,
  p : @f.PlaceExpr,
) -> Triple raise @utils.Fault {
  let c = ctx.builder
  match p {
    @f.Label(label) => {
      ignore(@f.id_of_name(ctx.resolution.data, label))
      (imm(2), name(label), imm(0))
    }
    @f.Deref(r) => {
      let v = value(ctx, region, @f.VReg(r))
      require(c, (v.0, @c.Gt, imm(1)))
      require(c, (v.0, @c.Lt, imm(5)))
      v
    }
    @f.SAcc(v) => (imm(3), index(c, value(ctx, region, v)), imm(0))
    @f.HAcc(h, i) => {
      let h = index(c, value(ctx, region, h))
      require(c, (h, @c.Lt, imm(2147483647)))
      let i = index(c, value(ctx, region, i))
      (imm(4), h, i)
    }
  }
}

///|
fn heap_info(c : Builder, handle : @c.ValueExpr) -> Triple {
  require(c, (handle, @c.Lt, imm(2147483647)))
  let slot = times3(c, handle)
  let (active, base, len) = read3(c, 4, slot)
  require(c, (active, @c.Eq, imm(1)))
  (base, len, slot)
}

///|
fn static_addr(
  ctx : Lowering,
  key : @c.ValueExpr,
) -> @c.ValueExpr raise @utils.Fault {
  let c = ctx.builder
  let out = fresh(c, "static_offset")
  choose(c, key, ctx.statics, i => set(c, out, imm(3 * i)))
  @c.Reg(out)
}

///|
fn access(
  ctx : Lowering,
  key : Triple,
  action : (Int32, @c.ValueExpr) -> Unit raise @utils.Fault,
) -> Unit raise @utils.Fault {
  let c = ctx.builder
  let (t, a, b) = key
  choose(c, t, [(imm(2), 2), (imm(3), 3), (imm(4), 4)], kind => {
    match kind {
      2 => action(3, static_addr(ctx, a))
      3 => {
        require(c, (a, @c.Lt, @c.Reg("stack_size")))
        action(1, times3(c, a))
      }
      _ => {
        let (base, len, _) = heap_info(c, a)
        require(c, (b, @c.Lt, len))
        let i = binary(c, base, @c.Add, times3(c, b))
        action(2, i)
      }
    }
  })
}

///|
fn load_key(ctx : Lowering, key : Triple) -> Triple raise @utils.Fault {
  let c = ctx.builder
  let t = fresh(c, "loaded_tag")
  let a = fresh(c, "loaded_a")
  let b = fresh(c, "loaded_b")
  access(ctx, key, (memory, index) => {
    let v = read3(c, memory, index)
    set(c, t, v.0)
    set(c, a, v.1)
    set(c, b, v.2)
  })
  (@c.Reg(t), @c.Reg(a), @c.Reg(b))
}

///|
fn jump(
  ctx : Lowering,
  region : Int32,
  enter : Bool,
  v : @f.ValueExpr,
) -> Unit raise @utils.Fault {
  let c = ctx.builder
  let (t, a, _) = match v {
    @f.CodeLabel(label) => {
      let target = if enter {
        @f.region_target(ctx.resolution, label)
      } else {
        @f.local_target(ctx.resolution, region, label)
      }
      (imm(1), canonical(ctx, target), imm(0))
    }
    _ => value(ctx, region, v)
  }
  require(c, (t, @c.Eq, imm(1)))
  let targets = if enter {
    ctx.resolution.regions.mapi((i, entry) => {
      (canonical(ctx, @f.RegionTarget(entry.1)), region_label(i))
    })
  } else {
    ctx.resolution.blocks[region].mapi((i, entry) => {
      (canonical(ctx, @f.BlockTarget(entry.1)), block_label(region, i))
    })
  }
  finish(
    c,
    targets.map(entry => {
      cond: (a, @c.Eq, entry.0),
      target: @c.Label(entry.1),
    }),
    @c.Goto(@c.Label("runtime_error")),
  )
}

///|
fn condition(
  ctx : Lowering,
  region : Int32,
  cond : @f.Cond,
  yes : String,
  no : String,
) -> Unit raise @utils.Fault {
  let c = ctx.builder
  let (a, op, b) = match cond {
    @f.Eq(a, b) => (a, @c.Eq, b)
    @f.Lt(a, b) => (a, @c.Lt, b)
    @f.Gt(a, b) => (a, @c.Gt, b)
  }
  let left = value(ctx, region, a)
  let right = value(ctx, region, b)
  if op == @c.Eq {
    let fields = fresh(c, "equal_fields")
    branch(c, (left.0, @c.Eq, right.0), fields, no)
    start(c, fields)
    let last = fresh(c, "equal_last")
    branch(c, (left.1, @c.Eq, right.1), last, no)
    start(c, last)
    branch(c, (left.2, @c.Eq, right.2), yes, no)
  } else {
    let a = numeric(c, left)
    let b = numeric(c, right)
    branch(c, (a, op, b), yes, no)
  }
}

///|
fn statement(
  ctx : Lowering,
  region : Int32,
  stmt : @f.Stmt,
) -> Unit raise @utils.Fault {
  let c = ctx.builder
  match stmt {
    @f.Nop => ()
    @f.Assign(r, v) => {
      let v = value(ctx, region, v)
      copy(c, r, v)
    }
    @f.Load(r, p) => {
      let key = place(ctx, region, p)
      let v = load_key(ctx, key)
      copy(c, r, v)
    }
    @f.Store(p, v) => {
      let key = place(ctx, region, p)
      let v = value(ctx, region, v)
      access(ctx, key, (memory, index) => write3(c, memory, index, v))
    }
    @f.BinOp(r, a, op, b) => {
      let a = numeric(c, value(ctx, region, a))
      let b = numeric(c, value(ctx, region, b))
      let n = binary(
        c,
        a,
        match op {
          @f.Add => @c.Add
          @f.Sub => @c.Sub
        },
        b,
      )
      copy(c, r, (imm(0), n, imm(0)))
    }
    @f.Push(v) => {
      let v = value(ctx, region, v)
      let i = times3(c, @c.Reg("stack_size"))
      write3(c, 1, i, v)
      emit(
        c,
        @c.Assign("stack_size", @c.Binop(@c.Reg("stack_size"), @c.Add, imm(1))),
      )
    }
    @f.Pop(r) => {
      require(c, (@c.Reg("stack_size"), @c.Gt, imm(0)))
      emit(
        c,
        @c.Assign("stack_size", @c.Binop(@c.Reg("stack_size"), @c.Sub, imm(1))),
      )
      let i = times3(c, @c.Reg("stack_size"))
      let v = read3(c, 1, i)
      copy(c, r, v)
    }
    @f.LGet(r) => copy(c, r, (imm(0), @c.Reg("stack_size"), imm(0)))
    @f.HAlloc(v, r) => {
      let len = index(c, value(ctx, region, v))
      require(c, (@c.Reg("next_handle"), @c.Lt, imm(2147483647)))
      let h = bind(c, @c.Value(@c.Reg("next_handle")))
      let slot = times3(c, h)
      write3(c, 4, slot, (imm(1), @c.Reg("heap_end"), len))
      let size = times3(c, len)
      emit(c, @c.Assign("heap_end", @c.Binop(@c.Reg("heap_end"), @c.Add, size)))
      emit(
        c,
        @c.Assign(
          "next_handle",
          @c.Binop(@c.Reg("next_handle"), @c.Add, imm(1)),
        ),
      )
      copy(c, r, (imm(0), h, imm(0)))
    }
    @f.HFree(v) => {
      let h = index(c, value(ctx, region, v))
      let (_, _, slot) = heap_info(c, h)
      store(c, 4, slot, imm(0))
    }
    @f.Print(r) => {
      let n = numeric(c, value(ctx, region, @f.VReg(r)))
      emit(c, @c.Print(n))
    }
    @f.Input(p) => {
      let key = place(ctx, region, p)
      access(ctx, key, (memory, i) => {
        let p = addr(c, memory, offset(c, i, 1))
        emit(c, @c.Input(@c.Deref(p)))
        store(c, memory, i, imm(0))
        store(c, memory, offset(c, i, 2), imm(0))
      })
    }
  }
}
