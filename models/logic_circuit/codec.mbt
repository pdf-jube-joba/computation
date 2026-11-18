///|
fn json_gatekind(v : GateKind) -> Json {
  match v {
    Cst => @base.tag("Cst", [])
    Not => @base.tag("Not", [])
    And => @base.tag("And", [])
    Or => @base.tag("Or", [])
    Br => @base.tag("Br", [])
    Delay => @base.tag("Delay", [])
    End => @base.tag("End", [])
  }
}

///|
fn read_gatekind(j : Json) -> GateKind raise @base.Fault {
  let tag = @base.tag_name(j)
  let a = @base.array(j)[1:].to_owned()
  match tag {
    "Cst" => {
      @base.require(a.length() == 0, "Invalid Cst")
      Cst
    }
    "Not" => {
      @base.require(a.length() == 0, "Invalid Not")
      Not
    }
    "And" => {
      @base.require(a.length() == 0, "Invalid And")
      And
    }
    "Or" => {
      @base.require(a.length() == 0, "Invalid Or")
      Or
    }
    "Br" => {
      @base.require(a.length() == 0, "Invalid Br")
      Br
    }
    "Delay" => {
      @base.require(a.length() == 0, "Invalid Delay")
      Delay
    }
    "End" => {
      @base.require(a.length() == 0, "Invalid End")
      End
    }
    _ => @base.error("Invalid GateKind tag: " + tag)
  }
}

///|
fn json_circuit(v : Circuit) -> Json {
  match v {
    Gate(v0) => @base.tag("Gate", [json_gate(v0)])
    Mix(v0) => @base.tag("Mix", [json_mix(v0)])
    Iter(v0) => @base.tag("Iter", [json_iter(v0)])
  }
}

///|
fn read_circuit(j : Json) -> Circuit raise @base.Fault {
  let tag = @base.tag_name(j)
  let a = @base.array(j)[1:].to_owned()
  match tag {
    "Gate" => {
      @base.require(a.length() == 1, "Invalid Gate")
      Gate(read_gate(a[0]))
    }
    "Mix" => {
      @base.require(a.length() == 1, "Invalid Mix")
      Mix(read_mix(a[0]))
    }
    "Iter" => {
      @base.require(a.length() == 1, "Invalid Iter")
      Iter(read_iter(a[0]))
    }
    _ => @base.error("Invalid Circuit tag: " + tag)
  }
}

///|
fn json_gate(v : Gate) -> Json {
  @base.object([
    ("gate_kind", json_gatekind(v.gate_kind)),
    ("state", @base.jbool(v.state)),
  ])
}

///|
fn read_gate(j : Json) -> Gate raise @base.Fault {
  {
    gate_kind: read_gatekind(@base.field(j, "gate_kind")),
    state: @base.boolean(@base.field(j, "state")),
  }
}

///|
fn json_mix(v : Mix) -> Json {
  @base.object([
    ("kind", @base.jstr(v.kind)),
    (
      "verts",
      @base.jarray(
        v.verts.map(x => @base.jarray([@base.jstr(x.0), json_circuit(x.1)])),
      ),
    ),
    (
      "edges",
      @base.jarray(
        v.edges.map(x => {
          @base.jarray([
            @base.jarray([@base.jstr(x.0.0), @base.jstr(x.0.1)]),
            @base.jarray([@base.jstr(x.1.0), @base.jstr(x.1.1)]),
          ])
        }),
      ),
    ),
    (
      "inpin_maps",
      @base.jarray(
        v.inpin_maps.map(x => {
          @base.jarray([
            @base.jstr(x.0),
            @base.jarray([@base.jstr(x.1.0), @base.jstr(x.1.1)]),
          ])
        }),
      ),
    ),
    (
      "otpin_maps",
      @base.jarray(
        v.otpin_maps.map(x => {
          @base.jarray([
            @base.jstr(x.0),
            @base.jarray([@base.jstr(x.1.0), @base.jstr(x.1.1)]),
          ])
        }),
      ),
    ),
  ])
}

///|
fn read_mix(j : Json) -> Mix raise @base.Fault {
  {
    kind: @base.string(@base.field(j, "kind")),
    verts: @base.array(@base.field(j, "verts")).map(x => {
      let items = @base.array(x)
      @base.require(items.length() == 2, "Invalid tuple")
      (@base.string(items[0]), read_circuit(items[1]))
    }),
    edges: @base.array(@base.field(j, "edges")).map(x => {
      let items = @base.array(x)
      @base.require(items.length() == 2, "Invalid tuple")
      (
        {
          let items = @base.array(items[0])
          @base.require(items.length() == 2, "Invalid tuple")
          (@base.string(items[0]), @base.string(items[1]))
        },
        {
          let items = @base.array(items[1])
          @base.require(items.length() == 2, "Invalid tuple")
          (@base.string(items[0]), @base.string(items[1]))
        },
      )
    }),
    inpin_maps: @base.array(@base.field(j, "inpin_maps")).map(x => {
      let items = @base.array(x)
      @base.require(items.length() == 2, "Invalid tuple")
      (
        @base.string(items[0]),
        {
          let items = @base.array(items[1])
          @base.require(items.length() == 2, "Invalid tuple")
          (@base.string(items[0]), @base.string(items[1]))
        },
      )
    }),
    otpin_maps: @base.array(@base.field(j, "otpin_maps")).map(x => {
      let items = @base.array(x)
      @base.require(items.length() == 2, "Invalid tuple")
      (
        @base.string(items[0]),
        {
          let items = @base.array(items[1])
          @base.require(items.length() == 2, "Invalid tuple")
          (@base.string(items[0]), @base.string(items[1]))
        },
      )
    }),
  }
}

///|
fn json_iter(v : Iter) -> Json {
  @base.object([
    ("kind", @base.jstr(v.kind)),
    ("init", json_circuit(v.init)),
    ("used", @base.jarray(v.used.map(x => json_circuit(x)))),
    (
      "next_edges",
      @base.jarray(
        v.next_edges.map(x => @base.jarray([@base.jstr(x.0), @base.jstr(x.1)])),
      ),
    ),
    (
      "prev_edges",
      @base.jarray(
        v.prev_edges.map(x => @base.jarray([@base.jstr(x.0), @base.jstr(x.1)])),
      ),
    ),
    (
      "inpin_maps",
      @base.jarray(
        v.inpin_maps.map(x => {
          @base.jarray([
            @base.jstr(x.0),
            @base.jarray([@base.jstr(x.1.0), @base.jstr(x.1.1)]),
          ])
        }),
      ),
    ),
    (
      "otpin_maps",
      @base.jarray(
        v.otpin_maps.map(x => {
          @base.jarray([
            @base.jstr(x.0),
            @base.jarray([@base.jstr(x.1.0), @base.jstr(x.1.1)]),
          ])
        }),
      ),
    ),
  ])
}

///|
fn read_iter(j : Json) -> Iter raise @base.Fault {
  {
    kind: @base.string(@base.field(j, "kind")),
    init: read_circuit(@base.field(j, "init")),
    used: @base.array(@base.field(j, "used")).map(x => read_circuit(x)),
    next_edges: @base.array(@base.field(j, "next_edges")).map(x => {
      let items = @base.array(x)
      @base.require(items.length() == 2, "Invalid tuple")
      (@base.string(items[0]), @base.string(items[1]))
    }),
    prev_edges: @base.array(@base.field(j, "prev_edges")).map(x => {
      let items = @base.array(x)
      @base.require(items.length() == 2, "Invalid tuple")
      (@base.string(items[0]), @base.string(items[1]))
    }),
    inpin_maps: @base.array(@base.field(j, "inpin_maps")).map(x => {
      let items = @base.array(x)
      @base.require(items.length() == 2, "Invalid tuple")
      (
        @base.string(items[0]),
        {
          let items = @base.array(items[1])
          @base.require(items.length() == 2, "Invalid tuple")
          (@base.string(items[0]), @base.string(items[1]))
        },
      )
    }),
    otpin_maps: @base.array(@base.field(j, "otpin_maps")).map(x => {
      let items = @base.array(x)
      @base.require(items.length() == 2, "Invalid tuple")
      (
        @base.string(items[0]),
        {
          let items = @base.array(items[1])
          @base.require(items.length() == 2, "Invalid tuple")
          (@base.string(items[0]), @base.string(items[1]))
        },
      )
    }),
  }
}

///|
fn decode(j : Json) -> Circuit raise @base.Fault {
  let c = read_circuit(j)
  validate(c)
  c
}
