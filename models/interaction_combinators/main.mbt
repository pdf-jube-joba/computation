///|
#derive(Eq)
priv enum Kind {
  Gamma
  Delta
  Epsilon
}

///|
priv struct Agent {
  id : String
  kind : Kind
}

///|
#derive(Eq, Hash)
priv enum Port {
  AgentPort(String, Int32)
  Free(String)
}

///|
struct Net {
  agents : Array[Agent]
  wires : Array[(Port, Port)]
  loops : Int32
}

///|
struct State {
  net : Net
  steps : Int32
}

///|
fn arity(k : Kind) -> Int32 {
  if k == Epsilon {
    0
  } else {
    2
  }
}

///|
fn ordinal(k : Kind) -> Int32 {
  match k {
    Gamma => 0
    Delta => 1
    Epsilon => 2
  }
}

///|
fn kind_name(k : Kind) -> String {
  match k {
    Gamma => "gamma"
    Delta => "delta"
    Epsilon => "epsilon"
  }
}

///|
fn port_name(p : Port) -> String {
  match p {
    AgentPort(n, i) => n + "." + i.to_string()
    Free(n) => "@" + n
  }
}

///|
fn port_compare(a : Port, b : Port) -> Int32 {
  match (a, b) {
    (AgentPort(a, i), AgentPort(b, j)) => {
      let c = a.compare(b)
      if c == 0 {
        i.compare(j)
      } else {
        c
      }
    }
    (Free(a), Free(b)) => a.compare(b)
    (AgentPort(_, _), Free(_)) => -1
    (Free(_), AgentPort(_, _)) => 1
  }
}

///|
fn wire(a : Port, b : Port) -> (Port, Port) {
  if port_compare(a, b) <= 0 {
    (a, b)
  } else {
    (b, a)
  }
}

///|
fn normalize(net : Net) -> Net {
  let agents = net.agents.copy()
  agents.sort_by((a, b) => a.id.compare(b.id))
  let wires = net.wires.map(pair => wire(pair.0, pair.1))
  wires.sort_by((a, b) => {
    let c = port_compare(a.0, b.0)
    if c == 0 {
      port_compare(a.1, b.1)
    } else {
      c
    }
  })
  { ..net, agents, wires, }
}

///|
fn validate(net : Net) -> Net raise @utils.Fault {
  @utils.require(net.loops >= 0, "Negative cyclic wire count")
  let agents : Map[String, Kind] = Map([])
  for a in net.agents {
    ignore(@utils.identifier(a.id))
    @utils.require(!agents.contains(a.id), "Duplicate agent: " + a.id)
    agents[a.id] = a.kind
  }
  let used : Map[Port, Bool] = Map([])
  fn use_port(p : Port) -> Unit raise @utils.Fault {
    match p {
      Free(n) => ignore(@utils.identifier(n))
      AgentPort(n, i) => {
        let k = match agents.get(n) {
          Some(k) => k
          None => @utils.error("Unknown agent: " + n)
        }
        @utils.require(i >= 0 && i <= arity(k), "Invalid port: " + port_name(p))
      }
    }
    @utils.require(
      !used.contains(p),
      "Port used more than once: " + port_name(p),
    )
    used[p] = true
  }
  for pair in net.wires {
    use_port(pair.0)
    use_port(pair.1)
  }
  for a in net.agents {
    for i in 0..<(arity(a.kind) + 1) {
      @utils.require(
        used.contains(AgentPort(a.id, i)),
        "Unconnected port: " + port_name(AgentPort(a.id, i)),
      )
    }
  }
  normalize(net)
}

///|
fn parse(text : String) -> Net raise @utils.Fault {
  let p = @utils.create(text)
  let agents = []
  let wires = []
  let loops : Ref[Int32?] = { val: None, }
  fn port(p : @utils.Parser) -> Port raise @utils.Fault {
    if p.eat('@') {
      Free(p.ident())
    } else {
      let n = p.ident()
      p.expect('.')
      AgentPort(n, p.integer())
    }
  }
  while !p.eof() {
    match p.peek() {
      @utils.Keyword(_) => {
        let word = p.keyword()
        if word == "loops" {
          @utils.require(loops.val is None, "Duplicate \\loops declaration")
          loops.val = Some(p.integer())
        } else {
          let kind = match word {
            "gamma" => Gamma
            "delta" => Delta
            "epsilon" => Epsilon
            _ => p.error("Expected \\gamma, \\delta, \\epsilon or \\loops")
          }
          agents.push({ id: p.ident(), kind, })
        }
      }
      _ => {
        let a = port(p)
        p.expect('~')
        wires.push((a, port(p)))
      }
    }
    p.expect(';')
  }
  validate({ agents, wires, loops: loops.val.unwrap_or(0), })
}

///|
fn print(net : Net) -> String {
  let net = normalize(net)
  let lines = net.agents.map(a => "\\" + kind_name(a.kind) + " " + a.id + ";")
  lines.append(
    net.wires.map(p => port_name(p.0) + " ~ " + port_name(p.1) + ";"),
  )
  if net.loops != 0 {
    lines.push("\\loops " + net.loops.to_string() + ";")
  }
  @utils.join(lines, "\n")
}

///|
fn active_pairs(net : Net) -> Array[(String, String)] {
  let result = []
  for pair in normalize(net).wires {
    match pair {
      (AgentPort(a, 0), AgentPort(b, 0)) => result.push((a, b))
      _ => ()
    }
  }
  result
}

///|
fn splice(
  edges : Array[(Port, Port)],
  removed : (Port) -> Bool,
  initial_loops : Int32,
) -> (Array[(Port, Port)], Int32) raise @utils.Fault {
  let adjacency : Map[Port, Array[Port]] = Map([])
  let ports = []
  fn add(a : Port, b : Port) -> Unit {
    match adjacency.get(a) {
      Some(xs) => xs.push(b)
      None => {
        adjacency[a] = [b]
        ports.push(a)
      }
    }
  }
  for pair in edges {
    add(pair.0, pair.1)
    add(pair.1, pair.0)
  }
  ports.sort_by(port_compare)
  let seen : Map[Port, Bool] = Map([])
  let wires = []
  let mut loops = initial_loops
  for p in ports {
    if seen.contains(p) {
      continue
    }
    let todo = [p]
    let ends = []
    while !todo.is_empty() {
      let p = todo.pop().unwrap()
      if seen.contains(p) {
        continue
      }
      seen[p] = true
      if !removed(p) {
        ends.push(p)
      }
      todo.append(adjacency.get(p).unwrap())
    }
    if ends.length() == 2 {
      wires.push(wire(ends[0], ends[1]))
    } else if ends.is_empty() {
      @utils.require(loops < 2147483647, "Cyclic wire count overflow")
      loops += 1
    } else {
      @utils.error("Invalid rewrite wiring")
    }
  }
  (wires, loops)
}

///|
fn rewrite(
  net : Net,
  first : String,
  second : String,
) -> (Net, String) raise @utils.Fault {
  let (a, b) = if first <= second { (first, second) } else { (second, first) }
  @utils.require(
    active_pairs(net).contains((a, b)),
    "Selected agents are not an active pair",
  )
  fn kind(n : String) -> Kind {
    net.agents.iter().find_first(a => a.id == n).unwrap().kind
  }
  let ka = kind(a)
  let kb = kind(b)
  fn deleted(n : String) -> Bool {
    n == a || n == b
  }
  fn removed(p : Port) -> Bool {
    match p {
      AgentPort(n, _) => deleted(n)
      Free(_) => false
    }
  }
  let agents = net.agents.filter(a => !deleted(a.id))
  let used : Map[String, Bool] = Map([])
  for a in net.agents {
    used[a.id] = true
  }
  fn fresh(k : Kind) -> String {
    let mut i = 0
    while used.contains("n" + i.to_string()) {
      i += 1
    }
    let id = "n" + i.to_string()
    used[id] = true
    agents.push({ id, kind: k, })
    id
  }
  fn p(n : String, i : Int32) -> Port {
    AgentPort(n, i)
  }
  let replacement = match (ka, kb) {
    (Epsilon, Epsilon) => []
    (Gamma, Gamma) => [(p(a, 1), p(b, 2)), (p(a, 2), p(b, 1))]
    (Delta, Delta) => [(p(a, 1), p(b, 1)), (p(a, 2), p(b, 2))]
    (Epsilon, _) | (_, Epsilon) => {
      let node = if ka == Epsilon { b } else { a }
      Array::makei(2, i => (p(node, i + 1), p(fresh(Epsilon), 0)))
    }
    _ => {
      let (g, d) = if ka == Gamma { (a, b) } else { (b, a) }
      let d1 = fresh(Delta)
      let d2 = fresh(Delta)
      let g1 = fresh(Gamma)
      let g2 = fresh(Gamma)
      [
        (p(g, 1), p(d1, 0)),
        (p(g, 2), p(d2, 0)),
        (p(d, 1), p(g1, 0)),
        (p(d, 2), p(g2, 0)),
        (p(d1, 1), p(g1, 1)),
        (p(d1, 2), p(g2, 1)),
        (p(d2, 1), p(g1, 2)),
        (p(d2, 2), p(g2, 2)),
      ]
    }
  }
  let principal = wire(p(a, 0), p(b, 0))
  let edges = net.wires.filter(pair => wire(pair.0, pair.1) != principal)
  edges.append(replacement)
  let (wires, loops) = splice(edges, removed, net.loops)
  let (left, right) = if ordinal(ka) <= ordinal(kb) {
    (ka, kb)
  } else {
    (kb, ka)
  }
  (
    validate({ agents, wires, loops, }),
    kind_name(left) + "-" + kind_name(right),
  )
}

///|
fn make(code : Net, _input : Unit) -> State {
  { net: code, steps: 0, }
}

///|
fn step(
  s : State,
  input : Rinput,
) -> @definition.Transition[State, Routput, Foutput] raise @utils.Fault {
  let selected = input.pair
  let pairs = active_pairs(s.net)
  if pairs.is_empty() && selected is None {
    return @definition.Halt(s.net)
  }
  let pair = selected.unwrap_or_else(() => pairs[0])
  @utils.require(s.steps < 2147483647, "Step count overflow")
  let (net, output) = rewrite(s.net, pair.0, pair.1)
  @definition.Continue({ net, steps: s.steps + 1, }, output)
}

///|
fn kj(k : Kind) -> Json {
  @utils.tag(
    match k {
      Gamma => "Gamma"
      Delta => "Delta"
      Epsilon => "Epsilon"
    },
    [],
  )
}

///|
fn kd(j : Json) -> Kind raise @utils.Fault {
  match @utils.tag_name(j) {
    "Gamma" => Gamma
    "Delta" => Delta
    "Epsilon" => Epsilon
    _ => @utils.error("Invalid agent kind")
  }
}

///|
fn pj(p : Port) -> Json {
  match p {
    AgentPort(n, i) => @utils.tag("Agent", [@utils.jstr(n), @utils.jint(i)])
    Free(n) => @utils.tag("Free", [@utils.jstr(n)])
  }
}

///|
fn pd(j : Json) -> Port raise @utils.Fault {
  let n = @utils.identifier(@utils.string(@utils.tag_arg(j, 0)))
  match @utils.tag_name(j) {
    "Agent" => AgentPort(n, @utils.int(@utils.tag_arg(j, 1)))
    "Free" => Free(n)
    _ => @utils.error("Invalid net port")
  }
}

///|
fn nj(net : Net) -> Json {
  @utils.object([
    (
      "agents",
      @utils.jarray(
        net.agents.map(a => {
          @utils.object([("id", @utils.jstr(a.id)), ("kind", kj(a.kind))])
        }),
      ),
    ),
    (
      "wires",
      @utils.jarray(
        net.wires.map(pair => @utils.jarray([pj(pair.0), pj(pair.1)])),
      ),
    ),
    ("loops", @utils.jint(net.loops)),
  ])
}

///|
fn nd(j : Json) -> Net raise @utils.Fault {
  validate({
    agents: @utils.array(@utils.field(j, "agents")).map(j => {
      id: @utils.identifier(@utils.string(@utils.field(j, "id"))),
      kind: kd(@utils.field(j, "kind")),
    }),
    wires: @utils.array(@utils.field(j, "wires")).map(j => {
      let a = @utils.array(j)
      @utils.require(a.length() == 2, "Invalid wire")
      (pd(a[0]), pd(a[1]))
    }),
    loops: @utils.int(@utils.field(j, "loops")),
  })
}

///|
fn encode(s : State) -> Json {
  @utils.object([("net", nj(s.net)), ("steps", @utils.jint(s.steps))])
}

///|
fn decode(j : Json) -> State raise @utils.Fault {
  let steps = @utils.int(@utils.field(j, "steps"))
  @utils.require(steps >= 0, "Negative step count")
  { net: nd(@utils.field(j, "net")), steps, }
}

///|
fn render(s : State) -> Array[@definition.Block] {
  let active = active_pairs(s.net)
  let next = if active.is_empty() { None } else { Some(active[0]) }
  fn endpoint(p : Port) -> @definition.GraphEndpoint {
    match p {
      AgentPort(n, i) => { node: "agent:" + n, port: i, }
      Free(n) => { node: "free:" + n, port: 0, }
    }
  }
  let free : Array[String] = []
  for pair in s.net.wires {
    for p in [pair.0, pair.1] {
      match p {
        Free(n) if !free.contains(n) => free.push(n)
        _ => ()
      }
    }
  }
  free.sort()
  let nodes : Array[@definition.GraphNode] = s.net.agents.map(a => {
    id: "agent:" + a.id,
    label: (match a.kind {
      Gamma => "γ"
      Delta => "δ"
      Epsilon => "ε"
    }) +
    " " +
    a.id,
    ports: Array::makei(arity(a.kind) + 1, i => i.to_string()),
  })
  nodes.append(free.map(n => { id: "free:" + n, label: "@" + n, ports: [""], }))
  let edges : Array[@definition.GraphEdge] = s.net.wires.map(pair => {
    source: endpoint(pair.0),
    target: endpoint(pair.1),
    highlighted: match (pair.0, pair.1, next) {
      (AgentPort(a, 0), AgentPort(b, 0), Some((x, y))) => a == x && b == y
      _ => false
    },
  })
  [
    @definition.titled("steps", s.steps.to_string()),
    @definition.titled(
      "next interaction",
      match next {
        None => "No active pair"
        Some((a, b)) => a + " ↔ " + b
      },
    ),
    @definition.graph("net (principal port = 0)", nodes, edges),
    @definition.text_table(
      "active pairs",
      ["agent", "agent"],
      active.map(pair => [pair.0, pair.1]),
    ),
    @definition.text_table(
      "wires",
      ["port", "port"],
      s.net.wires.map(pair => [port_name(pair.0), port_name(pair.1)]),
    ),
    @definition.titled("cyclic wires without ports", s.net.loops.to_string()),
  ]
}

///|
pub fn model() -> @definition.Model[
  Code,
  Ainput,
  Rinput,
  Routput,
  Foutput,
  State,
] {
  @definition.Model(make=(code, input) => make(code, input), step=(state, input) => {
    step(state, input)
  })
}
