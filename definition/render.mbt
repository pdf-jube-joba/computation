///|
pub(all) enum Orientation {
  Vertical
  Horizontal
}

///|
pub(all) enum Display {
  Inline
  Block
}

///|
pub(all) struct GraphNode {
  id : String
  label : String
  ports : Array[String]
}

///|
pub(all) struct GraphEndpoint {
  node : String
  port : Int32
}

///|
pub(all) struct GraphEdge {
  source : GraphEndpoint
  target : GraphEndpoint
  highlighted : Bool
}

///|
pub(all) enum Block {
  Text(value~ : String, title~ : String?, class_name~ : String?)
  Table(
    title~ : String?,
    columns~ : Array[Block],
    rows~ : Array[Array[Block]],
    highlight~ : Int32?,
  )
  Container(
    title~ : String?,
    orientation~ : Orientation,
    display~ : Display,
    children~ : Array[Block],
  )
  Graph(title~ : String?, nodes~ : Array[GraphNode], edges~ : Array[GraphEdge])
}

///|
pub fn text(
  value : String,
  title~ : String? = None,
  class_name~ : String? = None,
) -> Block {
  Text(value~, title~, class_name~)
}

///|
pub fn titled(title : String, value : String) -> Block {
  text(value, title=Some(title))
}

///|
pub fn table(
  rows : Array[Array[Block]],
  title~ : String? = None,
  columns~ : Array[Block] = [],
  highlight~ : Int32? = None,
) -> Block {
  Table(title~, columns~, rows~, highlight~)
}

///|
pub fn text_table(
  title : String,
  columns : Array[String],
  rows : Array[Array[String]],
  highlight~ : Int32 = -1,
) -> Block {
  table(
    rows.map(row => row.map(value => text(value))),
    title=Some(title),
    columns=columns.map(value => text(value)),
    highlight=if highlight < 0 { None } else { Some(highlight) },
  )
}

///|
pub fn container(
  title : String,
  children : Array[Block],
  orientation~ : Orientation = Vertical,
  display~ : Display = Block,
) -> Block {
  Container(title=Some(title), orientation~, display~, children~)
}

///|
pub fn highlight(
  title : String,
  index : Int32,
  values : Array[String],
) -> Block {
  container(
    title,
    values.mapi((i, value) => {
      text(value, class_name=if i == index { Some("highlight") } else { None })
    }),
    orientation=Horizontal,
  )
}

///|
pub fn graph(
  title : String,
  nodes : Array[GraphNode],
  edges : Array[GraphEdge],
) -> Block {
  Graph(title=Some(title), nodes~, edges~)
}
