///|
fn endpoint(p : @utils.Parser) -> Endpoint raise @utils.Fault {
  let name = p.ident()
  p.expect('.')
  (name, p.ident())
}

///|
fn[T] items(
  p : @utils.Parser,
  read : (@utils.Parser) -> T raise @utils.Fault,
) -> Array[T] raise @utils.Fault {
  let g = p.group('{')
  let result = []
  while !g.eof() {
    if g.eat(',') {
      continue
    }
    result.push(read(g))
  }
  result
}

///|
fn reference(
  p : @utils.Parser,
  library : Array[(String, Circuit)],
) -> Circuit raise @utils.Fault {
  if p.peek() is @utils.Keyword(_) {
    let name = p.keyword()
    let parts = name.split("_").collect()
    let k = match parts[0].to_owned() {
      "NOT" => Not
      "AND" => And
      "OR" => Or
      "CST" => Cst
      "BR" => Br
      "DLY" => Delay
      "END" => End
      _ => p.error("Unknown builtin gate: " + name)
    }
    let state = if name == "END" {
      false
    } else {
      @utils.require(parts[0] != "END", "Unknown builtin gate: " + name)
      @utils.require(parts.length() == 2, "Unknown builtin gate: " + name)
      match parts[1].to_owned() {
        "T" => true
        "F" => false
        _ => p.error("Unknown builtin gate: " + name)
      }
    }
    Gate({ gate_kind: k, state, })
  } else {
    let name = p.ident()
    match get(library, name) {
      Some(c) => c
      None => p.error("not found name " + name)
    }
  }
}

///|
fn parse_code(text : String) -> Circuit raise @utils.Fault {
  if text.trim().has_prefix("[") {
    let c = read_circuit(@utils.parse_json(text))
    validate(c)
    return c
  }
  let p = @utils.create(text, nested_comments=false)
  let library : Array[(String, Circuit)] = []
  while !p.eof() {
    let graph = p.eat_keyword("graph")
    if !graph {
      p.expect_keyword("iter")
    }
    p.expect(':')
    let name = p.ident()
    let g = p.group('{')
    let c = if graph {
      g.expect_keyword("in")
      let pins = items(g, p => p.ident())
      g.expect_keyword("out")
      let otpin_maps = items(g, p => {
        let pin = p.ident()
        p.expect('=')
        (pin, endpoint(p))
      })
      let verts = []
      let edges = []
      let inpin_maps = []
      while !g.eof() {
        let vertex = g.ident()
        g.expect(',')
        let child = reference(g, library)
        let _ = items(g, p => {
          let pin = p.ident()
          p.expect('=')
          let first = p.ident()
          if p.eat('.') {
            let source = (first, p.ident())
            edges.push((source, (vertex, pin)))
          } else {
            @utils.require(pins.contains(first), "not found inpin " + first)
            inpin_maps.push((first, (vertex, pin)))
          }
        })
        verts.push((vertex, child))
      }
      Mix({ kind: name, verts, edges, inpin_maps, otpin_maps, })
    } else {
      let init = reference(g, library)
      g.expect(',')
      fn conn(p : @utils.Parser) -> (String, String) raise @utils.Fault {
        let a = p.ident()
        p.expect('=')
        (a, p.ident())
      }
      g.expect_keyword("next")
      let next_edges = items(g, conn)
      g.expect_keyword("prev")
      let prev_edges = items(g, conn)
      Iter({
        kind: name,
        init,
        used: [],
        next_edges,
        prev_edges,
        inpin_maps: [],
        otpin_maps: [],
      })
    }
    g.finish()
    validate(c)
    if get(library, name) is None {
      library.push((name, c))
    }
  }
  match get(library, "main") {
    Some(c) => c
    None => @utils.error("not found main")
  }
}

///|
fn read_signal(text : String) -> Signal raise @utils.Fault {
  let p = @utils.create(text)
  let result = []
  if p.eof() {
    return result
  }
  while true {
    let start = p.offset()
    let owner = if p.eat('@') {
      External(p.ident())
    } else if p.eat('$') {
      Instance(p.integer())
    } else {
      Internal(p.ident())
    }
    let owner_end = start + owner_text(owner).length()
    @utils.require(p.offset() == owner_end, "Expected contiguous endpoint.PIN")
    p.expect('.')
    @utils.require(
      p.offset() == owner_end + 1,
      "Expected contiguous endpoint.PIN",
    )
    let pin = p.ident()
    p.expect('=')
    let value = if p.eat_keyword("true") {
      true
    } else {
      p.expect_keyword("false")
      false
    }
    result.push(((owner, pin), value))
    if !p.eat(',') {
      break
    }
  }
  p.finish()
  result
}

///|
fn bool_text(value : Bool) -> String {
  if value {
    "\\true"
  } else {
    "\\false"
  }
}

///|
fn signal_text(signal : Signal) -> String {
  @utils.join(
    signal.map(entry => {
      owner_text(entry.0.0) + "." + entry.0.1 + "=" + bool_text(entry.1)
    }),
    ", ",
  )
}
