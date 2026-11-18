///|
fn render(c : Circuit) -> Json {
  let (verts, input_maps, output_maps) = match c {
    Gate(_) => {
      let owner = External(kind(c))
      (
        [(owner, c)],
        pins_of(c, true).map(p => (p, p)),
        pins_of(c, false).map(p => (p, p)),
      )
    }
    Mix(m) =>
      (
        m.verts.map(entry => (Internal(entry.0), entry.1)),
        mix_pins(m, true).map(p => {
          (map_internal(m.kind, m.inpin_maps, p), (Internal(p.0), p.1))
        }),
        mix_pins(m, false).map(p => {
          (map_internal(m.kind, m.otpin_maps, p), (Internal(p.0), p.1))
        }),
      )
    Iter(i) =>
      (
        i.used.mapi((n, child) => (Instance(n), child)),
        iter_pins(i, true).map(p => (p, p)),
        iter_pins(i, false).map(p => (p, p)),
      )
  }
  let names = @names.supply()
  for entry in verts {
    match entry.0 {
      Internal(n) | External(n) => names.reserve(n)
      _ => ()
    }
  }
  let instances : Map[Int32, String] = Map([])
  fn display(o : Owner) -> String {
    match o {
      Instance(id) =>
        match instances.get(id) {
          Some(name) => name
          None => {
            let name = names.fresh("instance")
            instances[id] = name
            name
          }
        }
      _ => owner_text(o)
    }
  }
  let inputs = input_maps.map(entry => [display(entry.0.0), entry.0.1])
  let output_rows = output_maps.map(entry => {
    let value = match get(verts, entry.1.0) {
      Some(child) => {
        let mut value = "?"
        for output in outputs(child) {
          if output.0.1 == entry.1.1 {
            value = bool_text(output.1)
            break
          }
        }
        value
      }
      None => "?"
    }
    [display(entry.0.0), entry.0.1, value]
  })
  let vertices = []
  for entry in verts {
    for output in outputs(entry.1) {
      vertices.push([display(entry.0), output.0.1, bool_text(output.1)])
    }
  }
  @base.jarray([
    @base.titled("kind", kind(c)),
    @base.table("inputs", ["name", "pin"], inputs),
    @base.table("outputs", ["name", "pin", "value"], output_rows),
    @base.table("verts", ["vert", "pin", "value"], vertices),
  ])
}

///|
fn make(code : String, input : String) -> Circuit raise @base.Fault {
  @base.unit_input(input)
  parse_code(code)
}

///|
fn step(
  c : Circuit,
  input : String,
) -> @runtime.Transition[Circuit] raise @base.Fault {
  let next = tick(c, read_signal(input))
  @runtime.Continue(next, signal_text(outputs(next)))
}

///|
pub fn model() -> @runtime.Model {
  @runtime.model(make, step, json_circuit, decode, render)
}
