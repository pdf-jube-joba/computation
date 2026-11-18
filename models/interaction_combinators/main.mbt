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
priv struct Net {
  agents : Array[Agent]
  wires : Array[(Port, Port)]
  loops : Int32
}

///|
priv struct State {
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
fn validate(net : Net) -> Net raise @base.Fault {
  @base.require(net.loops >= 0, "Negative cyclic wire count")
  let agents : Map[String, Kind] = Map([])
  for a in net.agents {
    ignore(@base.identifier(a.id))
    @base.require(!agents.contains(a.id), "Duplicate agent: " + a.id)
    agents[a.id] = a.kind
  }
  let used : Map[Port, Bool] = Map([])
  fn use_port(p : Port) -> Unit raise @base.Fault {
    match p {
      Free(n) => ignore(@base.identifier(n))
      AgentPort(n, i) => {
        let k = match agents.get(n) {
          Some(k) => k
          None => @base.error("Unknown agent: " + n)
        }
        @base.require(i >= 0 && i <= arity(k), "Invalid port: " + port_name(p))
      }
    }
    @base.require(
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
      @base.require(
        used.contains(AgentPort(a.id, i)),
        "Unconnected port: " + port_name(AgentPort(a.id, i)),
      )
    }
  }
  normalize(net)
}

///|
fn parse(text : String) -> Net raise @base.Fault {
  let p = @p.create(text)
  let agents = []
  let wires = []
  let loops : Ref[Int32?] = { val: None, }
  fn port(p : @p.Parser) -> Port raise @base.Fault {
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
      @p.Keyword(_) => {
        let word = p.keyword()
        if word == "loops" {
          @base.require(loops.val is None, "Duplicate \\loops declaration")
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
  @base.join(lines, "\n")
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
) -> (Array[(Port, Port)], Int32) raise @base.Fault {
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
      @base.require(loops < 2147483647, "Cyclic wire count overflow")
      loops += 1
    } else {
      @base.error("Invalid rewrite wiring")
    }
  }
  (wires, loops)
}

///|
fn rewrite(
  net : Net,
  first : String,
  second : String,
) -> (Net, String) raise @base.Fault {
  let (a, b) = if first <= second { (first, second) } else { (second, first) }
  @base.require(
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
fn make(code : String, input : String) -> State raise @base.Fault {
  @base.unit_input(input)
  { net: parse(code), steps: 0, }
}

///|
fn step(
  s : State,
  input : String,
) -> @runtime.Transition[State] raise @base.Fault {
  let selected = if input.trim().is_empty() {
    None
  } else {
    let p = @p.create(input)
    let a = p.ident()
    let b = p.ident()
    p.finish()
    Some((a, b))
  }
  let pairs = active_pairs(s.net)
  if pairs.is_empty() && selected is None {
    return @runtime.Halt(print(s.net))
  }
  let pair = selected.unwrap_or_else(() => pairs[0])
  @base.require(s.steps < 2147483647, "Step count overflow")
  let (net, output) = rewrite(s.net, pair.0, pair.1)
  @runtime.Continue({ net, steps: s.steps + 1, }, output)
}

///|
fn kj(k : Kind) -> Json {
  @base.tag(
    match k {
      Gamma => "Gamma"
      Delta => "Delta"
      Epsilon => "Epsilon"
    },
    [],
  )
}

///|
fn kd(j : Json) -> Kind raise @base.Fault {
  match @base.tag_name(j) {
    "Gamma" => Gamma
    "Delta" => Delta
    "Epsilon" => Epsilon
    _ => @base.error("Invalid agent kind")
  }
}

///|
fn pj(p : Port) -> Json {
  match p {
    AgentPort(n, i) => @base.tag("Agent", [@base.jstr(n), @base.jint(i)])
    Free(n) => @base.tag("Free", [@base.jstr(n)])
  }
}

///|
fn pd(j : Json) -> Port raise @base.Fault {
  let n = @base.identifier(@base.string(@base.tag_arg(j, 0)))
  match @base.tag_name(j) {
    "Agent" => AgentPort(n, @base.int(@base.tag_arg(j, 1)))
    "Free" => Free(n)
    _ => @base.error("Invalid net port")
  }
}

///|
fn nj(net : Net) -> Json {
  @base.object([
    (
      "agents",
      @base.jarray(
        net.agents.map(a => {
          @base.object([("id", @base.jstr(a.id)), ("kind", kj(a.kind))])
        }),
      ),
    ),
    (
      "wires",
      @base.jarray(
        net.wires.map(pair => @base.jarray([pj(pair.0), pj(pair.1)])),
      ),
    ),
    ("loops", @base.jint(net.loops)),
  ])
}

///|
fn nd(j : Json) -> Net raise @base.Fault {
  validate({
    agents: @base.array(@base.field(j, "agents")).map(j => {
      id: @base.identifier(@base.string(@base.field(j, "id"))),
      kind: kd(@base.field(j, "kind")),
    }),
    wires: @base.array(@base.field(j, "wires")).map(j => {
      let a = @base.array(j)
      @base.require(a.length() == 2, "Invalid wire")
      (pd(a[0]), pd(a[1]))
    }),
    loops: @base.int(@base.field(j, "loops")),
  })
}

///|
fn encode(s : State) -> Json {
  @base.object([("net", nj(s.net)), ("steps", @base.jint(s.steps))])
}

///|
fn decode(j : Json) -> State raise @base.Fault {
  let steps = @base.int(@base.field(j, "steps"))
  @base.require(steps >= 0, "Negative step count")
  { net: nd(@base.field(j, "net")), steps, }
}

///|
fn render(s : State) -> Json {
  let active = active_pairs(s.net)
  let next = if active.is_empty() { None } else { Some(active[0]) }
  fn endpoint(p : Port) -> Json {
    match p {
      AgentPort(n, i) =>
        @base.object([
          ("node", @base.jstr("agent:" + n)),
          ("port", @base.jint(i)),
        ])
      Free(n) =>
        @base.object([
          ("node", @base.jstr("free:" + n)),
          ("port", @base.jint(0)),
        ])
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
  let nodes = s.net.agents.map(a => {
    @base.object([
      ("id", @base.jstr("agent:" + a.id)),
      (
        "label",
        @base.jstr(
          (match a.kind {
            Gamma => "γ"
            Delta => "δ"
            Epsilon => "ε"
          }) +
          " " +
          a.id,
        ),
      ),
      (
        "ports",
        @base.jarray(
          Array::makei(arity(a.kind) + 1, i => @base.jstr(i.to_string())),
        ),
      ),
    ])
  })
  nodes.append(
    free.map(n => {
      @base.object([
        ("id", @base.jstr("free:" + n)),
        ("label", @base.jstr("@" + n)),
        ("ports", @base.jarray([@base.jstr("")])),
      ])
    }),
  )
  let edges = s.net.wires.map(pair => {
    @base.object([
      ("source", endpoint(pair.0)),
      ("target", endpoint(pair.1)),
      (
        "highlighted",
        @base.jbool(
          match (pair.0, pair.1, next) {
            (AgentPort(a, 0), AgentPort(b, 0), Some((x, y))) => a == x && b == y
            _ => false
          },
        ),
      ),
    ])
  })
  @base.jarray([
    @base.titled("steps", s.steps.to_string()),
    @base.titled(
      "next interaction",
      match next {
        None => "No active pair"
        Some((a, b)) => a + " ↔ " + b
      },
    ),
    @base.object([
      ("kind", @base.jstr("graph")),
      ("title", @base.jstr("net (principal port = 0)")),
      ("nodes", @base.jarray(nodes)),
      ("edges", @base.jarray(edges)),
    ]),
    @base.table(
      "active pairs",
      ["agent", "agent"],
      active.map(pair => [pair.0, pair.1]),
    ),
    @base.table(
      "wires",
      ["port", "port"],
      s.net.wires.map(pair => [port_name(pair.0), port_name(pair.1)]),
    ),
    @base.titled("cyclic wires without ports", s.net.loops.to_string()),
  ])
}

///|
pub fn model() -> @runtime.Model {
  @runtime.model(make, step, encode, decode, render)
}

///|
test "annihilation preserves free wires and counts closed cyclic wires" {
  for
    case in [
      ("gamma", "a.1 ~ b.2; a.2 ~ b.1;", 2),
      ("gamma", "a.1 ~ b.1; a.2 ~ b.2;", 1),
      ("gamma", "a.1 ~ a.2; b.1 ~ b.2;", 1),
      ("delta", "a.1 ~ b.1; a.2 ~ b.2;", 2),
      ("delta", "a.1 ~ b.2; a.2 ~ b.1;", 1),
    ] {
    let m = model()
    m.make("\\" + case.0 + " a; \\" + case.0 + " b; a.0 ~ b.0; " + case.1, "")
    ignore(m.step(""))
    assert_eq(
      @base.string(@base.field(@base.parse_json(m.step("")), "foutput")),
      "\\loops " + case.2.to_string() + ";",
    )
  }
  let m = model()
  m.make(
    "\\gamma a; \\gamma b; a.0 ~ b.0; a.1 ~ @x; a.2 ~ @y; b.1 ~ @u; b.2 ~ @v;", "",
  )
  let saved = m.snapshot()
  assert_true(
    try {
      ignore(m.step("a missing"))
      false
    } catch {
      _ => true
    },
  )
  assert_eq(m.snapshot(), saved)
  ignore(m.step("b a"))
  assert_eq(
    @base.string(@base.field(@base.parse_json(m.step("")), "foutput")),
    "@u ~ @y;\n@v ~ @x;",
  )
}

///|
test "commutation and erasure leave valid independently allocated agents" {
  let m = model()
  m.make(
    "\\delta dup; \\gamma pair; \\epsilon l; \\epsilon r; dup.0 ~ pair.0; dup.1 ~ @left; dup.2 ~ @right; pair.1 ~ l.0; pair.2 ~ r.0;",
    "",
  )
  for _ in 0..<3 {
    let saved = m.snapshot()
    let result = m.step("")
    m.restore(saved)
    assert_eq(m.step(""), result)
  }
  let result = @base.parse_json(m.step(""))
  let net = parse(@base.string(@base.field(result, "foutput")))
  assert_eq(net.agents.length(), 6)
  assert_eq(net.agents.filter(a => a.kind == Gamma).length(), 2)
  assert_eq(net.agents.filter(a => a.kind == Epsilon).length(), 4)
}
