use utils::number::Number;
use utils::{Machine, StepResult};

#[derive(Debug, Clone, Default)]
pub struct MachineCode(pub Vec<Number>);


#[derive(Debug, Clone, Default)]
pub struct RegisterMemoryImage {
    pub register: [Number; 8],
    pub memory: Vec<Number>,
}


#[derive(Debug, Clone)]
pub struct TinyIsaMachine {
    pub regs: [Number; 8],
    pub flag: bool,
    pub code_len: usize,
    pub memory: Vec<Number>,
    pub halted: bool,
}

#[derive(Debug, Clone)]
pub(crate) enum DecodedInst {
    Nop,
    Halt,
    Rst,
    Ld {
        rd: usize,
        rb: usize,
    },
    St {
        rs: usize,
        rb: usize,
    },
    Mov {
        cond_flag: bool,
        rd: usize,
        rs: usize,
    },
    Ldi {
        cond_flag: bool,
        rd: usize,
        imm: Number,
    },
    Add {
        cond_flag: bool,
        rd: usize,
        rs: usize,
    },
    Sub {
        cond_flag: bool,
        rd: usize,
        rs: usize,
    },
    Addi {
        cond_flag: bool,
        rd: usize,
        imm: Number,
    },
    Subi {
        cond_flag: bool,
        rd: usize,
        imm: Number,
    },
    Eq {
        rd: usize,
        rs: usize,
    },
    Lt {
        rd: usize,
        rs: usize,
    },
    Gt {
        rd: usize,
        rs: usize,
    },
}

impl TinyIsaMachine {
    fn snapshot_output(&self) -> RegisterMemoryImage {
        RegisterMemoryImage {
            register: self.regs.clone(),
            memory: self.memory[self.code_len..].to_vec(),
        }
    }

    fn pc(&self) -> Result<usize, String> {
        self.regs[0].as_usize()
    }

    fn read_mem(&self, addr: usize) -> Number {
        self.memory.get(addr).cloned().unwrap_or_default()
    }

    fn write_mem(&mut self, addr: usize, value: Number) {
        if addr >= self.memory.len() {
            self.memory.resize(addr + 1, Number::default());
        }
        self.memory[addr] = value;
    }

    fn advance_pc_if_not_r0_write(&mut self, writes_r0: bool) {
        if !writes_r0 {
            self.regs[0] += 1usize;
        }
    }

    fn write_reg_with_pc_rule(&mut self, rd: usize, value: Number) {
        self.regs[rd] = value;
        self.advance_pc_if_not_r0_write(rd == 0);
    }

    fn continue_result(self) -> StepResult<Self> {
        StepResult::Continue {
            next: self,
            output: (),
        }
    }

    fn halt_result(self) -> StepResult<Self> {
        let snapshot = self.clone();
        let output = snapshot.snapshot_output();
        StepResult::Halt { snapshot, output }
    }

    pub(crate) fn decode(word: &Number) -> Result<DecodedInst, String> {
        let bytes = word.as_u8array();
        let lo = bytes.first().copied().unwrap_or(0) as u16;
        let hi = bytes.get(1).copied().unwrap_or(0) as u16;
        let op = lo | (hi << 8);

        let top = ((op >> 8) & 0xFF) as u8;
        let mid = ((op >> 6) & 0x03) as u8;
        let d = ((op >> 3) & 0x07) as usize;
        let s = (op & 0x07) as usize;
        let imm = Number::from_u8array(bytes.get(2..).unwrap_or(&[]));

        match (top, mid, d, s) {
            (0x00, 0b00, 0, 0) => Ok(DecodedInst::Nop),
            (0x00, 0b01, 0, 0) => Ok(DecodedInst::Halt),
            (0x00, 0b00, 0b111, 0b111) => Ok(DecodedInst::Rst),

            (0x01, 0b00, rd, rb) => Ok(DecodedInst::Ld { rd, rb }),
            (0x02, 0b00, rs, rb) => Ok(DecodedInst::St { rs, rb }),

            (0x08, 0b00 | 0b01, rd, rs) => Ok(DecodedInst::Mov {
                cond_flag: (mid & 0b01) != 0,
                rd,
                rs,
            }),
            (0x08, 0b10 | 0b11, rd, 0) => Ok(DecodedInst::Ldi {
                cond_flag: (mid & 0b01) != 0,
                rd,
                imm,
            }),

            (0x04, 0b00 | 0b01, rd, rs) => Ok(DecodedInst::Add {
                cond_flag: (mid & 0b01) != 0,
                rd,
                rs,
            }),
            (0x05, 0b00 | 0b01, rd, rs) => Ok(DecodedInst::Sub {
                cond_flag: (mid & 0b01) != 0,
                rd,
                rs,
            }),
            (0x04, 0b10 | 0b11, rd, 0) => Ok(DecodedInst::Addi {
                cond_flag: (mid & 0b01) != 0,
                rd,
                imm,
            }),
            (0x05, 0b10 | 0b11, rd, 0) => Ok(DecodedInst::Subi {
                cond_flag: (mid & 0b01) != 0,
                rd,
                imm,
            }),

            // Compare instructions have no `f` bit semantics here; keep the encoding strict.
            (0x11, 0b00, rd, rs) => Ok(DecodedInst::Eq { rd, rs }),
            (0x12, 0b00, rd, rs) => Ok(DecodedInst::Lt { rd, rs }),
            (0x14, 0b00, rd, rs) => Ok(DecodedInst::Gt { rd, rs }),

            _ => Err(format!("Invalid instruction encoding: 0x{op:04x}")),
        }
    }
}

impl Machine for TinyIsaMachine {
    type Code = MachineCode;
    type AInput = RegisterMemoryImage;
    type FOutput = RegisterMemoryImage;
    type SnapShot = TinyIsaMachine;
    type RInput = ();
    type ROutput = ();

    fn make(code: Self::Code, ainput: Self::AInput) -> Result<Self, String> {
        let code_len = code.0.len();
        let mut memory = code.0;
        memory.extend(ainput.memory);

        Ok(Self {
            regs: ainput.register,
            flag: false,
            code_len,
            memory,
            halted: false,
        })
    }

    fn step(self, _rinput: Self::RInput) -> Result<StepResult<Self>, String> {
        if self.halted {
            return Ok(self.halt_result());
        }

        let mut next = self;
        let pc = next.pc()?;
        let word = next.read_mem(pc);
        let inst = Self::decode(&word)?;

        match inst {
            DecodedInst::Nop => {
                next.regs[0] += 1usize;
                Ok(next.continue_result())
            }
            DecodedInst::Halt => {
                next.halted = true;
                Ok(next.halt_result())
            }
            DecodedInst::Rst => {
                next.flag = false;
                next.regs[0] += 1usize;
                Ok(next.continue_result())
            }
            DecodedInst::Ld { rd, rb } => {
                let addr = next.regs[rb].as_usize()?;
                let value = next.read_mem(addr);
                next.write_reg_with_pc_rule(rd, value);
                Ok(next.continue_result())
            }
            DecodedInst::St { rs, rb } => {
                let addr = next.regs[rb].as_usize()?;
                let value = next.regs[rs].clone();
                next.write_mem(addr, value);
                next.regs[0] += 1usize;
                Ok(next.continue_result())
            }
            DecodedInst::Mov { cond_flag, rd, rs } => {
                if cond_flag && !next.flag {
                    next.regs[0] += 1usize;
                } else {
                    let value = next.regs[rs].clone();
                    next.write_reg_with_pc_rule(rd, value);
                }
                Ok(next.continue_result())
            }
            DecodedInst::Ldi { cond_flag, rd, imm } => {
                if cond_flag && !next.flag {
                    next.regs[0] += 1usize;
                } else {
                    next.write_reg_with_pc_rule(rd, imm);
                }
                Ok(next.continue_result())
            }
            DecodedInst::Add { cond_flag, rd, rs } => {
                if cond_flag && !next.flag {
                    next.regs[0] += 1usize;
                } else {
                    let value = next.regs[rd].clone() + next.regs[rs].clone();
                    next.write_reg_with_pc_rule(rd, value);
                }
                Ok(next.continue_result())
            }
            DecodedInst::Sub { cond_flag, rd, rs } => {
                if cond_flag && !next.flag {
                    next.regs[0] += 1usize;
                } else {
                    let value = next.regs[rd].clone() - next.regs[rs].clone();
                    next.write_reg_with_pc_rule(rd, value);
                }
                Ok(next.continue_result())
            }
            DecodedInst::Addi { cond_flag, rd, imm } => {
                if cond_flag && !next.flag {
                    next.regs[0] += 1usize;
                } else {
                    let value = next.regs[rd].clone() + imm;
                    next.write_reg_with_pc_rule(rd, value);
                }
                Ok(next.continue_result())
            }
            DecodedInst::Subi { cond_flag, rd, imm } => {
                if cond_flag && !next.flag {
                    next.regs[0] += 1usize;
                } else {
                    let value = next.regs[rd].clone() - imm;
                    next.write_reg_with_pc_rule(rd, value);
                }
                Ok(next.continue_result())
            }
            DecodedInst::Eq { rd, rs } => {
                next.flag = next.regs[rd] == next.regs[rs];
                next.regs[0] += 1usize;
                Ok(next.continue_result())
            }
            DecodedInst::Lt { rd, rs } => {
                next.flag = next.regs[rd] < next.regs[rs];
                next.regs[0] += 1usize;
                Ok(next.continue_result())
            }
            DecodedInst::Gt { rd, rs } => {
                next.flag = next.regs[rd] > next.regs[rs];
                next.regs[0] += 1usize;
                Ok(next.continue_result())
            }
        }
    }

    fn current(&self) -> Self::SnapShot {
        self.clone()
    }
}
