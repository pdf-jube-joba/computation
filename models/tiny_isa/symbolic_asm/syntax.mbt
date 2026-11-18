///|
priv enum Builtin {
  This
  CodeLen
  DataLen
}

///|
priv enum RawValue {
  Imm(@base.Nat)
  Label(String)
  Const(String)
  Builtin(Builtin)
}

///|
priv enum Operand {
  Reg(Int32)
  Value(RawValue)
}

///|
priv struct RawInst {
  mnemonic : String
  cond_flag : Bool
  operands : Array[Operand]
}

///|
priv enum Directive {
  Text
  Data
  Equ(String, @base.Nat)
}

///|
priv enum Item {
  Directive(Directive)
  LabelDef(String)
  Inst(RawInst)
  DataValue(RawValue)
}

///|
priv struct Symbols {
  consts : Array[(String, @base.Nat)]
  text_labels : Array[(String, Int32)]
  data_labels : Array[(String, Int32)]
  code_len : Int32
  data_len : Int32
}

///|
priv struct DebugInfo {
  text_labels_by_addr : Array[(Int32, Array[String])]
  data_labels_by_addr : Array[(Int32, Array[String])]
}

///|
fn raw_value(p : @p.Parser) -> RawValue raise @base.Fault {
  if p.eat('#') {
    ignore(p.eat('+'))
    Imm(p.natural())
  } else if p.eat('@') {
    Label(p.ident())
  } else {
    match p.peek() {
      @p.Number(_) => Imm(p.natural())
      @p.Keyword(_) =>
        Builtin(
          match p.keyword() {
            "THIS" => This
            "CODE_LEN" => CodeLen
            "DATA_LEN" => DataLen
            _ => p.error("Expected a builtin address")
          },
        )
      _ => Const(p.ident())
    }
  }
}

///|
fn read_reg(p : @p.Parser) -> Int32 raise @base.Fault {
  let n = p.ident()
  @base.require(n.has_prefix("r"), "Expected register operand")
  let i = @base.index(@base.nat(n.unsafe_substring(start=1, end=n.length())))
  @base.require(
    i < 8,
    "Register out of range (expected 0..=7): " + i.to_string(),
  )
  i
}

///|
fn parse(text : String) -> Array[Item] raise @base.Fault {
  let code = []
  let mut data = false
  for line in text.split("\n") {
    let line = line.to_owned().split("//").head().unwrap().trim().to_owned()
    if line.is_empty() {
      continue
    }
    let p = @p.create(line)
    if line.has_prefix("@") && line.has_suffix(":") {
      p.expect('@')
      let n = p.ident()
      p.expect(':')
      p.finish()
      code.push(LabelDef(n))
      continue
    }
    match p.peek() {
      @p.Keyword("text") => {
        ignore(p.bump())
        p.finish()
        data = false
        code.push(Directive(Text))
      }
      @p.Keyword("data") => {
        ignore(p.bump())
        p.finish()
        data = true
        code.push(Directive(Data))
      }
      @p.Keyword("equ") => {
        ignore(p.bump())
        let n = p.ident()
        ignore(p.eat('#'))
        ignore(p.eat('+'))
        let v = p.natural()
        p.finish()
        code.push(Directive(Equ(n, v)))
      }
      _ => {
        let line = if line.has_suffix(";") {
          line
          .unsafe_substring(start=0, end=line.length() - 1)
          .trim()
          .to_owned()
        } else {
          line
        }
        let p = @p.create(line)
        if data {
          let words = String::from_array(
              line.to_array().map(c => if c == '\t' { ' ' } else { c }),
            )
            .split(" ")
            .filter(s => !s.is_empty())
            .count()
          @base.require(
            words == 1,
            "Data value line must contain a single value",
          )
          let v = raw_value(p)
          p.finish()
          code.push(DataValue(v))
        } else {
          let mnemonic = p.keyword()
          let cond_flag = p.eat_keyword("f")
          let operands = []
          while !p.eof() {
            operands.push(
              if p.eat('%') {
                Reg(read_reg(p))
              } else {
                Value(raw_value(p))
              },
            )
          }
          code.push(Inst({ mnemonic, cond_flag, operands, }))
        }
      }
    }
  }
  code
}

///|
fn raw_text(v : RawValue) -> String {
  match v {
    Imm(n) => n.to_string()
    Label(n) => "@" + n
    Const(n) => n
    Builtin(This) => "\\THIS"
    Builtin(CodeLen) => "\\CODE_LEN"
    Builtin(DataLen) => "\\DATA_LEN"
  }
}

///|
fn lower(raw : RawInst) -> @machine.Instr[RawValue] raise @base.Fault {
  let m = raw.mnemonic
  let cf = raw.cond_flag
  let ops = raw.operands
  fn arity(n : Int32) -> Unit raise @base.Fault {
    @base.require(
      ops.length() == n,
      m + " requires " + n.to_string() + " operands",
    )
  }
  fn no_flag() -> Unit raise @base.Fault {
    @base.require(!cf, m + " \\f is not supported")
  }
  fn reg(v : Operand) -> Int32 raise @base.Fault {
    match v {
      Reg(i) if i >= 0 && i < 8 => i
      _ => @base.error("Expected register operand")
    }
  }
  fn imm(v : Operand) -> RawValue raise @base.Fault {
    match v {
      Value(v) => v
      _ => @base.error("Expected value operand")
    }
  }
  match m {
    "nop" => {
      arity(0)
      no_flag()
      @machine.Nop
    }
    "halt" => {
      arity(0)
      no_flag()
      @machine.Halt
    }
    "rst" => {
      arity(0)
      no_flag()
      @machine.Rst
    }
    "jump" => {
      arity(1)
      match ops[0] {
        Reg(i) => @machine.Mov(cf, 0, reg(Reg(i)))
        v => @machine.Ldi(cf, 0, imm(v))
      }
    }
    _ => {
      arity(2)
      let a = ops[0]
      let b = ops[1]
      match m {
        "ld" => {
          no_flag()
          @machine.Ld(reg(a), reg(b))
        }
        "st" => {
          no_flag()
          @machine.St(reg(a), reg(b))
        }
        "mov" => @machine.Mov(cf, reg(a), reg(b))
        "ldi" => @machine.Ldi(cf, reg(a), imm(b))
        "add" =>
          match b {
            Reg(i) => @machine.Add(cf, reg(a), reg(Reg(i)))
            _ => @machine.Addi(cf, reg(a), imm(b))
          }
        "sub" =>
          match b {
            Reg(i) => @machine.Sub(cf, reg(a), reg(Reg(i)))
            _ => @machine.Subi(cf, reg(a), imm(b))
          }
        "addi" => @machine.Addi(cf, reg(a), imm(b))
        "subi" => @machine.Subi(cf, reg(a), imm(b))
        "eq" => {
          no_flag()
          @machine.Eq(reg(a), reg(b))
        }
        "lt" => {
          no_flag()
          @machine.Lt(reg(a), reg(b))
        }
        "gt" => {
          no_flag()
          @machine.Gt(reg(a), reg(b))
        }
        _ => @base.error("Unsupported symbolic instruction in preprocess: " + m)
      }
    }
  }
}

///|
fn symbols(code : Array[Item]) -> Symbols raise @base.Fault {
  let consts = []
  let text_labels = []
  let data_labels = []
  let mut code_len = 0
  let mut data_len = 0
  let mut data = false
  let seen : Map[String, Bool] = Map([])
  fn available(n : String) -> Unit raise @base.Fault {
    @base.require(!seen.contains(n), "Duplicate symbol definition: " + n)
    seen[n] = true
  }
  for item in code {
    match item {
      Directive(Text) => data = false
      Directive(Data) => data = true
      Directive(Equ(n, v)) => {
        available(n)
        consts.push((n, v))
      }
      LabelDef(n) => {
        available(n)
        if data {
          data_labels.push((n, data_len))
        } else {
          text_labels.push((n, code_len))
        }
      }
      Inst(_) => {
        @base.require(!data, "Instruction found outside \\text section")
        code_len += 1
      }
      DataValue(_) => {
        @base.require(data, "Data value found outside \\data section")
        data_len += 1
      }
    }
  }
  { consts, text_labels, data_labels, code_len, data_len, }
}

///|
fn resolve(
  symbols : Symbols,
  data : Bool,
  pos : Int32,
  v : RawValue,
) -> @base.Nat raise @base.Fault {
  match v {
    Imm(n) => n
    Const(n) =>
      match @data.env_get(symbols.consts, n) {
        Some(v) => v
        None => @base.error("Unknown const reference: " + n)
      }
    Builtin(This) => @base.nint(pos)
    Builtin(CodeLen) => @base.nint(symbols.code_len)
    Builtin(DataLen) => @base.nint(symbols.data_len)
    Label(n) =>
      match @data.env_get(symbols.text_labels, n) {
        Some(i) => @base.nint(i)
        None =>
          match @data.env_get(symbols.data_labels, n) {
            Some(i) => @base.nint(i)
            None =>
              @base.error(
                "Unknown label reference in " +
                (if data { "\\data" } else { "\\text" }) +
                ": @" +
                n,
              )
          }
      }
  }
}

///|
fn debug_labels(
  labels : Array[(String, Int32)],
) -> Array[(Int32, Array[String])] {
  let out : Array[(Int32, Array[String])] = []
  for p in labels {
    match out.search_by(v => v.0 == p.1) {
      Some(i) => out[i].1.push(p.0)
      None => out.push((p.1, [p.0]))
    }
  }
  for p in out {
    p.1.sort()
  }
  out.sort_by((a, b) => a.0.compare(b.0))
  out
}
