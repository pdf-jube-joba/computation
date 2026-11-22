///|
fn lower_blocks(blocks : Array[@i.Block]) -> Array[@i.Block] {
  blocks.mapi((index, b) => {
    ..b,
    body: b.body.map(s => {
      match s {
        @i.Continue(c) => @i.Jump(b.label, c)
        @i.Break(c) =>
          if index + 1 == blocks.length() {
            @i.Return(c)
          } else {
            @i.Jump(blocks[index + 1].label, c)
          }
        _ => s
      }
    }),
  })
}

///|
pub fn compile(program : @i.Code) -> @j.Code raise @utils.Fault {
  @i.validate(program)
  let variables = @utils.supply()
  let labels = @utils.supply()
  for f in program.functions {
    for name in @i.collect_vars(f.blocks) {
      variables.reserve(name)
    }
    for b in f.blocks {
      labels.reserve(b.label)
    }
  }
  fn expand_block(block : @i.Block) -> Array[@i.Block] raise @utils.Fault {
    let prefix = []
    for index, stmt in block.body {
      match stmt {
        @i.Call(name) => {
          let callee = @i.find_function(program, name)
          let vars = @i.collect_vars(callee.blocks)
          let variable_map : Map[String, String] = Map([])
          let label_map : Map[String, String] = Map([])
          for name in vars {
            variable_map[name] = variables.fresh(name)
          }
          for b in callee.blocks {
            label_map[b.label] = labels.fresh(b.label)
          }
          let cont = labels.fresh("call_ret")
          let init_label = labels.fresh("call_init")
          fn variable_name(name : String) -> String {
            variable_map.get(name).unwrap_or(name)
          }
          fn rename(name : String) -> String raise @utils.Fault {
            match label_map.get(name) {
              Some(n) => n
              None => @utils.error("Unknown label: " + name)
            }
          }
          fn lv(v : @s.Lvalue) -> @s.Lvalue {
            match v {
              @s.PutHead => @s.PutHead
              @s.PutVar(n) => @s.PutVar(variable_name(n))
            }
          }
          fn rv(v : @s.Rvalue) -> @s.Rvalue {
            match v {
              @s.Var(n) => @s.Var(variable_name(n))
              _ => v
            }
          }
          fn cond(c : @s.Condition?) -> @s.Condition? {
            c.map(c => { left: rv(c.left), right: rv(c.right), })
          }
          let renamed : Array[@i.Block] = lower_blocks(callee.blocks).map(b => {
            label: rename(b.label),
            body: b.body.map(s => {
              match s {
                @i.Assign(dst, src) => @i.Assign(lv(dst), rv(src))
                @i.Break(c) => @i.Break(cond(c))
                @i.Continue(c) => @i.Continue(cond(c))
                @i.Jump(label, c) => @i.Jump(rename(label), cond(c))
                @i.Return(c) => @i.Jump(cont, cond(c))
                _ => s
              }
            }),
          })
          let blocks : Array[@i.Block] = [
            {
              label: init_label,
              body: vars.map(n => {
                @i.Assign(@s.PutVar(variable_name(n)), @s.Const("-"))
              }),
            },
          ]
          blocks.append(renamed)
          let result = [{ ..block, body: prefix, }]
          for b in blocks {
            result.append(expand_block(b))
          }
          result.append(
            expand_block({
              label: cont,
              body: block.body[index + 1:].to_owned(),
            }),
          )
          return result
        }
        _ => prefix.push(stmt)
      }
    }
    [{ ..block, body: prefix, }]
  }
  let main = @i.find_function(program, "main")
  let blocks = []
  for b in lower_blocks(main.blocks) {
    blocks.append(expand_block(b))
  }
  let offsets : Map[String, Int32] = Map([])
  let mut offset = 0
  for b in blocks {
    @utils.require(
      !offsets.contains(b.label),
      "label '" + b.label + "' is duplicated",
    )
    offsets[b.label] = offset
    offset += b.body.length()
  }
  let body = []
  for b in lower_blocks(blocks) {
    for stmt in b.body {
      body.push(
        match stmt {
          @i.Lt => @j.Lt
          @i.Rt => @j.Rt
          @i.Assign(dst, src) => @j.Assign(dst, src)
          @i.Jump(label, c) => {
            let target = match offsets.get(label) {
              Some(i) => i
              None => @utils.error("jump target '" + label + "' not found")
            }
            @j.Jump(target, c)
          }
          @i.Return(c) => @j.Jump(offset, c)
          _ => @utils.error("Unflattened instruction")
        },
      )
    }
  }
  let result = @j.Code::{ alphabet: program.alphabet, body, }
  @j.validate(result)
  result
}

///|
pub fn compiler() -> @definition.Compiler[
  @i.Code,
  @i.Ainput,
  @i.Rinput,
  @i.Routput,
  @i.Foutput,
  @j.Code,
  @j.Ainput,
  @j.Rinput,
  @j.Routput,
  @j.Foutput,
] {
  @definition.Compiler(
    compile_code=compile,
    encode_ainput=input => input,
    encode_rinput=input => input,
    decode_routput=output => output,
    decode_foutput=output => output,
  )
}
