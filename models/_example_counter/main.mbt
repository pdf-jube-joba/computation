///|
fn make(code : String, input : String) -> Int32 raise @base.Fault {
  @base.unit_input(input)
  let count = if code.trim().is_empty() { 0 } else { @base.integer(code) }
  @base.require(count >= 0, "Expected a nonnegative counter")
  count
}

///|
fn step(
  count : Int32,
  input : String,
) -> @runtime.Transition[Int32] raise @base.Fault {
  let p = @p.create(input)
  let operation = p.keyword()
  p.finish()
  match operation {
    "inc" => {
      @base.require(count < 2147483647, "Counter overflow")
      let next = count + 1
      if next >= 10 {
        @runtime.Halt("End")
      } else {
        @runtime.Continue(next, "inc")
      }
    }
    "dec" => {
      @base.require(count > 0, "Count cannot be negative")
      @runtime.Continue(count - 1, "dec")
    }
    _ => @base.error("Expected \\inc or \\dec")
  }
}

///|
fn encode(count : Int32) -> Json {
  @base.object([("count", @base.jint(count))])
}

///|
fn decode(value : Json) -> Int32 raise @base.Fault {
  let count = @base.int(@base.field(value, "count"))
  @base.require(count >= 0, "Expected a nonnegative count")
  count
}

///|
fn render(count : Int32) -> Json {
  @base.jarray([
    @base.table("status", ["field", "value"], [["count", count.to_string()]]),
  ])
}

///|
pub fn model() -> @runtime.Model {
  @runtime.model(make, step, encode, decode, render)
}

///|
test "counter preserves state on input errors and halts at ten" {
  let m = model()
  m.make("8", "")
  assert_eq(@base.parse_json(m.snapshot()), @base.parse_json("{\"count\":8}"))
  assert_true(
    try {
      ignore(m.step("invalid"))
      false
    } catch {
      _ => true
    },
  )
  assert_eq(@base.parse_json(m.snapshot()), @base.parse_json("{\"count\":8}"))
  assert_eq(
    @base.string(@base.field(@base.parse_json(m.step("\\inc")), "kind")),
    "continue",
  )
  assert_eq(
    @base.string(@base.field(@base.parse_json(m.step("\\inc")), "foutput")),
    "End",
  )
  assert_true(
    try {
      ignore(m.snapshot())
      false
    } catch {
      _ => true
    },
  )
  m.restore("{\"count\":3}")
  assert_eq(
    @base.string(@base.field(@base.parse_json(m.step("\\dec")), "routput")),
    "dec",
  )
}
