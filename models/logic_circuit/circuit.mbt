///|
#derive(Eq)
priv enum Owner {
  External(String)
  Internal(String)
  Instance(Int32)
}

///|
type Pin = (Owner, String)

///|
type Endpoint = (String, String)

///|
type Signal = Array[(Pin, Bool)]

///|
priv enum GateKind {
  Cst
  Not
  And
  Or
  Br
  Delay
  End
}

///|
priv struct Gate {
  gate_kind : GateKind
  state : Bool
}

///|
priv enum Circuit {
  Gate(Gate)
  Mix(Mix)
  Iter(Iter)
}

///|
priv struct Mix {
  kind : String
  verts : Array[(String, Circuit)]
  edges : Array[(Endpoint, Endpoint)]
  inpin_maps : Array[(String, Endpoint)]
  otpin_maps : Array[(String, Endpoint)]
}

///|
priv struct Iter {
  kind : String
  init : Circuit
  used : Array[Circuit]
  next_edges : Array[(String, String)]
  prev_edges : Array[(String, String)]
  inpin_maps : Array[(String, Endpoint)]
  otpin_maps : Array[(String, Endpoint)]
}

///|
fn[K : Eq, V] get(entries : Array[(K, V)], key : K) -> V? {
  for entry in entries {
    if entry.0 == key {
      return Some(entry.1)
    }
  }
  None
}

///|
fn owner_text(o : Owner) -> String {
  match o {
    External(n) => "@" + n
    Internal(n) => n
    Instance(i) => "$" + i.to_string()
  }
}

///|
fn gate_name(k : GateKind) -> String {
  match k {
    Cst => "CST"
    Not => "NOT"
    And => "AND"
    Or => "OR"
    Br => "BR"
    Delay => "DLY"
    End => "END"
  }
}

///|
fn kind(c : Circuit) -> String {
  match c {
    Gate(g) => gate_name(g.gate_kind)
    Mix(m) => m.kind
    Iter(i) => i.kind
  }
}

///|
fn map_internal(
  kind : String,
  maps : Array[(String, Endpoint)],
  endpoint : Endpoint,
) -> Pin {
  for entry in maps {
    if entry.1 == endpoint {
      return (External(kind), entry.0)
    }
  }
  (Internal(endpoint.0), endpoint.1)
}

///|
fn mix_pins(m : Mix, input : Bool) -> Array[Endpoint] {
  let pins = []
  for entry in m.verts {
    for pin in pins_of(entry.1, input) {
      let endpoint = (entry.0, pin.1)
      let connected = if input {
        m.edges.any(e => e.1 == endpoint)
      } else {
        m.edges.any(e => e.0 == endpoint)
      }
      if !connected {
        pins.push(endpoint)
      }
    }
  }
  pins
}

///|
fn iter_pins(i : Iter, input : Bool) -> Array[Pin] {
  let pins = []
  for n, child in i.used {
    for pin in pins_of(child, input) {
      let p = pin.1
      let available = if input {
        (n == 0 || !i.next_edges.any(e => e.1 == p)) &&
        !i.prev_edges.any(e => e.1 == p)
      } else {
        (n == 0 || get(i.prev_edges, p) is None) && get(i.next_edges, p) is None
      }
      if available {
        pins.push((Instance(n), p))
      }
    }
  }
  pins
}

///|
fn pins_of(c : Circuit, input : Bool) -> Array[Pin] {
  match c {
    Gate(g) => {
      let pins = if input {
        match g.gate_kind {
          Cst => []
          And | Or => ["IN0", "IN1"]
          _ => ["IN"]
        }
      } else {
        match g.gate_kind {
          End => []
          Br => ["OUT0", "OUT1"]
          _ => ["OUT"]
        }
      }
      pins.map(p => (External(gate_name(g.gate_kind)), p))
    }
    Mix(m) =>
      mix_pins(m, input).map(p => {
        map_internal(m.kind, if input { m.inpin_maps } else { m.otpin_maps }, p)
      })
    Iter(i) =>
      if input && i.used.is_empty() {
        pins_of(i.init, true).map(p => (External(i.kind), p.1))
      } else {
        iter_pins(i, input)
      }
  }
}

///|
fn outputs(c : Circuit) -> Signal {
  match c {
    Gate(g) => pins_of(c, false).map(p => (p, g.state))
    Mix(m) => {
      let result = []
      for entry in m.verts {
        for value in outputs(entry.1) {
          let endpoint = (entry.0, value.0.1)
          if get(m.edges, endpoint) is None {
            result.push((map_internal(m.kind, m.otpin_maps, endpoint), value.1))
          }
        }
      }
      result
    }
    Iter(i) => {
      let result = []
      for n, child in i.used {
        for entry in outputs(child) {
          let pin = entry.0.1
          if (n == 0 || get(i.prev_edges, pin) is None) &&
            get(i.next_edges, pin) is None {
            result.push(((Instance(n), pin), entry.1))
          }
        }
      }
      result
    }
  }
}

///|
fn check_mix(m : Mix) -> Unit raise @base.Fault {
  let names : Map[String, Bool] = Map([])
  let inputs = []
  let outputs = []
  for entry in m.verts {
    @base.require(!names.contains(entry.0), "duplicate vertex name: " + entry.0)
    names[entry.0] = true
    for pin in pins_of(entry.1, true) {
      inputs.push((entry.0, pin.1))
    }
    for pin in pins_of(entry.1, false) {
      outputs.push((entry.0, pin.1))
    }
  }
  let mut available_in = inputs
  let mut available_out = outputs
  for edge in m.edges {
    @base.require(
      available_out.contains(edge.0) && available_in.contains(edge.1),
      "edge endpoint not in vertices",
    )
    available_out = available_out.filter(p => p != edge.0)
    available_in = available_in.filter(p => p != edge.1)
  }
  for entry in m.inpin_maps {
    @base.require(
      inputs.contains(entry.1),
      "input map endpoint not in vertices",
    )
  }
  for entry in m.otpin_maps {
    @base.require(
      outputs.contains(entry.1),
      "output map endpoint not in vertices",
    )
  }
}

///|
fn validate(c : Circuit) -> Unit raise @base.Fault {
  match c {
    Gate(_) => ()
    Mix(m) => {
      for entry in m.verts {
        validate(entry.1)
      }
      check_mix(m)
    }
    Iter(i) => {
      validate(i.init)
      for child in i.used {
        validate(child)
      }
    }
  }
}

///|
/// The first delivered signal wins, including when it is false.
fn add_input(
  inputs : Array[(String, Bool)],
  pin : String,
  value : Bool,
) -> Unit {
  if get(inputs, pin) is None {
    inputs.push((pin, value))
  }
}

///|
fn deliver(child : Circuit, inputs : Array[(String, Bool)]) -> Signal {
  let result = []
  let pins = pins_of(child, true)
  for entry in inputs {
    for pin in pins {
      if pin.1 == entry.0 {
        result.push((pin, entry.1))
        break
      }
    }
  }
  result
}

///|
fn input_value(inputs : Signal, pin : String) -> Bool {
  for entry in inputs {
    if entry.0.1 == pin {
      return entry.1
    }
  }
  false
}

///|
/// Every child sees outputs from the previous tick.
fn tick(c : Circuit, inputs : Signal) -> Circuit raise @base.Fault {
  match c {
    Gate(g) => {
      let state = match g.gate_kind {
        Cst => g.state
        Not => !input_value(inputs, "IN")
        And => input_value(inputs, "IN0") && input_value(inputs, "IN1")
        Or => input_value(inputs, "IN0") || input_value(inputs, "IN1")
        Br | Delay | End => input_value(inputs, "IN")
      }
      Gate({ ..g, state, })
    }
    Mix(m) => {
      check_mix(m)
      let pending : Array[Array[(String, Bool)]] = m.verts.map(_ => [])
      fn send(name : String, pin : String, value : Bool) -> Unit {
        for id, entry in m.verts {
          if entry.0 == name {
            add_input(pending[id], pin, value)
            return
          }
        }
      }
      for entry in m.verts {
        for value in outputs(entry.1) {
          if get(m.edges, (entry.0, value.0.1)) is Some((name, pin)) {
            send(name, pin, value.1)
          }
        }
      }
      for entry in inputs {
        let target = match entry.0.0 {
          External(name) if name == m.kind => get(m.inpin_maps, entry.0.1)
          Internal(name) => Some((name, entry.0.1))
          _ => None
        }
        if target is Some((name, pin)) {
          send(name, pin, entry.1)
        }
      }
      Mix({
        ..m,
        verts: m.verts.mapi((id, entry) => {
          (entry.0, tick(entry.1, deliver(entry.1, pending[id])))
        }),
      })
    }
    Iter(i) => {
      let count = i.used.length()
      let pending : Array[Array[(String, Bool)]] = Array::makei(count + 1, _ => {
        []
      })
      for n, child in i.used {
        for entry in outputs(child) {
          if get(i.next_edges, entry.0.1) is Some(pin) {
            add_input(pending[n + 1], pin, entry.1)
          }
          if get(i.prev_edges, entry.0.1) is Some(pin) {
            @base.require(
              n > 0,
              "Iterator predecessor of index zero is out of bounds",
            )
            add_input(pending[n - 1], pin, entry.1)
          }
        }
      }
      for entry in inputs {
        let id = match entry.0.0 {
          Instance(n) => Some(n)
          External(name) if name == i.kind => Some(count)
          _ => None
        }
        if id is Some(n) {
          @base.require(
            n >= 0 && n <= count,
            "Iterator input ID is out of bounds",
          )
          add_input(pending[n], entry.0.1, entry.1)
        }
      }
      let used = i.used.mapi((n, child) => {
        tick(child, deliver(child, pending[n]))
      })
      if pending[count].any(entry => entry.1) {
        used.push(tick(i.init, deliver(i.init, pending[count])))
      }
      Iter({ ..i, used, })
    }
  }
}
