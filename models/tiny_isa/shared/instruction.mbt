///|
pub(all) enum Instr[T] {
  Nop
  Halt
  Rst
  Ld(Int32, Int32)
  St(Int32, Int32)
  Mov(Bool, Int32, Int32)
  Ldi(Bool, Int32, T)
  Add(Bool, Int32, Int32)
  Sub(Bool, Int32, Int32)
  Addi(Bool, Int32, T)
  Subi(Bool, Int32, T)
  Eq(Int32, Int32)
  Lt(Int32, Int32)
  Gt(Int32, Int32)
}

///|
pub fn read(
  memory : Array[@base.Nat],
  addr : Int32,
) -> @base.Nat raise @base.Fault {
  @base.require(addr >= 0, "Negative address")
  if addr < memory.length() {
    memory[addr]
  } else {
    @base.zero()
  }
}

///|
pub fn write(
  memory : Array[@base.Nat],
  addr : Int32,
  value : @base.Nat,
) -> Array[@base.Nat] raise @base.Fault {
  @base.require(addr >= 0 && addr < 2147483647, "Memory address out of range")
  let memory = Array::makei(Int32::max(memory.length(), addr + 1), i => {
    if i < memory.length() {
      memory[i]
    } else {
      @base.zero()
    }
  })
  memory[addr] = value
  memory
}

///|
pub fn validate_regs(regs : Array[@base.Nat]) -> Unit raise @base.Fault {
  @base.require(regs.length() == 8, "Expected 8 registers")
}

///|
pub fn[T] execute(
  inst : Instr[T],
  resolve : (T) -> @base.Nat raise @base.Fault,
  regs : Array[@base.Nat],
  flag : Bool,
  memory : Array[@base.Nat],
) -> (Array[@base.Nat], Bool, Array[@base.Nat], Bool) raise @base.Fault {
  validate_regs(regs)
  let regs = regs.copy()
  fn check(i : Int32) -> Unit raise @base.Fault {
    @base.require(i >= 0 && i < 8, "Register out of range (expected 0..=7)")
  }
  fn advance() -> Unit {
    regs[0] = regs[0] + @base.one()
  }
  fn assign(d : Int32, n : @base.Nat) -> Unit raise @base.Fault {
    check(d)
    regs[d] = n
    if d != 0 {
      advance()
    }
  }
  fn arithmetic(
    f : Bool,
    d : Int32,
    rhs : () -> @base.Nat raise @base.Fault,
    sub : Bool,
  ) -> Unit raise @base.Fault {
    check(d)
    if f && !flag {
      advance()
    } else {
      let n = rhs()
      assign(d, if sub { @base.sub(regs[d], n) } else { regs[d] + n })
    }
  }
  match inst {
    Halt => (regs, flag, memory, true)
    Nop => {
      advance()
      (regs, flag, memory, false)
    }
    Rst => {
      advance()
      (regs, false, memory, false)
    }
    Ld(d, b) => {
      check(b)
      assign(d, read(memory, @base.index(regs[b])))
      (regs, flag, memory, false)
    }
    St(s, b) => {
      check(s)
      check(b)
      let memory = write(memory, @base.index(regs[b]), regs[s])
      advance()
      (regs, flag, memory, false)
    }
    Mov(f, d, s) => {
      check(d)
      check(s)
      if f && !flag {
        advance()
      } else {
        assign(d, regs[s])
      }
      (regs, flag, memory, false)
    }
    Ldi(f, d, n) => {
      check(d)
      if f && !flag {
        advance()
      } else {
        assign(d, resolve(n))
      }
      (regs, flag, memory, false)
    }
    Add(f, d, s) => {
      check(s)
      arithmetic(f, d, () => regs[s], false)
      (regs, flag, memory, false)
    }
    Sub(f, d, s) => {
      check(s)
      arithmetic(f, d, () => regs[s], true)
      (regs, flag, memory, false)
    }
    Addi(f, d, n) => {
      arithmetic(f, d, () => resolve(n), false)
      (regs, flag, memory, false)
    }
    Subi(f, d, n) => {
      arithmetic(f, d, () => resolve(n), true)
      (regs, flag, memory, false)
    }
    Eq(d, s) => {
      check(d)
      check(s)
      let flag = regs[d] == regs[s]
      advance()
      (regs, flag, memory, false)
    }
    Lt(d, s) => {
      check(d)
      check(s)
      let flag = regs[d] < regs[s]
      advance()
      (regs, flag, memory, false)
    }
    Gt(d, s) => {
      check(d)
      check(s)
      let flag = regs[d] > regs[s]
      advance()
      (regs, flag, memory, false)
    }
  }
}

///|
pub fn decode(word : @base.Nat) -> Instr[@base.Nat] raise @base.Fault {
  let imm = word / @base.nint(65536)
  let op = (word % @base.nint(65536)).to_int()
  let top = op >> 8
  let mid = (op >> 6) & 3
  let d = (op >> 3) & 7
  let s = op & 7
  let cf = (mid & 1) != 0
  match (top, mid, d, s) {
    (0, 0, 0, 0) => Nop
    (0, 1, 0, 0) => Halt
    (0, 0, 7, 7) => Rst
    (1, 0, d, s) => Ld(d, s)
    (2, 0, d, s) => St(d, s)
    (8, 0 | 1, d, s) => Mov(cf, d, s)
    (8, 2 | 3, d, 0) => Ldi(cf, d, imm)
    (4, 0 | 1, d, s) => Add(cf, d, s)
    (5, 0 | 1, d, s) => Sub(cf, d, s)
    (4, 2 | 3, d, 0) => Addi(cf, d, imm)
    (5, 2 | 3, d, 0) => Subi(cf, d, imm)
    (17, 0, d, s) => Eq(d, s)
    (18, 0, d, s) => Lt(d, s)
    (20, 0, d, s) => Gt(d, s)
    _ => @base.error("Invalid instruction encoding: " + op.to_string())
  }
}

///|
pub fn encode(i : Instr[@base.Nat]) -> @base.Nat {
  fn rr(top : Int32, mid : Int32, d : Int32, s : Int32) -> @base.Nat {
    @base.nint((top << 8) | (mid << 6) | (d << 3) | s)
  }
  fn cf(flag : Bool) -> Int32 {
    if flag {
      1
    } else {
      0
    }
  }
  fn ri(top : Int32, f : Bool, d : Int32, n : @base.Nat) -> @base.Nat {
    rr(top, 2 + cf(f), d, 0) + n * @base.nint(65536)
  }
  match i {
    Nop => @base.zero()
    Halt => @base.nint(64)
    Rst => @base.nint(63)
    Ld(d, s) => rr(1, 0, d, s)
    St(d, s) => rr(2, 0, d, s)
    Mov(f, d, s) => rr(8, cf(f), d, s)
    Ldi(f, d, n) => ri(8, f, d, n)
    Add(f, d, s) => rr(4, cf(f), d, s)
    Sub(f, d, s) => rr(5, cf(f), d, s)
    Addi(f, d, n) => ri(4, f, d, n)
    Subi(f, d, n) => ri(5, f, d, n)
    Eq(d, s) => rr(17, 0, d, s)
    Lt(d, s) => rr(18, 0, d, s)
    Gt(d, s) => rr(20, 0, d, s)
  }
}

///|
pub fn[T] text(inst : Instr[T], show : (T) -> String) -> String {
  match inst {
    Nop => "Nop"
    Halt => "Halt"
    Rst => "Rst"
    Ld(d, b) => "Ld { rd: \{d}, rb: \{b} }"
    St(s, b) => "St { rs: \{s}, rb: \{b} }"
    Mov(f, d, s) => "Mov { cond_flag: \{f}, rd: \{d}, rs: \{s} }"
    Ldi(f, d, n) => "Ldi { cond_flag: \{f}, rd: \{d}, imm: " + show(n) + " }"
    Add(f, d, s) => "Add { cond_flag: \{f}, rd: \{d}, rs: \{s} }"
    Sub(f, d, s) => "Sub { cond_flag: \{f}, rd: \{d}, rs: \{s} }"
    Addi(f, d, n) => "Addi { cond_flag: \{f}, rd: \{d}, imm: " + show(n) + " }"
    Subi(f, d, n) => "Subi { cond_flag: \{f}, rd: \{d}, imm: " + show(n) + " }"
    Eq(d, s) => "Eq { rd: \{d}, rs: \{s} }"
    Lt(d, s) => "Lt { rd: \{d}, rs: \{s} }"
    Gt(d, s) => "Gt { rd: \{d}, rs: \{s} }"
  }
}

///|
pub fn registers(regs : Array[@base.Nat]) -> Json {
  @base.table(
    "registers",
    ["reg", "value"],
    regs.mapi((i, n) => ["r" + i.to_string(), n.to_string()]),
  )
}

///|
pub fn natural_lines(text : String) -> Array[@base.Nat] raise @base.Fault {
  text
  .split("\n")
  .filter(s => !s.trim().is_empty())
  .map(s => s.trim().to_owned())
  .collect()
  .map(@base.nat)
}

///|
pub(all) struct Image {
  register : Array[@base.Nat]
  memory : Array[@base.Nat]
}

///|
pub fn image(text : String) -> Image raise @base.Fault {
  let lines = text
    .split("\n")
    .filter(s => !s.trim().is_empty())
    .map(s => s.trim().to_owned())
    .collect()
  @base.require(!lines.is_empty(), "Expected first line with 8 registers")
  let regs = lines[0]
    .split(",")
    .filter(s => !s.trim().is_empty())
    .map(s => s.trim().to_owned())
    .collect()
    .map(@base.nat)
  validate_regs(regs)
  { register: regs, memory: lines[1:].map(@base.nat), }
}

///|
pub fn image_text(regs : Array[@base.Nat], memory : Array[@base.Nat]) -> String {
  @base.join(regs.map(n => n.to_string()), ", ") +
  (if memory.is_empty() {
    ""
  } else {
    "\n" + @base.join(memory.map(n => n.to_string()), "\n")
  })
}

///|
pub fn nats_json(ns : Array[@base.Nat]) -> Json {
  @base.jarray(ns.map(n => @base.jstr(n.to_string())))
}

///|
pub fn decode_nats(j : Json) -> Array[@base.Nat] raise @base.Fault {
  @base.array(j).map(@base.json_nat)
}
