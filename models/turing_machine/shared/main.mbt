///|
pub(all) struct Entry {
  reads : Array[String]
  source : Int32
  writes : Array[String]
  target : Int32
  directions : Array[Int32]
}

///|
pub(all) struct Definition {
  count : Int32
  alphabet : Array[String]
  initial : Int32
  accepted : Array[Int32]
  code : Array[Entry]
  names : Array[(Int32, String)]
}

///|
priv struct State {
  definition : Definition
  current : Int32
  tapes : Array[@data.Tape[String]]
}

///|
pub fn sign(text : String) -> String raise @base.Fault {
  let text = text.trim().to_owned()
  if text == "-" {
    text
  } else {
    @base.identifier(text)
  }
}

///|
fn direction(text : String) -> Int32 raise @base.Fault {
  let p = @p.create(text)
  let d = p.keyword()
  p.finish()
  match d {
    "L" => -1
    "C" => 0
    "R" => 1
    _ => @base.error("Invalid direction")
  }
}

///|
pub fn direction_text(d : Int32) -> String {
  if d < 0 {
    "\\L"
  } else if d > 0 {
    "\\R"
  } else {
    "\\C"
  }
}

///|
fn comma(text : String) -> Array[String] {
  text.split(",").map(s => s.trim().to_owned()).collect()
}

///|
fn words(text : String) -> Array[String] {
  String::from_array(text.to_array().map(c => if c == '\t' { ' ' } else { c }))
  .split(" ")
  .filter(s => !s.is_empty())
  .map(s => s.to_owned())
  .collect()
}

///|
fn strip(line : String) -> String {
  line.split("#").head().unwrap_or("").trim().to_owned()
}

///|
pub fn validate(d : Definition, multi : Bool) -> Unit raise @base.Fault {
  @base.require(d.count > 0, "Tape count must be positive")
  if multi {
    @base.require(
      d.alphabet.contains("-"),
      "Alphabet must contain the blank symbol (-)",
    )
    let seen : Map[String, Bool] = Map([])
    for s in d.alphabet {
      ignore(sign(s))
      @base.require(!seen.contains(s), "Duplicate alphabet symbol")
      seen[s] = true
    }
  }
  let ids : Map[Int32, Bool] = Map([])
  let names : Map[String, Bool] = Map([])
  for pair in d.names {
    ignore(@base.identifier(pair.1))
    @base.require(
      pair.0 >= 0 && !ids.contains(pair.0),
      "Duplicate or negative state ID",
    )
    @base.require(!names.contains(pair.1), "Duplicate state name")
    ids[pair.0] = true
    names[pair.1] = true
  }
  @base.require(
    ids.contains(d.initial) && d.accepted.all(id => ids.contains(id)),
    "Unknown state ID",
  )
  let seen : Map[String, Bool] = Map([])
  for entry in d.code {
    @base.require(
      ids.contains(entry.source) && ids.contains(entry.target),
      "Unknown state ID",
    )
    @base.require(
      !d.accepted.contains(entry.source),
      "Code contains accepted state",
    )
    @base.require(
      entry.reads.length() == d.count &&
      entry.writes.length() == d.count &&
      entry.directions.length() == d.count,
      "Transition vector length differs from tape count",
    )
    for s in entry.reads {
      ignore(sign(s))
      if multi {
        @base.require(d.alphabet.contains(s), "Symbol outside alphabet: " + s)
      }
    }
    for s in entry.writes {
      ignore(sign(s))
      if multi {
        @base.require(d.alphabet.contains(s), "Symbol outside alphabet: " + s)
      }
    }
    let key = entry.source.to_string() + ":" + @base.join(entry.reads, ",")
    @base.require(
      !seen.contains(key),
      "Duplicate transition for state/read vector",
    )
    seen[key] = true
  }
}

///|
pub fn parse_code(text : String, multi : Bool) -> Definition raise @base.Fault {
  let lines = text.split("\n").map(s => s.to_owned()).collect()
  @base.require(
    lines.length() >= (if multi { 4 } else { 2 }),
    "Missing code header lines",
  )
  let lines = if multi { lines.map(strip) } else { lines }
  fn header(prefix : String, line : String) -> String raise @base.Fault {
    @base.require(line.has_prefix(prefix), "Expected " + prefix)
    line
    .unsafe_substring(start=prefix.length(), end=line.length())
    .trim()
    .to_owned()
  }
  let count = if multi { @base.integer(header("tapes:", lines[0])) } else { 1 }
  let alphabet = if multi {
    comma(header("alphabet:", lines[1])).map(sign)
  } else {
    []
  }
  let offset = if multi { 2 } else { 0 }
  let names : Array[(Int32, String)] = []
  fn state(name : String) -> Int32 raise @base.Fault {
    let name = @base.identifier(name.trim().to_owned())
    for pair in names {
      if pair.1 == name {
        return pair.0
      }
    }
    let id = names.length()
    names.push((id, name))
    id
  }
  let initial = state(lines[offset])
  let accepted = if lines[offset + 1].trim().is_empty() {
    []
  } else {
    comma(lines[offset + 1]).map(state)
  }
  let code = []
  for line in lines[offset + 2:] {
    let line = strip(line)
    if line.is_empty() {
      continue
    }
    let columns = comma(line)
    @base.require(
      if multi {
        columns.length() == 5
      } else {
        columns.length() >= 5
      },
      "Invalid transition: " + line,
    )
    let reads = if multi {
      words(columns[0]).map(sign)
    } else {
      [sign(columns[0])]
    }
    let source = state(columns[1])
    let writes = if multi {
      words(columns[2]).map(sign)
    } else {
      [sign(columns[2])]
    }
    let target = state(columns[3])
    let directions = if multi {
      words(columns[4]).map(direction)
    } else {
      [direction(columns[4])]
    }
    code.push({ reads, source, writes, target, directions, })
  }
  let d = { count, alphabet, initial, accepted, code, names, }
  validate(d, multi)
  d
}

///|
fn validate_state(s : State, multi : Bool) -> State raise @base.Fault {
  validate(s.definition, multi)
  @base.require(
    s.definition.names.any(pair => pair.0 == s.current),
    "Unknown current state ID",
  )
  @base.require(
    s.tapes.length() == s.definition.count,
    "Input tape count differs from code",
  )
  for tape in s.tapes {
    for value in tape.values() {
      ignore(sign(value))
      if multi {
        @base.require(
          s.definition.alphabet.contains(value),
          "Input symbol outside alphabet: " + value,
        )
      }
    }
  }
  s
}

///|
fn make(code : String, input : String, multi : Bool) -> State raise @base.Fault {
  let definition = parse_code(code, multi)
  let lines = if multi {
    input.trim().to_owned().split("\n").map(s => s.to_owned()).collect()
  } else {
    [input]
  }
  let tapes = lines.map(text => @data.parse_tape(text, sign))
  validate_state({ definition, current: definition.initial, tapes, }, multi)
}

///|
fn next_entry(s : State) -> Int32? {
  if s.definition.accepted.contains(s.current) {
    return None
  }
  let reads = s.tapes.map(t => t.head)
  for i, entry in s.definition.code {
    if entry.source == s.current && entry.reads == reads {
      return Some(i)
    }
  }
  None
}

///|
fn step(
  s : State,
  input : String,
) -> @runtime.Transition[State] raise @base.Fault {
  @base.unit_input(input)
  let s = match next_entry(s) {
    None => s
    Some(i) => {
      let e = s.definition.code[i]
      {
        ..s,
        current: e.target,
        tapes: s.tapes.mapi((i, tape) => {
          tape.write(e.writes[i]).shift(e.directions[i], "-")
        }),
      }
    }
  }
  match next_entry(s) {
    None => @runtime.Halt(@base.join(s.tapes.map(t => t.text(s => s)), "\n"))
    Some(_) => @runtime.Continue(s, "")
  }
}

///|
fn sign_json(s : String) -> Json {
  if s == "-" {
    Json::null()
  } else {
    @base.jstr(s)
  }
}

///|
fn sign_decode(j : Json) -> String raise @base.Fault {
  match j {
    Null => "-"
    _ => @base.identifier(@base.string(j))
  }
}

///|
fn dj(d : Int32) -> Json {
  @base.tag(if d < 0 { "Left" } else if d > 0 { "Right" } else { "Constant" }, [])
}

///|
fn dd(j : Json) -> Int32 raise @base.Fault {
  match @base.tag_name(j) {
    "Left" => -1
    "Right" => 1
    "Constant" => 0
    _ => @base.error("Invalid direction snapshot")
  }
}

///|
fn definition_json(d : Definition, multi : Bool) -> Json {
  let fields = [
    ("init_state", @base.jint(d.initial)),
    ("accepted_state", @base.jarray(d.accepted.map(@base.jint))),
    (
      "state_names",
      @base.jarray(
        d.names.map(pair => {
          @base.jarray([@base.jint(pair.0), @base.jstr(pair.1)])
        }),
      ),
    ),
    (
      "code",
      @base.jarray(
        d.code.map(e => {
          @base.jarray([
            @base.jarray([
              if multi {
                @base.jarray(e.reads.map(sign_json))
              } else {
                sign_json(e.reads[0])
              },
              @base.jint(e.source),
            ]),
            @base.jarray([
              if multi {
                @base.jarray(e.writes.map(sign_json))
              } else {
                sign_json(e.writes[0])
              },
              @base.jint(e.target),
              if multi {
                @base.jarray(e.directions.map(dj))
              } else {
                dj(e.directions[0])
              },
            ]),
          ])
        }),
      ),
    ),
  ]
  if multi {
    fields.push(("tape_count", @base.jint(d.count)))
    fields.push(("alphabet", @base.jarray(d.alphabet.map(sign_json))))
  }
  @base.object(fields)
}

///|
fn encode(s : State, multi : Bool) -> Json {
  @base.object([
    ("definition", definition_json(s.definition, multi)),
    ("current_state", @base.jint(s.current)),
    (
      if multi {
        "tapes"
      } else {
        "tape"
      },
      if multi {
        @base.jarray(s.tapes.map(t => t.json(sign_json)))
      } else {
        s.tapes[0].json(sign_json)
      },
    ),
  ])
}

///|
fn decode(j : Json, multi : Bool) -> State raise @base.Fault {
  let d = @base.field(j, "definition")
  let current = @base.field(j, "current_state")
  let named = match current {
    String(_) => !multi
    _ => false
  }
  let names : Array[(Int32, String)] = if named {
    []
  } else {
    @base.array(@base.field(d, "state_names")).map(j => {
      let a = @base.array(j)
      @base.require(a.length() == 2, "Invalid state name")
      (@base.int(a[0]), @base.identifier(@base.string(a[1])))
    })
  }
  fn state(j : Json) -> Int32 raise @base.Fault {
    if !named {
      return @base.int(j)
    }
    let name = @base.identifier(@base.string(j))
    for pair in names {
      if pair.1 == name {
        return pair.0
      }
    }
    let id = names.length()
    names.push((id, name))
    id
  }
  let initial = state(@base.field(d, "init_state"))
  let accepted = @base.array(@base.field(d, "accepted_state")).map(state)
  let code = @base.array(@base.field(d, "code")).map(j => {
    let a = @base.array(j)
    @base.require(a.length() == 2, "Invalid transition")
    let key = @base.array(a[0])
    let value = @base.array(a[1])
    @base.require(
      key.length() == 2 && value.length() == 3,
      "Invalid transition",
    )
    {
      reads: if multi {
        @base.array(key[0]).map(sign_decode)
      } else {
        [sign_decode(key[0])]
      },
      source: state(key[1]),
      writes: if multi {
        @base.array(value[0]).map(sign_decode)
      } else {
        [sign_decode(value[0])]
      },
      target: state(value[1]),
      directions: if multi {
        @base.array(value[2]).map(dd)
      } else {
        [dd(value[2])]
      },
    }
  })
  let current = if named {
    let name = @base.string(current)
    let found = names.iter().find_first(pair => pair.1 == name)
    match found {
      Some(pair) => pair.0
      None => @base.error("Unknown current state name")
    }
  } else {
    @base.int(current)
  }
  let definition = {
    count: if multi {
      @base.int(@base.field(d, "tape_count"))
    } else {
      1
    },
    alphabet: if multi {
      @base.array(@base.field(d, "alphabet")).map(sign_decode)
    } else {
      []
    },
    initial,
    accepted,
    code,
    names,
  }
  let tapes = if multi {
    @base.array(@base.field(j, "tapes")).map(j => {
      @data.decode_tape(j, sign_decode)
    })
  } else {
    [@data.decode_tape(@base.field(j, "tape"), sign_decode)]
  }
  validate_state({ definition, current, tapes, }, multi)
}

///|
fn render(s : State, multi : Bool) -> Json {
  fn name(id : Int32) -> String {
    s.definition.names.iter().find_first(pair => pair.0 == id).unwrap().1
  }
  let table = @base.table(
    "code",
    if multi {
      ["read", "state", "write", "next_state", "directions"]
    } else {
      ["key_sign", "key_state", "next_sign", "next_state", "direction"]
    },
    s.definition.code.map(e => {
      [
        @base.join(e.reads, " "),
        name(e.source),
        @base.join(e.writes, " "),
        name(e.target),
        @base.join(e.directions.map(direction_text), " "),
      ]
    }),
    highlight=next_entry(s).unwrap_or(-1),
  )
  let blocks = [table, @base.titled("state", name(s.current))]
  for i, tape in s.tapes {
    blocks.push(
      @base.highlight(
        if multi {
          "tape " + (i + 1).to_string()
        } else {
          "tape"
        },
        tape.left.length(),
        tape.values(),
      ),
    )
  }
  @base.jarray(blocks)
}

///|
pub fn model(multi~ : Bool = false) -> @runtime.Model {
  @runtime.model(
    (code, input) => make(code, input, multi),
    step,
    s => encode(s, multi),
    j => decode(j, multi),
    s => render(s, multi),
  )
}

///|
pub fn multi_model() -> @runtime.Model {
  model(multi=true)
}

///|
pub fn print_code(d : Definition) -> String {
  fn name(id : Int32) -> String {
    for entry in d.names {
      if entry.0 == id {
        return entry.1
      }
    }
    "q" + id.to_string()
  }
  let header = name(d.initial) +
    "\n" +
    @base.join(d.accepted.map(name), ",") +
    "\n"
  header +
  @base.join(
    d.code.map(entry => {
      @base.join(entry.reads, " ") +
      "," +
      name(entry.source) +
      "," +
      @base.join(entry.writes, " ") +
      "," +
      name(entry.target) +
      "," +
      @base.join(entry.directions.map(direction_text), " ") +
      "\n"
    }),
    "",
  )
}
