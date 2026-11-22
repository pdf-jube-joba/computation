///|
pub fn encode_tuple(
  numbers : Array[@utils.Nat],
) -> @syntax.Tape raise @utils.Fault {
  let signs = []
  for n in numbers {
    signs.push("-")
    for _ in 0..<@utils.index(n) {
      signs.push("l")
    }
  }
  signs.push("x")
  { value: @utils.Tape::{ left: [], head: "x", right: signs, }, }
}

///|
pub fn decode_number(input : @syntax.Tape) -> @utils.Nat raise @utils.Fault {
  let tape = input.value
  @utils.require(
    tape.head == "x",
    "failed to decode tape as recursive_function output",
  )
  let values = []
  let mut current = 0
  for sign in tape.right {
    if sign == "x" {
      break
    }
    if sign == "-" {
      values.push(current)
      current = 0
    } else {
      current += 1
    }
  }
  values.push(current)
  @utils.require(values.length() == 2, "expected a single output value")
  @utils.nint(values[1].reinterpret_as_uint())
}
