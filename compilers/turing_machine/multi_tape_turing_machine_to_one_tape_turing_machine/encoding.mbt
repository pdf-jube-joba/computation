///|
fn cell(tape : Int32, head : Bool, sign : String) -> String {
  "mt" +
  tape.to_string() +
  "_" +
  (if head { "h" } else { "p" }) +
  "_" +
  (if sign == "-" { "B" } else { "S" + sign })
}

///|
fn read_cell(sign : String) -> (Int32, Bool, String) raise @base.Fault {
  let parts = sign.split("_").map(p => p.to_owned()).collect()
  @base.require(
    parts.length() >= 3 && parts[0].has_prefix("mt"),
    "Invalid encoded tape cell",
  )
  let tape = @base.integer(
    parts[0].unsafe_substring(start=2, end=parts[0].length()),
  )
  @base.require(tape >= 0, "Invalid encoded tape cell")
  let head = match parts[1] {
    "h" => true
    "p" => false
    _ => @base.error("Invalid encoded tape cell")
  }
  let payload = @base.join(parts[2:].to_owned(), "_")
  let sign = if payload == "B" {
    "-"
  } else {
    @base.require(payload.has_prefix("S"), "Invalid encoded tape cell")
    @base.identifier(payload.unsafe_substring(start=1, end=payload.length()))
  }
  (tape, head, sign)
}

///|
fn encode_input(text : String) -> String raise @base.Fault {
  let tapes = text
    .split("\n")
    .map(s => s.trim().to_owned())
    .filter(s => !s.is_empty())
    .collect()
    .map(s => @data.parse_tape(s, @tm.sign))
  @base.require(!tapes.is_empty(), "Expected at least one tape")
  let mut left = 0
  let mut right = 0
  for t in tapes {
    left = left.max(t.left.length())
    right = right.max(t.right.length())
  }
  let columns = []
  for position = -left; position <= right; position = position + 1 {
    for id, tape in tapes {
      let sign = if position == 0 {
        tape.head
      } else if position < 0 {
        if -position - 1 < tape.left.length() {
          tape.left[-position - 1]
        } else {
          "-"
        }
      } else if position - 1 < tape.right.length() {
        tape.right[position - 1]
      } else {
        "-"
      }
      columns.push(cell(id, position == 0, sign))
    }
  }
  columns.push("mt_right")
  @data.Tape::{ left: [], head: "mt_left", right: columns, }.text(s => s)
}

///|
fn trim_side(signs : Array[String]) -> Array[String] {
  let mut end = signs.length()
  while end > 0 && signs[end - 1] == "-" {
    end -= 1
  }
  signs[:end].to_owned()
}

///|
fn decode_output(text : String) -> String raise @base.Fault {
  let tape = @data.parse_tape(text, @tm.sign)
  @base.require(
    tape.head == "mt_left" && tape.left.all(s => s == "-"),
    "Expected encoded output head at left boundary",
  )
  let cells = []
  let mut found = false
  for sign in tape.right {
    if found {
      @base.require(sign == "-", "Data outside encoded right boundary")
    } else if sign == "mt_right" {
      found = true
    } else {
      cells.push(read_cell(sign))
    }
  }
  @base.require(found, "Missing encoded right boundary")
  let mut width = 0
  for entry in cells {
    if entry.0 == 0 && width > 0 {
      break
    }
    @base.require(entry.0 == width, "Invalid encoded column order")
    width += 1
  }
  @base.require(
    width > 0 && cells.length() % width == 0,
    "Incomplete encoded column",
  )
  let signs : Array[Array[String]] = Array::makei(width, _ => [])
  let heads : Array[Int32?] = Array::make(width, None)
  for pos, entry in cells {
    let (id, head, sign) = entry
    @base.require(id == pos % width, "Invalid encoded column order")
    if head {
      @base.require(heads[id] is None, "Duplicate encoded head")
      heads[id] = Some(pos / width)
    }
    signs[id].push(sign)
  }
  let tapes = signs.mapi((id, signs) => {
    let head = match heads[id] {
      Some(h) => h
      None => @base.error("Missing encoded head")
    }
    @data.Tape::{
      left: trim_side(signs[:head].to_owned().rev()),
      head: signs[head],
      right: trim_side(signs[head + 1:].to_owned()),
    }.text(s => s)
  })
  @base.join(tapes, "\n")
}
