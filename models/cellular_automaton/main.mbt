///|
fn row(text : String) -> Array[Bool] raise @base.Fault {
  let chars = text.trim().to_owned().to_array()
  @base.require(
    !chars.is_empty(),
    "Expected a nonempty row of 0/1 or ./# cells",
  )
  chars.map(c => {
    match c {
      '0' | '.' => false
      '1' | '#' => true
      _ => @base.error("Expected only 0/1 or ./# cells in a row")
    }
  })
}

///|
fn row_text(cells : Array[Bool]) -> String {
  @base.join(cells.map(b => if b { "1" } else { "0" }), "")
}

///|
fn row_json(cells : Array[Bool]) -> Json {
  @base.jarray(cells.map(@base.jbool))
}

///|
fn decode_row(j : Json) -> Array[Bool] raise @base.Fault {
  @base.array(j).map(@base.boolean)
}

///|
fn cell_table(cells : Array[Array[Bool]]) -> Json {
  @base.table(
    "cells",
    [],
    cells.map(r => r.map(b => if b { "■" } else { "□" })),
  )
}

///|
priv struct Elementary {
  rule : Int32
  generation : Int32
  background : Bool
  cells : Array[Bool]
}

///|
fn valid_rule(rule : Int32) -> Unit raise @base.Fault {
  @base.require(
    rule >= 0 && rule <= 255,
    "Expected an elementary cellular automaton rule (0..255)",
  )
}

///|
fn elementary_make(
  code : String,
  input : String,
) -> Elementary raise @base.Fault {
  let rule = @base.index(@base.nat(code))
  valid_rule(rule)
  { rule, generation: 0, background: false, cells: row(input), }
}

///|
fn elementary_step(
  s : Elementary,
  input : String,
) -> @runtime.Transition[Elementary] raise @base.Fault {
  @base.unit_input(input)
  @base.require(s.generation < 2147483647, "Generation overflow")
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
  @runtime.Continue(
    {
      ..s,
      generation: s.generation + 1,
      background: transition(s.background, s.background, s.background),
      cells,
    },
    row_text(cells),
  )
}

///|
fn elementary_encode(s : Elementary) -> Json {
  @base.object([
    ("rule", @base.jint(s.rule)),
    ("generation", @base.jint(s.generation)),
    ("background", @base.jbool(s.background)),
    ("cells", row_json(s.cells)),
  ])
}

///|
fn elementary_decode(j : Json) -> Elementary raise @base.Fault {
  let rule = @base.int(@base.field(j, "rule"))
  valid_rule(rule)
  let generation = @base.int(@base.field(j, "generation"))
  let cells = decode_row(@base.field(j, "cells"))
  @base.require(
    !cells.is_empty() &&
    generation >= 0 &&
    generation <= (cells.length() - 1) / 2,
    "Invalid generation for the stored row",
  )
  {
    rule,
    generation,
    cells,
    background: @base.boolean(@base.field(j, "background")),
  }
}

///|
pub fn model() -> @runtime.Model {
  @runtime.model(
    elementary_make,
    elementary_step,
    elementary_encode,
    elementary_decode,
    s => {
      @base.jarray([
        @base.titled("rule", s.rule.to_string()),
        @base.titled("generation", s.generation.to_string()),
        @base.titled("left coordinate", (-s.generation).to_string()),
        @base.titled("background", row_text([s.background])),
        cell_table([s.cells]),
      ])
    },
  )
}

///|
priv struct Life {
  generation : Int32
  cells : Array[Array[Bool]]
}

///|
fn valid_grid(cells : Array[Array[Bool]]) -> Unit raise @base.Fault {
  @base.require(
    !cells.is_empty() && !cells[0].is_empty(),
    "Expected a nonempty grid",
  )
  @base.require(
    cells.all(r => r.length() == cells[0].length()),
    "Grid rows must have the same width",
  )
}

///|
fn life_make(code : String, input : String) -> Life raise @base.Fault {
  @base.unit_input(code)
  let cells = input
    .trim()
    .to_owned()
    .split("\n")
    .collect()
    .map(r => row(r.to_owned()))
  valid_grid(cells)
  { generation: 0, cells, }
}

///|
fn life_step(
  s : Life,
  input : String,
) -> @runtime.Transition[Life] raise @base.Fault {
  @base.unit_input(input)
  @base.require(s.generation < 2147483647, "Generation overflow")
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
  @runtime.Continue(
    { generation: s.generation + 1, cells, },
    @base.join(cells.map(row_text), "\n"),
  )
}

///|
fn life_encode(s : Life) -> Json {
  @base.object([
    ("generation", @base.jint(s.generation)),
    ("cells", @base.jarray(s.cells.map(row_json))),
  ])
}

///|
fn life_decode(j : Json) -> Life raise @base.Fault {
  let generation = @base.int(@base.field(j, "generation"))
  let cells = @base.array(@base.field(j, "cells")).map(decode_row)
  valid_grid(cells)
  @base.require(
    generation >= 0 &&
    generation <= (Int32::min(cells.length(), cells[0].length()) - 1) / 2,
    "Invalid generation for the stored grid",
  )
  { generation, cells, }
}

///|
pub fn life_model() -> @runtime.Model {
  @runtime.model(life_make, life_step, life_encode, life_decode, s => {
    @base.jarray([
      @base.titled("rule", "B3/S23"),
      @base.titled("generation", s.generation.to_string()),
      @base.titled(
        "top-left coordinate",
        "(\{-s.generation}, \{-s.generation})",
      ),
      cell_table(s.cells),
    ])
  })
}
