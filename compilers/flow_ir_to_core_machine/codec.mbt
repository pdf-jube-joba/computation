///|
fn name(n : String) -> @c.ValueExpr {
  @c.Imm(@data.of_bytes(n))
}

///|
fn encode_value(v : @f.FlowValue) -> Array[@base.Nat] {
  let zero = @base.zero()
  match v {
    @f.Num(n) => [zero, n, zero]
    @f.Code_label(l) => [@base.one(), @data.of_bytes(l), zero]
    @f.Key(@f.Static(l)) => [@base.nint(2), @data.of_bytes(l), zero]
    @f.Key(@f.Stack(i)) => [@base.nint(3), @base.nint(i), zero]
    @f.Key(@f.Heap(h, i)) => [@base.nint(4), @base.nint(h), @base.nint(i)]
  }
}

///|
fn encode_input(text : String) -> String raise @base.Fault {
  let input = @f.parse_input(text)
  let values = [@base.nint(input.length())]
  for entry in input {
    values.push(@data.of_bytes(entry.0))
    values.append(encode_value(entry.1))
  }
  @c.output(values.mapi((i, n) => (@base.nint(i), @c.Num(n))))
}

///|
fn decode_output(text : String) -> String raise @base.Fault {
  let memory = @c.parse_input(text)
  fn read(i : Int32) -> @base.Nat raise @base.Fault {
    for cell in memory {
      if cell.0 == @base.nint(i) {
        match cell.1 {
          @c.Num(n) => return n
          _ => @base.error("Expected numeric encoded output")
        }
      }
    }
    @base.zero()
  }
  let count = @base.index(read(0))
  @base.require(count <= memory.length(), "Invalid output record count")
  let values = []
  for i in 0..<count {
    let offset = 1 + 4 * i
    let label = @base.identifier(@data.to_bytes(read(offset)))
    let tag = @base.index(read(offset + 1))
    let a = read(offset + 2)
    let b = read(offset + 3)
    let value : @f.FlowValue = match tag {
      0 => @f.Num(a)
      1 => @f.Code_label(@base.identifier(@data.to_bytes(a)))
      2 => @f.Key(@f.Static(@base.identifier(@data.to_bytes(a))))
      3 => @f.Key(@f.Stack(@base.index(a)))
      4 => @f.Key(@f.Heap(@base.index(a), @base.index(b)))
      _ => @base.error("Invalid flow value tag")
    }
    values.push((label, value))
  }
  @data.print_env(@data.env_sorted(values), @f.flow_text)
}
