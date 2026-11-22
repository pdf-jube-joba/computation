///|
pub(all) struct Code {
  value : @utils.LoopCode
}

///|
pub(all) struct ByteArray {
  values : Array[Int32]
}

///|
pub impl @definition.Text for Code with fn parse(text) {
  { value: parse_code(text), }
}

///|
pub impl @definition.Text for ByteArray with fn parse(text) {
  {
    values: if text.trim().is_empty() {
      []
    } else {
      @utils.array(@utils.parse_json(text)).map(byte)
    },
  }
}

///|
pub impl @definition.Text for ByteArray with fn print(value) {
  bytes_text(value.values)
}

///|
pub type Ainput = ByteArray

///|
pub type Rinput = Unit

///|
pub(all) enum Routput {
  Silent
  Wrote(Byte)
}

///|
pub impl @definition.Text for Routput with fn parse(text) {
  if text.trim().is_empty() {
    Silent
  } else {
    let value = @utils.unsigned(text)
    @utils.require(value <= 255U, "Expected a byte (0..255)")
    Wrote(value.to_byte())
  }
}

///|
pub impl @definition.Text for Routput with fn print(value) {
  match value {
    Silent => ""
    Wrote(value) => value.to_int().to_string()
  }
}

///|
pub type Foutput = ByteArray

///|
pub impl @definition.Snapshot for State with fn encode(value) {
  encode(value)
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
  @utils.join(value.value.instructions, "")
}
