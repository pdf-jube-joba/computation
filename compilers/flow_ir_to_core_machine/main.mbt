///|
fn compile(source : @f.Code) -> @c.Code raise @base.Fault {
  @f.validate(source)
  let resolution = @f.build_resolution(source)
  let main_region = @f.id_of_name(resolution.regions, "main")
  let c : Builder = { serial: 0, label: "main", stmts: [], blocks: [], }
  let ctx = {
    builder: c,
    resolution,
    statics: source.statics.mapi((i, s) => (name(s.label), i)),
  }
  set(c, "stack_size", imm(0))
  set(c, "heap_end", imm(0))
  set(c, "next_handle", imm(1))
  for i, definition in source.statics {
    write3(c, 3, imm(3 * i), (imm(0), @c.Imm(definition.value), imm(0)))
  }
  let count = load(c, 0, imm(0))
  let cursor = fresh(c, "input_cursor")
  set(c, cursor, imm(0))
  let loop_label = fresh(c, "input_loop")
  let body = fresh(c, "input_body")
  let ready = fresh(c, "ready")
  go(c, loop_label)
  start(c, loop_label)
  branch(c, (@c.Reg(cursor), @c.Lt, count), body, ready)
  start(c, body)
  let twice = binary(c, @c.Reg(cursor), @c.Add, @c.Reg(cursor))
  let off = binary(c, twice, @c.Add, twice)
  let off = offset(c, off, 1)
  let key = load(c, 0, off)
  let (tag, a, b) = read3(c, 0, offset(c, off, 1))
  let payload = fresh(c, "input_payload")
  set(c, payload, a)
  let aliases = resolution.regions.map(entry => entry.0) +
    resolution.blocks[main_region].map(entry => entry.0) +
    resolution.label_names.map(entry => entry.1)
  let unique : Map[String, Bool] = Map([])
  for label in aliases {
    unique[label] = true
  }
  let aliases = unique.keys().collect()
  aliases.sort()
  choose(
    c,
    tag,
    [(imm(0), 0), (imm(1), 1), (imm(2), 2), (imm(3), 3), (imm(4), 4)],
    kind => {
      match kind {
        1 => {
          let cases = aliases.map(label => {
            (name(label), @f.resolve_target(resolution, main_region, label))
          })
          choose(c, a, cases, target => set(c, payload, canonical(ctx, target)))
        }
        2 => choose(c, a, ctx.statics, _ => ())
        _ => ()
      }
    },
  )
  let slot = static_addr(ctx, key)
  write3(c, 3, slot, (tag, @c.Reg(payload), b))
  emit(c, @c.Assign(cursor, @c.Binop(@c.Reg(cursor), @c.Add, imm(1))))
  go(c, loop_label)
  start(c, ready)
  go(c, region_label(main_region))
  for region_index, region in source.regions {
    start(c, region_label(region_index))
    for entry in resolution.registers {
      copy(c, entry.0, (imm(0), imm(0), imm(0)))
    }
    go(c, block_label(region_index, 0))
    for block_index, block in region.blocks {
      start(c, block_label(region_index, block_index))
      for stmt in block.stmts {
        statement(ctx, region_index, stmt)
      }
      for j in @f.ifs(block.cont) {
        let yes = fresh(c, "taken")
        let no = fresh(c, "not_taken")
        condition(ctx, region_index, j.cond, yes, no)
        start(c, yes)
        jump(ctx, region_index, false, j.target)
        start(c, no)
      }
      match block.cont {
        @f.Go(_, v) => jump(ctx, region_index, false, v)
        @f.Enter(_, v) => jump(ctx, region_index, true, v)
        @f.Halt(_) => go(c, "output")
      }
    }
  }
  start(c, "output")
  store(c, 0, imm(0), imm(source.statics.length()))
  for i, definition in source.statics {
    let v = read3(c, 3, imm(3 * i))
    store(c, 0, imm(1 + 4 * i), name(definition.label))
    write3(c, 0, imm(2 + 4 * i), v)
  }
  finish(c, [], @c.Halt)
  start(c, "runtime_error")
  ignore(bind(c, @c.Len(@c.Memory(0))))
  finish(c, [], @c.Halt)
  { memories: 5, blocks: c.blocks, }
}

///|
pub fn compiler() -> @runtime.Compiler {
  {
    compile_code: text => @c.print_code(compile(@f.parse(text))),
    encode_ainput: encode_input,
    encode_rinput: text => text,
    decode_routput: text => text,
    decode_foutput: decode_output,
  }
}
