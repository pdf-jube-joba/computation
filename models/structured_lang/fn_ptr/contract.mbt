///|
struct Final {
  current_function : String
  finished : FrameStore
  remaining_store : Array[FrameStore]
}

///|
pub impl @definition.Text for Code with fn parse(text) {
  parse_code(text)
}

///|
pub impl @definition.Text for Final with fn print(value) {
  final_output(value)
}

///|
pub type Ainput = Unit

///|
pub type Rinput = Unit

///|
pub type Routput = Unit

///|
pub type Foutput = Final

///|
pub impl @definition.Snapshot for State with fn encode(value) {
  json_state(value)
}

///|
pub impl @definition.Snapshot for State with fn decode(value) {
  decode(value)
}

///|
pub impl @definition.Render for State with fn render(value) {
  render(value)
}

///|
pub impl @definition.Text for Code with fn print(value) {
  @utils.join(
    value.functions.map(f => {
      "\\fn " +
      f.name +
      "(" +
      @utils.join(f.params, ", ") +
      ") { " +
      @utils.join(f.body.map(stmt_text), "; ") +
      " }"
    }),
    "\n",
  )
}

///|
pub impl @definition.Text for Final with fn parse(text) {
  let p = @utils.create(text)
  fn location(p : @utils.Parser) -> Location raise @utils.Fault {
    p.expect_keyword("loc")
    let g = p.group('(')
    let frame_id = g.integer()
    g.expect(',')
    let slot = g.integer()
    g.finish()
    { frame_id, slot, }
  }
  fn value(p : @utils.Parser) -> Value raise @utils.Fault {
    if p.peek() == @utils.Keyword("loc") {
      Location(location(p))
    } else if p.eat_keyword("null_ptr") {
      Null_ptr
    } else {
      let sign = if p.eat('-') {
        "-"
      } else {
        ignore(p.eat('+'))
        ""
      }
      let digits = p.natural().to_string()
      Num(
        @string.parse_int64(sign + digits) catch {
          _ => @utils.error("Integer out of range")
        },
      )
    }
  }
  fn frame(
    p : @utils.Parser,
    frame_id : Int32,
  ) -> FrameStore raise @utils.Fault {
    let values : Array[(Location, Value)] = []
    while p.peek() == @utils.Keyword("loc") {
      let key = location(p)
      @utils.require(
        key.frame_id == frame_id,
        "Location belongs to a different frame",
      )
      @utils.require(!values.any(pair => pair.0 == key), "Duplicate location")
      p.expect('=')
      values.push((key, value(p)))
    }
    { frame_id, values, }
  }
  p.expect_text("halted current_function:")
  let current_function = p.ident()
  p.expect_text("finished_frame:")
  let finished = frame(p, p.integer())
  p.expect_text("remaining_store:")
  let remaining_store : Array[FrameStore] = []
  if p.eat('<') {
    p.expect_text("empty>")
  } else {
    while !p.eof() {
      p.expect_text("frame")
      let id = p.integer()
      p.expect(':')
      @utils.require(
        !remaining_store.any(f => f.frame_id == id),
        "Duplicate frame",
      )
      remaining_store.push(frame(p, id))
    }
  }
  p.finish()
  remaining_store.rev_in_place()
  { current_function, finished, remaining_store, }
}
