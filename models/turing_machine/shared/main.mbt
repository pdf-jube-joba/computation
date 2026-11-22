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
struct State {
  definition : Definition
  current : Int32
  tapes : Array[@utils.Tape[String]]
}

///|
pub fn sign(text : String) -> String raise @utils.Fault {
  let text = text.trim().to_owned()
  if text == "-" {
    text
  } else {
    @utils.identifier(text)
  }
}

///|
fn direction(text : String) -> Int32 raise @utils.Fault {
  let p = @utils.create(text)
  let d = p.keyword()
  p.finish()
  match d {
    "L" => -1
    "C" => 0
    "R" => 1
    _ => @utils.error("Invalid direction")
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
pub fn validate(d : Definition, multi : Bool) -> Unit raise @utils.Fault {
  @utils.require(d.count > 0, "Tape count must be positive")
  if multi {
    @utils.require(
      d.alphabet.contains("-"),
      "Alphabet must contain the blank symbol (-)",
    )
    let seen : Map[String, Bool] = Map([])
    for s in d.alphabet {
      ignore(sign(s))
      @utils.require(!seen.contains(s), "Duplicate alphabet symbol")
      seen[s] = true
    }
  }
  let ids : Map[Int32, Bool] = Map([])
  let names : Map[String, Bool] = Map([])
  for pair in d.names {
    ignore(@utils.identifier(pair.1))
    @utils.require(
      pair.0 >= 0 && !ids.contains(pair.0),
      "Duplicate or negative state ID",
    )
    @utils.require(!names.contains(pair.1), "Duplicate state name")
    ids[pair.0] = true
    names[pair.1] = true
  }
  @utils.require(
    ids.contains(d.initial) && d.accepted.all(id => ids.contains(id)),
    "Unknown state ID",
  )
  let seen : Map[String, Bool] = Map([])
  for entry in d.code {
    @utils.require(
      ids.contains(entry.source) && ids.contains(entry.target),
      "Unknown state ID",
    )
    @utils.require(
      !d.accepted.contains(entry.source),
      "Code contains accepted state",
    )
    @utils.require(
      entry.reads.length() == d.count &&
      entry.writes.length() == d.count &&
      entry.directions.length() == d.count,
      "Transition vector length differs from tape count",
    )
    for s in entry.reads {
      ignore(sign(s))
      if multi {
        @utils.require(d.alphabet.contains(s), "Symbol outside alphabet: " + s)
      }
    }
    for s in entry.writes {
      ignore(sign(s))
      if multi {
        @utils.require(d.alphabet.contains(s), "Symbol outside alphabet: " + s)
      }
    }
    let key = entry.source.to_string() + ":" + @utils.join(entry.reads, ",")
    @utils.require(
      !seen.contains(key),
      "Duplicate transition for state/read vector",
    )
    seen[key] = true
  }
}

///|
pub fn parse_code(text : String, multi : Bool) -> Definition raise @utils.Fault {
  let lines = text.split("\n").map(s => s.to_owned()).collect()
  @utils.require(
    lines.length() >= (if multi { 4 } else { 2 }),
    "Missing code header lines",
  )
  let lines = if multi { lines.map(strip) } else { lines }
  fn header(prefix : String, line : String) -> String raise @utils.Fault {
    @utils.require(line.has_prefix(prefix), "Expected " + prefix)
    line
    .unsafe_substring(start=prefix.length(), end=line.length())
    .trim()
    .to_owned()
  }
  let count = if multi { @utils.integer(header("tapes:", lines[0])) } else { 1 }
  let alphabet = if multi {
    comma(header("alphabet:", lines[1])).map(sign)
  } else {
    []
  }
  let offset = if multi { 2 } else { 0 }
  let names : Array[(Int32, String)] = []
  fn state(name : String) -> Int32 raise @utils.Fault {
    let name = @utils.identifier(name.trim().to_owned())
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
    @utils.require(
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
fn validate_state(s : State, multi : Bool) -> State raise @utils.Fault {
  validate(s.definition, multi)
  @utils.require(
    s.definition.names.any(pair => pair.0 == s.current),
    "Unknown current state ID",
  )
  @utils.require(
    s.tapes.length() == s.definition.count,
    "Input tape count differs from code",
  )
  for tape in s.tapes {
    for value in tape.values() {
      ignore(sign(value))
      if multi {
        @utils.require(
          s.definition.alphabet.contains(value),
          "Input symbol outside alphabet: " + value,
        )
      }
    }
  }
  s
}

///|
fn make(
  definition : Definition,
  tapes : Array[@utils.Tape[String]],
  multi : Bool,
) -> State raise @utils.Fault {
  validate(definition, multi)
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
  _input : Rinput,
) -> @definition.Transition[State, Unit, Tapes] {
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
    None => @definition.Halt(Tapes::{ values: s.tapes, })
    Some(_) => @definition.Continue(s, ())
  }
}

///|
fn sign_json(s : String) -> Json {
  if s == "-" {
    Json::null()
  } else {
    @utils.jstr(s)
  }
}

///|
fn sign_decode(j : Json) -> String raise @utils.Fault {
  match j {
    Null => "-"
    _ => @utils.identifier(@utils.string(j))
  }
}

///|
fn dj(d : Int32) -> Json {
  @utils.tag(
    if d < 0 {
      "Left"
    } else if d > 0 {
      "Right"
    } else {
      "Constant"
    },
    [],
  )
}

///|
fn dd(j : Json) -> Int32 raise @utils.Fault {
  match @utils.tag_name(j) {
    "Left" => -1
    "Right" => 1
    "Constant" => 0
    _ => @utils.error("Invalid direction snapshot")
  }
}

///|
fn definition_json(d : Definition, multi : Bool) -> Json {
  let fields = [
    ("init_state", @utils.jint(d.initial)),
    ("accepted_state", @utils.jarray(d.accepted.map(@utils.jint))),
    (
      "state_names",
      @utils.jarray(
        d.names.map(pair => {
          @utils.jarray([@utils.jint(pair.0), @utils.jstr(pair.1)])
        }),
      ),
    ),
    (
      "code",
      @utils.jarray(
        d.code.map(e => {
          @utils.jarray([
            @utils.jarray([
              if multi {
                @utils.jarray(e.reads.map(sign_json))
              } else {
                sign_json(e.reads[0])
              },
              @utils.jint(e.source),
            ]),
            @utils.jarray([
              if multi {
                @utils.jarray(e.writes.map(sign_json))
              } else {
                sign_json(e.writes[0])
              },
              @utils.jint(e.target),
              if multi {
                @utils.jarray(e.directions.map(dj))
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
    fields.push(("tape_count", @utils.jint(d.count)))
    fields.push(("alphabet", @utils.jarray(d.alphabet.map(sign_json))))
  }
  @utils.object(fields)
}

///|
fn encode(s : State, multi : Bool) -> Json {
  @utils.object([
    ("definition", definition_json(s.definition, multi)),
    ("current_state", @utils.jint(s.current)),
    (
      if multi {
        "tapes"
      } else {
        "tape"
      },
      if multi {
        @utils.jarray(s.tapes.map(t => t.json(sign_json)))
      } else {
        s.tapes[0].json(sign_json)
      },
    ),
  ])
}

///|
fn decode(j : Json, multi : Bool) -> State raise @utils.Fault {
  let d = @utils.field(j, "definition")
  let current = @utils.field(j, "current_state")
  let named = match current {
    String(_) => !multi
    _ => false
  }
  let names : Array[(Int32, String)] = if named {
    []
  } else {
    @utils.array(@utils.field(d, "state_names")).map(j => {
      let a = @utils.array(j)
      @utils.require(a.length() == 2, "Invalid state name")
      (@utils.int(a[0]), @utils.identifier(@utils.string(a[1])))
    })
  }
  fn state(j : Json) -> Int32 raise @utils.Fault {
    if !named {
      return @utils.int(j)
    }
    let name = @utils.identifier(@utils.string(j))
    for pair in names {
      if pair.1 == name {
        return pair.0
      }
    }
    let id = names.length()
    names.push((id, name))
    id
  }
  let initial = state(@utils.field(d, "init_state"))
  let accepted = @utils.array(@utils.field(d, "accepted_state")).map(state)
  let code = @utils.array(@utils.field(d, "code")).map(j => {
    let a = @utils.array(j)
    @utils.require(a.length() == 2, "Invalid transition")
    let key = @utils.array(a[0])
    let value = @utils.array(a[1])
    @utils.require(
      key.length() == 2 && value.length() == 3,
      "Invalid transition",
    )
    {
      reads: if multi {
        @utils.array(key[0]).map(sign_decode)
      } else {
        [sign_decode(key[0])]
      },
      source: state(key[1]),
      writes: if multi {
        @utils.array(value[0]).map(sign_decode)
      } else {
        [sign_decode(value[0])]
      },
      target: state(value[1]),
      directions: if multi {
        @utils.array(value[2]).map(dd)
      } else {
        [dd(value[2])]
      },
    }
  })
  let current = if named {
    let name = @utils.string(current)
    let found = names.iter().find_first(pair => pair.1 == name)
    match found {
      Some(pair) => pair.0
      None => @utils.error("Unknown current state name")
    }
  } else {
    @utils.int(current)
  }
  let definition = {
    count: if multi {
      @utils.int(@utils.field(d, "tape_count"))
    } else {
      1
    },
    alphabet: if multi {
      @utils.array(@utils.field(d, "alphabet")).map(sign_decode)
    } else {
      []
    },
    initial,
    accepted,
    code,
    names,
  }
  let tapes = if multi {
    @utils.array(@utils.field(j, "tapes")).map(j => {
      @utils.decode_tape(j, sign_decode)
    })
  } else {
    [@utils.decode_tape(@utils.field(j, "tape"), sign_decode)]
  }
  validate_state({ definition, current, tapes, }, multi)
}

///|
fn render(s : State, multi : Bool) -> Array[@definition.Block] {
  fn name(id : Int32) -> String {
    s.definition.names.iter().find_first(pair => pair.0 == id).unwrap().1
  }
  let table = @definition.text_table(
    "code",
    if multi {
      ["read", "state", "write", "next_state", "directions"]
    } else {
      ["key_sign", "key_state", "next_sign", "next_state", "direction"]
    },
    s.definition.code.map(e => {
      [
        @utils.join(e.reads, " "),
        name(e.source),
        @utils.join(e.writes, " "),
        name(e.target),
        @utils.join(e.directions.map(direction_text), " "),
      ]
    }),
    highlight=next_entry(s).unwrap_or(-1),
  )
  let blocks = [table, @definition.titled("state", name(s.current))]
  for i, tape in s.tapes {
    blocks.push(
      @definition.highlight(
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
  blocks
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
  @definition.Model(make=(code, input) => make(code, [input.value], false), step=(
    state,
    input,
  ) => {
    match step(state, input) {
      @definition.Continue(next, output) => @definition.Continue(next, output)
      @definition.Halt(output) =>
        @definition.Halt(Tape::{ value: output.values[0], })
    }
  })
}

///|
pub fn multi_model() -> @definition.Model[
  MultiCode,
  Tapes,
  Unit,
  Unit,
  Tapes,
  MultiState,
] {
  @definition.Model(
    make=(code, input) => { value: make(code.definition, input.values, true), },
    step=(state, input) => {
      match step(state.value, input) {
        @definition.Continue(next, output) =>
          @definition.Continue({ value: next, }, output)
        @definition.Halt(output) => @definition.Halt(output)
      }
    },
  )
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
    @utils.join(d.accepted.map(name), ",") +
    "\n"
  header +
  @utils.join(
    d.code.map(entry => {
      @utils.join(entry.reads, " ") +
      "," +
      name(entry.source) +
      "," +
      @utils.join(entry.writes, " ") +
      "," +
      name(entry.target) +
      "," +
      @utils.join(entry.directions.map(direction_text), " ") +
      "\n"
    }),
    "",
  )
}
