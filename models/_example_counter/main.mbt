///|
pub(all) struct Count {
  value : UInt32
}

///|
pub(all) enum Operation {
  Inc
  Dec
}

///|
pub impl @definition.Text for Count with fn parse(text) {
  let count : UInt32 = if text.trim().is_empty() {
    0
  } else {
    @utils.unsigned(text)
  }
  { value: count, }
}

///|
pub impl @definition.Text for Count with fn print(value) {
  value.value.to_string()
}

///|
pub impl @definition.Text for Operation with fn parse(text) {
  let p = @utils.create(text)
  let operation = p.keyword()
  p.finish()
  match operation {
    "inc" => Inc
    "dec" => Dec
    _ => @utils.error("Expected \\inc or \\dec")
  }
}

///|
pub impl @definition.Text for Operation with fn print(value) {
  match value {
    Inc => "\\inc"
    Dec => "\\dec"
  }
}

///|
pub type Code = Count

///|
pub type Ainput = Unit

///|
pub type Rinput = Operation

///|
pub type Routput = Event

///|
pub type Foutput = Unit

///|
pub impl @definition.Snapshot for Count with fn encode(value) {
  encode(value.value)
}

///|
pub impl @definition.Snapshot for Count with fn decode(value) {
  { value: decode(value), }
}

///|
pub impl @definition.Render for Count with fn render(value) {
  render(value.value)
}

///|
pub(all) enum Event {
  Incremented
  Decremented
}

///|
pub impl @definition.Text for Event with fn parse(text) {
  match text {
    "inc" => Incremented
    "dec" => Decremented
    _ => @utils.error("Expected inc or dec")
  }
}

///|
pub impl @definition.Text for Event with fn print(value) {
  match value {
    Incremented => "inc"
    Decremented => "dec"
  }
}

///|
fn make(code : Count, _input : Unit) -> Count {
  code
}

///|
fn step(
  state : Count,
  input : Operation,
) -> @definition.Transition[Count, Event, Unit] raise @utils.Fault {
  let count = state.value
  match input {
    Inc => {
      @utils.require(count < 4294967295U, "Counter overflow")
      let next = count + 1
      if next >= 10 {
        @definition.Halt(())
      } else {
        @definition.Continue({ value: next, }, Incremented)
      }
    }
    Dec => {
      @utils.require(count > 0, "Count cannot be negative")
      @definition.Continue({ value: count - 1, }, Decremented)
    }
  }
}

///|
fn encode(count : UInt32) -> Json {
  @utils.object([("count", @utils.juint(count))])
}

///|
fn decode(value : Json) -> UInt32 raise @utils.Fault {
  let count = @utils.uint(@utils.field(value, "count"))
  count
}

///|
fn render(count : UInt32) -> Array[@definition.Block] {
  [
    @definition.text_table("status", ["field", "value"], [
      ["count", count.to_string()],
    ]),
  ]
}

///|
pub fn model() -> @definition.Model[
  Code,
  Ainput,
  Rinput,
  Routput,
  Foutput,
  Count,
] {
  @definition.Model(make=(code, input) => make(code, input), step=(state, input) => {
    step(state, input)
  })
}
