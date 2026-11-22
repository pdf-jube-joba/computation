///|
fn name(n : String) -> @c.ValueExpr {
  @c.Imm(@utils.of_bytes(n))
}

///|
fn encode_value(v : @f.FlowValue) -> Array[@utils.Nat] {
  let zero = @utils.zero()
  match v {
    @f.Num(n) => [zero, n, zero]
    @f.Code_label(l) => [@utils.one(), @utils.of_bytes(l), zero]
    @f.Key(@f.Static(l)) => [@utils.nint(2), @utils.of_bytes(l), zero]
    @f.Key(@f.Stack(i)) =>
      [@utils.nint(3), @utils.nint(i.reinterpret_as_uint()), zero]
    @f.Key(@f.Heap(h, i)) =>
      [
        @utils.nint(4),
        @utils.nint(h.reinterpret_as_uint()),
        @utils.nint(i.reinterpret_as_uint()),
      ]
  }
}

///|
fn encode_input(input : @f.Environment) -> @c.MemoryImage {
  let input = input.values
  let values = [@utils.nint(input.length().reinterpret_as_uint())]
  for entry in input {
    values.push(@utils.of_bytes(entry.0))
    values.append(encode_value(entry.1))
  }
  {
    values: values.mapi((i, n) => {
      (@utils.nint(i.reinterpret_as_uint()), @c.Num(n))
    }),
  }
}

///|
fn decode_output(output : @c.MemoryImage) -> @f.Environment raise @utils.Fault {
  let memory = output.values
  fn read(i : Int32) -> @utils.Nat raise @utils.Fault {
    for cell in memory {
      if cell.0 == @utils.nint(i.reinterpret_as_uint()) {
        match cell.1 {
          @c.Num(n) => return n
          _ => @utils.error("Expected numeric encoded output")
        }
      }
    }
    @utils.zero()
  }
  let count = @utils.index(read(0))
  @utils.require(count <= memory.length(), "Invalid output record count")
  let values = []
  for i in 0..<count {
    let offset = 1 + 4 * i
    let label = @utils.identifier(@utils.to_bytes(read(offset)))
    let tag = @utils.index(read(offset + 1))
    let a = read(offset + 2)
    let b = read(offset + 3)
    let value : @f.FlowValue = match tag {
      0 => @f.Num(a)
      1 => @f.Code_label(@utils.identifier(@utils.to_bytes(a)))
      2 => @f.Key(@f.Static(@utils.identifier(@utils.to_bytes(a))))
      3 => @f.Key(@f.Stack(@utils.index(a)))
      4 => @f.Key(@f.Heap(@utils.index(a), @utils.index(b)))
      _ => @utils.error("Invalid flow value tag")
    }
    values.push((label, value))
  }
  { values: @utils.env_sorted(values), }
}
