///|
priv enum Instruction {
  Command(@target.Command)
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
fn emit(b : Builder, command : @target.Command) -> Unit {
  b.commands.push(Command(command))
}

///|
fn cpy(b : Builder, dst : String, src : String) -> Unit {
  emit(b, @target.Cpy(dst, src))
}

///|
fn clr(b : Builder, dst : String) -> Unit {
  emit(b, @target.Clr(dst))
}

///|
fn inc(b : Builder, dst : String) -> Unit {
  emit(b, @target.Inc(dst))
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
      emit(b, @target.Dec(counter))
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
pub fn compiler() -> @definition.Compiler[
  @source.Code,
  @source.Ainput,
  @source.Rinput,
  @source.Routput,
  @source.Foutput,
  @target.Code,
  @target.Ainput,
  @target.Rinput,
  @target.Routput,
  @target.Foutput,
] {
  @definition.Compiler(
    compile_code=f => {
      @source.validate(f)
      let b = { commands: [], labels: [], next_temp: 0, }
      clr(b, "__one")
      inc(b, "__one")
      compile_into(
        b,
        f,
        Array::makei(@source.arity(f), i => "x" + i.to_string()),
        "y",
      )
      @target.Code::{
        commands: b.commands.map(i => {
          match i {
            Command(command) => command
            Branch(v, label) =>
              @target.Ifnz(
                v,
                @utils.nint(b.labels[label].reinterpret_as_uint()),
              )
          }
        }),
      }
    },
    encode_ainput=input => {
      values: input.values.mapi((i, n) => ("x" + i.to_string(), n)),
    },
    encode_rinput=input => input,
    decode_routput=output => output,
    decode_foutput=output => {
      let result : Ref[@utils.Nat] = { val: @utils.zero(), }
      for (name, n) in output.values {
        if name == "y" {
          result.val = n
        }
      }
      result.val
    },
  )
}
