///|
fn compile(d : @tm.Definition) -> @tm.Definition raise @base.Fault {
  @tm.validate(d, true)
  let names : Array[(Int32, String)] = []
  let code : Array[@tm.Entry] = []
  fn fresh(hint : String) -> Int32 {
    let id = names.length()
    names.push((id, hint + "_" + id.to_string()))
    id
  }
  fn emit(
    source : Int32,
    read : String,
    write : String,
    target : Int32,
    dir : Int32,
  ) -> Unit {
    code.push({
      source,
      reads: [read],
      writes: [write],
      target,
      directions: [dir],
    })
  }
  fn keep(source : Int32, sign : String, target : Int32, dir : Int32) -> Unit {
    emit(source, sign, sign, target, dir)
  }
  let cells = []
  for i in 0..<d.count {
    for sign in d.alphabet {
      cells.push((i, false, sign))
      cells.push((i, true, sign))
    }
  }
  fn each_cell(f : (String) -> Unit) -> Unit {
    for entry in cells {
      f(cell(entry.0, entry.1, entry.2))
    }
  }
  let states : Map[Int32, Int32] = Map([])
  for entry in d.names {
    states[entry.0] = fresh(entry.1)
  }
  fn state(id : Int32) -> Int32 raise @base.Fault {
    match states.get(id) {
      Some(s) => s
      None => @base.error("Unknown state")
    }
  }
  let stuck = fresh("stuck")
  let rewinds : Map[Int32, Int32] = Map([])
  fn rewind(target : Int32) -> Int32 {
    match rewinds.get(target) {
      Some(s) => s
      None => {
        let s = fresh("rewind")
        rewinds[target] = s
        each_cell(sign => keep(s, sign, s, -1))
        keep(s, "mt_right", s, -1)
        keep(s, "mt_left", target, 0)
        s
      }
    }
  }
  fn begin_scan(start : Int32, scan : Int32) -> Unit {
    keep(start, "mt_left", scan, 1)
  }
  fn extend_columns(start : Int32, target : Int32) -> Unit {
    let sweep = fresh("extend_right_scan")
    let left = Array::makei(d.count, _ => fresh("extend_left"))
    emit(start, "mt_left", cell(d.count - 1, false, "-"), left[d.count - 1], -1)
    for i = d.count - 1; i >= 1; i = i - 1 {
      emit(left[i], "-", cell(i - 1, false, "-"), left[i - 1], -1)
    }
    emit(left[0], "-", "mt_left", sweep, 1)
    each_cell(sign => keep(sweep, sign, sweep, 1))
    let right = Array::makei(d.count, _ => fresh("extend_right"))
    emit(sweep, "mt_right", cell(0, false, "-"), right[0], 1)
    for i in 0..<(d.count - 1) {
      emit(right[i], "-", cell(i + 1, false, "-"), right[i + 1], 1)
    }
    emit(right[d.count - 1], "-", "mt_right", rewind(target), -1)
  }
  for entry in d.names {
    let source = entry.0
    let rules = d.code.filter(rule => rule.source == source)
    let starts = rules.mapi((i, _) => {
      if i == 0 {
        state(source)
      } else {
        fresh("candidate")
      }
    })
    for rule_index, rule in rules {
      let fail = if rule_index + 1 == starts.length() {
        stuck
      } else {
        starts[rule_index + 1]
      }
      let expand = fresh("expand")
      let checks = Array::makei(d.count, i => {
        if i == 0 {
          starts[rule_index]
        } else {
          fresh("check")
        }
      })
      for i, expected in rule.reads {
        let scan = fresh("read_head")
        let success = if i + 1 == d.count { expand } else { checks[i + 1] }
        begin_scan(checks[i], scan)
        for entry in cells {
          let (j, head, sign) = entry
          let encoded = cell(j, head, sign)
          if i == j && head {
            keep(
              scan,
              encoded,
              rewind(if sign == expected { success } else { fail }),
              -1,
            )
          } else {
            keep(scan, encoded, scan, 1)
          }
        }
        keep(scan, "mt_right", rewind(fail), -1)
      }
      let updates = Array::makei(d.count, _ => fresh("update"))
      extend_columns(expand, updates[0])
      for i, write in rule.writes {
        let dir = rule.directions[i]
        let scan = fresh("write_head")
        let next = if i + 1 == d.count {
          state(rule.target)
        } else {
          updates[i + 1]
        }
        let back = rewind(next)
        begin_scan(updates[i], scan)
        let destination = if dir == 0 {
          back
        } else {
          let mark = fresh("mark_head")
          for sign in d.alphabet {
            emit(mark, cell(i, false, sign), cell(i, true, sign), back, -1)
          }
          let mut remaining = mark
          for _ in 0..<(d.count - 1) {
            let moving = fresh("move_head")
            let target = remaining
            each_cell(sign => keep(moving, sign, target, dir))
            remaining = moving
          }
          remaining
        }
        for entry in cells {
          let (j, head, sign) = entry
          let encoded = cell(j, head, sign)
          if i == j && head {
            emit(
              scan,
              encoded,
              cell(i, dir == 0, write),
              destination,
              if dir == 0 {
                -1
              } else {
                dir
              },
            )
          } else {
            keep(scan, encoded, scan, 1)
          }
        }
      }
    }
  }
  {
    count: 1,
    alphabet: [],
    initial: state(d.initial),
    accepted: d.accepted.map(state),
    code,
    names,
  }
}

///|
fn unit_text(text : String) -> String raise @base.Fault {
  @base.unit_input(text)
  ""
}

///|
pub fn compiler() -> @runtime.Compiler {
  {
    compile_code: text => @tm.print_code(compile(@tm.parse_code(text, true))),
    encode_ainput: encode_input,
    encode_rinput: unit_text,
    decode_routput: unit_text,
    decode_foutput: decode_output,
  }
}
