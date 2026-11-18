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
pub fn compile(program : @i.Code) -> @j.Code raise @base.Fault {
  @i.validate(program)
  let variables = @names.supply()
  let labels = @names.supply()
  for f in program.functions {
    for name in @i.collect_vars(f.blocks) {
      variables.reserve(name)
    }
    for b in f.blocks {
      labels.reserve(b.label)
    }
  }
  fn expand_block(block : @i.Block) -> Array[@i.Block] raise @base.Fault {
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
          fn rename(name : String) -> String raise @base.Fault {
            match label_map.get(name) {
              Some(n) => n
              None => @base.error("Unknown label: " + name)
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
    @base.require(
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
              None => @base.error("jump target '" + label + "' not found")
            }
            @j.Jump(target, c)
          }
          @i.Return(c) => @j.Jump(offset, c)
          _ => @base.error("Unflattened instruction")
        },
      )
    }
  }
  let result = @j.Code::{ alphabet: program.alphabet, body, }
  @j.validate(result)
  result
}

///|
fn tape(text : String) -> String raise @base.Fault {
  @data.parse_tape(text, @s.sign).text(s => s)
}

///|
pub fn compiler() -> @runtime.Compiler {
  {
    compile_code: text => @j.print_code(compile(@i.parse_code(text))),
    encode_ainput: tape,
    encode_rinput: text => {
      @base.unit_input(text)
      ""
    },
    decode_routput: text => {
      @base.unit_input(text)
      ""
    },
    decode_foutput: tape,
  }
}

///|
test "inlined calls reset locals and preserve nested call continuations" {
  let code = "\\alphabet: (-,x) \\fn inner { \\label entry: { tmp := @ @ := \\const x \\return } } \\fn outer { \\label entry: { \\call inner \\RT \\return } } \\fn main { \\label entry: { tmp := @ \\call outer \\call outer @ := tmp \\return } }"
  let target = @j.model()
  let c = compiler()
  target.make((c.compile_code)(code), (c.encode_ainput)("|-|"))
  let mut output = None
  for _ in 0..<100 {
    let j = @base.parse_json(target.step(""))
    if @base.string(@base.field(j, "kind")) == "halt" {
      output = Some(@base.string(@base.field(j, "foutput")))
      break
    }
  }
  assert_eq(output, Some("x,x|-|"))
}
