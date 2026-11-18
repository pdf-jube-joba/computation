///|
priv enum Instruction {
  Command(String)
  Branch(String, Int32)
}

///|
priv struct Builder {
  commands : Array[Instruction]
  labels : Array[Int32]
  mut next_temp : Int32
}

///|
fn label(b : Builder) -> Int32 {
  let i = b.labels.length()
  b.labels.push(-1)
  i
}

///|
fn mark(b : Builder, i : Int32) -> Unit {
  b.labels[i] = b.commands.length()
}

///|
fn temp(b : Builder) -> String {
  let n = b.next_temp
  b.next_temp += 1
  if n == 0 {
    "__tmp"
  } else {
    "__tmp_" + (n - 1).to_string()
  }
}

///|
fn emit(b : Builder, code : String) -> Unit {
  b.commands.push(Command(code + ";"))
}

///|
fn cpy(b : Builder, dst : String, src : String) -> Unit {
  emit(b, "\\cpy " + dst + " <- " + src)
}

///|
fn clr(b : Builder, dst : String) -> Unit {
  emit(b, "\\clr " + dst)
}

///|
fn inc(b : Builder, dst : String) -> Unit {
  emit(b, "\\inc " + dst)
}

///|
fn branch(b : Builder, v : String, l : Int32) -> Unit {
  b.commands.push(Branch(v, l))
}

///|
fn go(b : Builder, l : Int32) -> Unit {
  branch(b, "__one", l)
}

///|
fn compile_into(
  b : Builder,
  f : @source.Func,
  args : Array[String],
  dst : String,
) -> Unit {
  match f {
    @source.Zero => clr(b, dst)
    @source.Succ => {
      cpy(b, dst, args[0])
      inc(b, dst)
    }
    @source.Proj(_, i) => cpy(b, dst, args[i])
    @source.Comp(f, inners) => {
      let args = inners.map(f => {
        let tmp = temp(b)
        compile_into(b, f, args, tmp)
        tmp
      })
      compile_into(b, f, args, dst)
    }
    @source.Prim(z, s) => {
      let rest = args[1:].to_owned()
      let result = temp(b)
      let counter = temp(b)
      let index = temp(b)
      let succ_value = temp(b)
      compile_into(b, z, rest, result)
      cpy(b, counter, args[0])
      clr(b, index)
      let condition = label(b)
      let body = label(b)
      let finish = label(b)
      mark(b, condition)
      branch(b, counter, body)
      go(b, finish)
      mark(b, body)
      let inputs = [result, index]
      inputs.append(rest)
      compile_into(b, s, inputs, succ_value)
      cpy(b, result, succ_value)
      inc(b, index)
      emit(b, "\\dec " + counter)
      go(b, condition)
      mark(b, finish)
      cpy(b, dst, result)
    }
    @source.Mu(f) => {
      let index = temp(b)
      let value = temp(b)
      clr(b, index)
      let start = label(b)
      let again = label(b)
      let finish = label(b)
      mark(b, start)
      let inputs = [index]
      inputs.append(args)
      compile_into(b, f, inputs, value)
      branch(b, value, again)
      cpy(b, dst, index)
      go(b, finish)
      mark(b, again)
      inc(b, index)
      go(b, start)
      mark(b, finish)
    }
  }
}

///|
pub fn compiler() -> @runtime.Compiler {
  {
    compile_code: text => {
      let f = @source.parse_code(text)
      let b = { commands: [], labels: [], next_temp: 0, }
      clr(b, "__one")
      inc(b, "__one")
      compile_into(
        b,
        f,
        Array::makei(@source.arity(f), i => "x" + i.to_string()),
        "y",
      )
      @base.join(
        b.commands.map(i => {
          match i {
            Command(text) => text
            Branch(v, label) =>
              "\\ifnz " + v + " : " + b.labels[label].to_string() + ";"
          }
        }),
        "\n",
      )
    },
    encode_ainput: text => {
      @base.join(
        @source.parse_input(text).mapi((i, n) => {
          "x" + i.to_string() + " = " + n.to_string()
        }),
        "\n",
      )
    },
    encode_rinput: text => {
      @base.unit_input(text)
      ""
    },
    decode_routput: text => {
      @base.unit_input(text)
      ""
    },
    decode_foutput: text => {
      let p = @p.create(text)
      let result : Ref[@base.Nat] = { val: @base.zero(), }
      while !p.eof() {
        let name = p.ident()
        p.expect('=')
        let n = p.natural()
        if name == "y" {
          result.val = n
        }
        ignore(p.eat(';'))
      }
      result.val.to_string()
    },
  }
}
