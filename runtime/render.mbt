///|
fn optional_string(
  fields : Array[(String, Json)],
  name : String,
  value : String?,
) -> Unit {
  match value {
    Some(value) => fields.push((name, @utils.jstr(value)))
    None => ()
  }
}

///|
fn endpoint_json(endpoint : @definition.GraphEndpoint) -> Json {
  @utils.object([
    ("node", @utils.jstr(endpoint.node)),
    ("port", @utils.jint(endpoint.port)),
  ])
}

///|
fn block_json(block : @definition.Block) -> Json {
  match block {
    @definition.Text(value~, title~, class_name~) => {
      let fields = [("kind", @utils.jstr("text")), ("text", @utils.jstr(value))]
      optional_string(fields, "title", title)
      optional_string(fields, "className", class_name)
      @utils.object(fields)
    }
    @definition.Table(title~, columns~, rows~, highlight~) => {
      let fields = [
        ("kind", @utils.jstr("table")),
        ("columns", @utils.jarray(columns.map(block_json))),
        (
          "rows",
          @utils.jarray(
            rows.mapi((index, cells) => {
              let fields = [("cells", @utils.jarray(cells.map(block_json)))]
              if highlight == Some(index) {
                fields.push(("className", @utils.jstr("highlight")))
              }
              @utils.object(fields)
            }),
          ),
        ),
      ]
      optional_string(fields, "title", title)
      @utils.object(fields)
    }
    @definition.Container(title~, orientation~, display~, children~) => {
      let fields = [
        ("kind", @utils.jstr("container")),
        (
          "orientation",
          @utils.jstr(
            match orientation {
              @definition.Vertical => "vertical"
              @definition.Horizontal => "horizontal"
            },
          ),
        ),
        (
          "display",
          @utils.jstr(
            match display {
              @definition.Inline => "inline"
              @definition.Block => "block"
            },
          ),
        ),
        ("children", @utils.jarray(children.map(block_json))),
      ]
      optional_string(fields, "title", title)
      @utils.object(fields)
    }
    @definition.Graph(title~, nodes~, edges~) => {
      let fields = [
        ("kind", @utils.jstr("graph")),
        (
          "nodes",
          @utils.jarray(
            nodes.map(node => {
              @utils.object([
                ("id", @utils.jstr(node.id)),
                ("label", @utils.jstr(node.label)),
                ("ports", @utils.jarray(node.ports.map(@utils.jstr))),
              ])
            }),
          ),
        ),
        (
          "edges",
          @utils.jarray(
            edges.map(edge => {
              @utils.object([
                ("source", endpoint_json(edge.source)),
                ("target", endpoint_json(edge.target)),
                ("highlighted", @utils.jbool(edge.highlighted)),
              ])
            }),
          ),
        ),
      ]
      optional_string(fields, "title", title)
      @utils.object(fields)
    }
  }
}

///|
fn render_json(blocks : Array[@definition.Block]) -> Json {
  @utils.jarray(blocks.map(block_json))
}
