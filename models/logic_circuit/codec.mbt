///|
fn json_gatekind(v : GateKind) -> Json {
  match v {
    Cst => @utils.tag("Cst", [])
    Not => @utils.tag("Not", [])
    And => @utils.tag("And", [])
    Or => @utils.tag("Or", [])
    Br => @utils.tag("Br", [])
    Delay => @utils.tag("Delay", [])
    End => @utils.tag("End", [])
  }
}

///|
fn read_gatekind(j : Json) -> GateKind raise @utils.Fault {
  let tag = @utils.tag_name(j)
  let a = @utils.array(j)[1:].to_owned()
  match tag {
    "Cst" => {
      @utils.require(a.length() == 0, "Invalid Cst")
      Cst
    }
    "Not" => {
      @utils.require(a.length() == 0, "Invalid Not")
      Not
    }
    "And" => {
      @utils.require(a.length() == 0, "Invalid And")
      And
    }
    "Or" => {
      @utils.require(a.length() == 0, "Invalid Or")
      Or
    }
    "Br" => {
      @utils.require(a.length() == 0, "Invalid Br")
      Br
    }
    "Delay" => {
      @utils.require(a.length() == 0, "Invalid Delay")
      Delay
    }
    "End" => {
      @utils.require(a.length() == 0, "Invalid End")
      End
    }
    _ => @utils.error("Invalid GateKind tag: " + tag)
  }
}

///|
fn json_circuit(v : Circuit) -> Json {
  match v {
    Gate(v0) => @utils.tag("Gate", [json_gate(v0)])
    Mix(v0) => @utils.tag("Mix", [json_mix(v0)])
    Iter(v0) => @utils.tag("Iter", [json_iter(v0)])
  }
}

///|
fn read_circuit(j : Json) -> Circuit raise @utils.Fault {
  let tag = @utils.tag_name(j)
  let a = @utils.array(j)[1:].to_owned()
  match tag {
    "Gate" => {
      @utils.require(a.length() == 1, "Invalid Gate")
      Gate(read_gate(a[0]))
    }
    "Mix" => {
      @utils.require(a.length() == 1, "Invalid Mix")
      Mix(read_mix(a[0]))
    }
    "Iter" => {
      @utils.require(a.length() == 1, "Invalid Iter")
      Iter(read_iter(a[0]))
    }
    _ => @utils.error("Invalid Circuit tag: " + tag)
  }
}

///|
fn json_gate(v : Gate) -> Json {
  @utils.object([
    ("gate_kind", json_gatekind(v.gate_kind)),
    ("state", @utils.jbool(v.state)),
  ])
}

///|
fn read_gate(j : Json) -> Gate raise @utils.Fault {
  {
    gate_kind: read_gatekind(@utils.field(j, "gate_kind")),
    state: @utils.boolean(@utils.field(j, "state")),
  }
}

///|
fn json_mix(v : Mix) -> Json {
  @utils.object([
    ("kind", @utils.jstr(v.kind)),
    (
      "verts",
      @utils.jarray(
        v.verts.map(x => @utils.jarray([@utils.jstr(x.0), json_circuit(x.1)])),
      ),
    ),
    (
      "edges",
      @utils.jarray(
        v.edges.map(x => {
          @utils.jarray([
            @utils.jarray([@utils.jstr(x.0.0), @utils.jstr(x.0.1)]),
            @utils.jarray([@utils.jstr(x.1.0), @utils.jstr(x.1.1)]),
          ])
        }),
      ),
    ),
    (
      "inpin_maps",
      @utils.jarray(
        v.inpin_maps.map(x => {
          @utils.jarray([
            @utils.jstr(x.0),
            @utils.jarray([@utils.jstr(x.1.0), @utils.jstr(x.1.1)]),
          ])
        }),
      ),
    ),
    (
      "otpin_maps",
      @utils.jarray(
        v.otpin_maps.map(x => {
          @utils.jarray([
            @utils.jstr(x.0),
            @utils.jarray([@utils.jstr(x.1.0), @utils.jstr(x.1.1)]),
          ])
        }),
      ),
    ),
  ])
}

///|
fn read_mix(j : Json) -> Mix raise @utils.Fault {
  {
    kind: @utils.string(@utils.field(j, "kind")),
    verts: @utils.array(@utils.field(j, "verts")).map(x => {
      let items = @utils.array(x)
      @utils.require(items.length() == 2, "Invalid tuple")
      (@utils.string(items[0]), read_circuit(items[1]))
    }),
    edges: @utils.array(@utils.field(j, "edges")).map(x => {
      let items = @utils.array(x)
      @utils.require(items.length() == 2, "Invalid tuple")
      (
        {
          let items = @utils.array(items[0])
          @utils.require(items.length() == 2, "Invalid tuple")
          (@utils.string(items[0]), @utils.string(items[1]))
        },
        {
          let items = @utils.array(items[1])
          @utils.require(items.length() == 2, "Invalid tuple")
          (@utils.string(items[0]), @utils.string(items[1]))
        },
      )
    }),
    inpin_maps: @utils.array(@utils.field(j, "inpin_maps")).map(x => {
      let items = @utils.array(x)
      @utils.require(items.length() == 2, "Invalid tuple")
      (
        @utils.string(items[0]),
        {
          let items = @utils.array(items[1])
          @utils.require(items.length() == 2, "Invalid tuple")
          (@utils.string(items[0]), @utils.string(items[1]))
        },
      )
    }),
    otpin_maps: @utils.array(@utils.field(j, "otpin_maps")).map(x => {
      let items = @utils.array(x)
      @utils.require(items.length() == 2, "Invalid tuple")
      (
        @utils.string(items[0]),
        {
          let items = @utils.array(items[1])
          @utils.require(items.length() == 2, "Invalid tuple")
          (@utils.string(items[0]), @utils.string(items[1]))
        },
      )
    }),
  }
}

///|
fn json_iter(v : Iter) -> Json {
  @utils.object([
    ("kind", @utils.jstr(v.kind)),
    ("init", json_circuit(v.init)),
    ("used", @utils.jarray(v.used.map(x => json_circuit(x)))),
    (
      "next_edges",
      @utils.jarray(
        v.next_edges.map(x => {
          @utils.jarray([@utils.jstr(x.0), @utils.jstr(x.1)])
        }),
      ),
    ),
    (
      "prev_edges",
      @utils.jarray(
        v.prev_edges.map(x => {
          @utils.jarray([@utils.jstr(x.0), @utils.jstr(x.1)])
        }),
      ),
    ),
    (
      "inpin_maps",
      @utils.jarray(
        v.inpin_maps.map(x => {
          @utils.jarray([
            @utils.jstr(x.0),
            @utils.jarray([@utils.jstr(x.1.0), @utils.jstr(x.1.1)]),
          ])
        }),
      ),
    ),
    (
      "otpin_maps",
      @utils.jarray(
        v.otpin_maps.map(x => {
          @utils.jarray([
            @utils.jstr(x.0),
            @utils.jarray([@utils.jstr(x.1.0), @utils.jstr(x.1.1)]),
          ])
        }),
      ),
    ),
  ])
}

///|
fn read_iter(j : Json) -> Iter raise @utils.Fault {
  {
    kind: @utils.string(@utils.field(j, "kind")),
    init: read_circuit(@utils.field(j, "init")),
    used: @utils.array(@utils.field(j, "used")).map(x => read_circuit(x)),
    next_edges: @utils.array(@utils.field(j, "next_edges")).map(x => {
      let items = @utils.array(x)
      @utils.require(items.length() == 2, "Invalid tuple")
      (@utils.string(items[0]), @utils.string(items[1]))
    }),
    prev_edges: @utils.array(@utils.field(j, "prev_edges")).map(x => {
      let items = @utils.array(x)
      @utils.require(items.length() == 2, "Invalid tuple")
      (@utils.string(items[0]), @utils.string(items[1]))
    }),
    inpin_maps: @utils.array(@utils.field(j, "inpin_maps")).map(x => {
      let items = @utils.array(x)
      @utils.require(items.length() == 2, "Invalid tuple")
      (
        @utils.string(items[0]),
        {
          let items = @utils.array(items[1])
          @utils.require(items.length() == 2, "Invalid tuple")
          (@utils.string(items[0]), @utils.string(items[1]))
        },
      )
    }),
    otpin_maps: @utils.array(@utils.field(j, "otpin_maps")).map(x => {
      let items = @utils.array(x)
      @utils.require(items.length() == 2, "Invalid tuple")
      (
        @utils.string(items[0]),
        {
          let items = @utils.array(items[1])
          @utils.require(items.length() == 2, "Invalid tuple")
          (@utils.string(items[0]), @utils.string(items[1]))
        },
      )
    }),
  }
}

///|
fn decode(j : Json) -> Circuit raise @utils.Fault {
  let c = read_circuit(j)
  validate(c)
  c
}
