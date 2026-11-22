///|
fn row(text : String) -> Array[Bool] raise @utils.Fault {
  let chars = text.trim().to_owned().to_array()
  @utils.require(
    !chars.is_empty(),
    "Expected a nonempty row of 0/1 or ./# cells",
  )
  chars.map(c => {
    match c {
      '0' | '.' => false
      '1' | '#' => true
      _ => @utils.error("Expected only 0/1 or ./# cells in a row")
    }
  })
}

///|
fn row_text(cells : Array[Bool]) -> String {
  @utils.join(cells.map(b => if b { "1" } else { "0" }), "")
}

///|
fn row_json(cells : Array[Bool]) -> Json {
  @utils.jarray(cells.map(@utils.jbool))
}

///|
fn decode_row(j : Json) -> Array[Bool] raise @utils.Fault {
  @utils.array(j).map(@utils.boolean)
}

///|
fn cell_table(cells : Array[Array[Bool]]) -> @definition.Block {
  @definition.text_table(
    "cells",
    [],
    cells.map(r => r.map(b => if b { "■" } else { "□" })),
  )
}

///|
struct Elementary {
  rule : Int32
  generation : Int32
  background : Bool
  cells : Array[Bool]
}

///|
fn valid_rule(rule : Int32) -> Unit raise @utils.Fault {
  @utils.require(
    rule >= 0 && rule <= 255,
    "Expected an elementary cellular automaton rule (0..255)",
  )
}

///|
fn elementary_make(code : Rule, input : Row) -> Elementary raise @utils.Fault {
  valid_rule(code.value)
  { rule: code.value, generation: 0, background: false, cells: input.cells, }
}

///|
fn elementary_step(
  s : Elementary,
  _input : Unit,
) -> @definition.Transition[Elementary, Row, Unit] raise @utils.Fault {
  @utils.require(s.generation < 2147483647, "Generation overflow")
  fn at(i) {
    if i < 0 || i >= s.cells.length() {
      s.background
    } else {
      s.cells[i]
    }
  }
  fn transition(a : Bool, b : Bool, c : Bool) -> Bool {
    let i = (if a { 4 } else { 0 }) +
      (if b { 2 } else { 0 }) +
      (if c { 1 } else { 0 })
    (s.rule & (1 << i)) != 0
  }
  let cells = Array::makei(s.cells.length() + 2, i => {
    transition(at(i - 2), at(i - 1), at(i))
  })
  @definition.Continue(
    {
      ..s,
      generation: s.generation + 1,
      background: transition(s.background, s.background, s.background),
      cells,
    },
    Row::{ cells, },
  )
}

///|
fn elementary_encode(s : Elementary) -> Json {
  @utils.object([
    ("rule", @utils.jint(s.rule)),
    ("generation", @utils.jint(s.generation)),
    ("background", @utils.jbool(s.background)),
    ("cells", row_json(s.cells)),
  ])
}

///|
fn elementary_decode(j : Json) -> Elementary raise @utils.Fault {
  let rule = @utils.int(@utils.field(j, "rule"))
  valid_rule(rule)
  let generation = @utils.int(@utils.field(j, "generation"))
  let cells = decode_row(@utils.field(j, "cells"))
  @utils.require(
    !cells.is_empty() &&
    generation >= 0 &&
    generation <= (cells.length() - 1) / 2,
    "Invalid generation for the stored row",
  )
  {
    rule,
    generation,
    cells,
    background: @utils.boolean(@utils.field(j, "background")),
  }
}

///|
pub fn model() -> @definition.Model[Rule, Row, Unit, Row, Unit, Elementary] {
  @definition.Model(make=(code, input) => elementary_make(code, input), step=(
    state,
    input,
  ) => elementary_step(state, input))
}

///|
struct Life {
  generation : Int32
  cells : Array[Array[Bool]]
}

///|
fn valid_grid(cells : Array[Array[Bool]]) -> Unit raise @utils.Fault {
  @utils.require(
    !cells.is_empty() && !cells[0].is_empty(),
    "Expected a nonempty grid",
  )
  @utils.require(
    cells.all(r => r.length() == cells[0].length()),
    "Grid rows must have the same width",
  )
}

///|
fn life_make(_code : Unit, input : Grid) -> Life raise @utils.Fault {
  valid_grid(input.cells)
  { generation: 0, cells: input.cells, }
}

///|
fn life_step(
  s : Life,
  _input : Unit,
) -> @definition.Transition[Life, Grid, Unit] raise @utils.Fault {
  @utils.require(s.generation < 2147483647, "Generation overflow")
  let height = s.cells.length()
  let width = s.cells[0].length()
  fn at(x, y) {
    x >= 0 && y >= 0 && x < width && y < height && s.cells[y][x]
  }
  let cells = Array::makei(height + 2, r => {
    Array::makei(width + 2, c => {
      let x = c - 1
      let y = r - 1
      let mut neighbours = 0
      for dy in -1..<2 {
        for dx in -1..<2 {
          if (dx != 0 || dy != 0) && at(x + dx, y + dy) {
            neighbours += 1
          }
        }
      }
      neighbours == 3 || (neighbours == 2 && at(x, y))
    })
  })
  @definition.Continue({ generation: s.generation + 1, cells, }, Grid::{
    cells,
  })
}

///|
fn life_encode(s : Life) -> Json {
  @utils.object([
    ("generation", @utils.jint(s.generation)),
    ("cells", @utils.jarray(s.cells.map(row_json))),
  ])
}

///|
fn life_decode(j : Json) -> Life raise @utils.Fault {
  let generation = @utils.int(@utils.field(j, "generation"))
  let cells = @utils.array(@utils.field(j, "cells")).map(decode_row)
  valid_grid(cells)
  @utils.require(
    generation >= 0 &&
    generation <= (Int32::min(cells.length(), cells[0].length()) - 1) / 2,
    "Invalid generation for the stored grid",
  )
  { generation, cells, }
}

///|
pub fn life_model() -> @definition.Model[Unit, Grid, Unit, Grid, Unit, Life] {
  @definition.Model(make=(code, input) => life_make(code, input), step=(
    state,
    input,
  ) => life_step(state, input))
}
