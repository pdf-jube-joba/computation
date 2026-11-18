///|
pub fn identity_tape(text : String) -> String raise @base.Fault {
  @data.parse_tape(text, @syntax.sign).text(s => s)
}

///|
pub fn unit_text(text : String) -> String raise @base.Fault {
  @base.unit_input(text)
  ""
}

///|
pub fn compiler(
  compile : (String) -> String raise @base.Fault,
) -> @runtime.Compiler {
  {
    compile_code: compile,
    encode_ainput: identity_tape,
    encode_rinput: unit_text,
    decode_routput: unit_text,
    decode_foutput: identity_tape,
  }
}

///|
pub fn encode_tuple(numbers : Array[@base.Nat]) -> String raise @base.Fault {
  let signs = []
  for n in numbers {
    signs.push("-")
    for _ in 0..<@base.index(n) {
      signs.push("l")
    }
  }
  signs.push("x")
  @data.Tape::{ left: [], head: "x", right: signs, }.text(s => s)
}

///|
pub fn decode_number(text : String) -> String raise @base.Fault {
  let tape = @data.parse_tape(text, @syntax.sign)
  @base.require(
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
  @base.require(values.length() == 2, "expected a single output value")
  values[1].to_string()
}
