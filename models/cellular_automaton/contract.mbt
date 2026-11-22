///|
pub(all) struct Rule {
  value : Int32
}

///|
pub(all) struct Row {
  cells : Array[Bool]
}

///|
pub(all) struct Grid {
  cells : Array[Array[Bool]]
}

///|
pub impl @definition.Text for Rule with fn parse(text) {
  let rule = @utils.index(@utils.nat(text))
  valid_rule(rule)
  { value: rule, }
}

///|
pub impl @definition.Text for Row with fn parse(text) {
  { cells: row(text), }
}

///|
pub impl @definition.Text for Row with fn print(value) {
  row_text(value.cells)
}

///|
pub impl @definition.Text for Grid with fn parse(text) {
  let cells = text
    .trim()
    .to_owned()
    .split("\n")
    .collect()
    .map(r => row(r.to_owned()))
  valid_grid(cells)
  { cells, }
}

///|
pub impl @definition.Text for Grid with fn print(value) {
  @utils.join(value.cells.map(row_text), "\n")
}

///|
pub impl @definition.Snapshot for Elementary with fn encode(value) {
  elementary_encode(value)
}

///|
pub impl @definition.Snapshot for Elementary with fn decode(value) {
  elementary_decode(value)
}

///|
pub impl @definition.Render for Elementary with fn render(s) {
  [
    @definition.titled("rule", s.rule.to_string()),
    @definition.titled("generation", s.generation.to_string()),
    @definition.titled("left coordinate", (-s.generation).to_string()),
    @definition.titled("background", row_text([s.background])),
    cell_table([s.cells]),
  ]
}

///|
pub impl @definition.Snapshot for Life with fn encode(value) {
  life_encode(value)
}

///|
pub impl @definition.Snapshot for Life with fn decode(value) {
  life_decode(value)
}

///|
pub impl @definition.Render for Life with fn render(s) {
  [
    @definition.titled("rule", "B3/S23"),
    @definition.titled("generation", s.generation.to_string()),
    @definition.titled(
      "top-left coordinate",
      "(\{-s.generation}, \{-s.generation})",
    ),
    cell_table(s.cells),
  ]
}

///|
pub impl @definition.Text for Rule with fn print(value) {
  value.value.to_string()
}
