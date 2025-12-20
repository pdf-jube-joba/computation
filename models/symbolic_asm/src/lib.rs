use serde::Serialize;
use utils::{Machine, TextCodec, alphabet::Alphabet, number::Number, parse::ParseTextCodec};

#[derive(Clone, Copy, Serialize)]
pub enum Register {
    R0,
    R1,
    R2,
    R3,
}

impl TextCodec for Register {
    fn parse(text: &str) -> Result<Self, String> {
        match text {
            "r0" => Ok(Register::R0),
            "r1" => Ok(Register::R1),
            "r2" => Ok(Register::R2),
            "r3" => Ok(Register::R3),
            _ => Err(format!("invalid register: {}", text)),
        }
    }

    fn write_fmt(&self, f: &mut impl std::fmt::Write) -> std::fmt::Result {
        let s = match self {
            Register::R0 => "r0",
            Register::R1 => "r1",
            Register::R2 => "r2",
            Register::R3 => "r3",
        };
        write!(f, "{}", s)
    }
}

#[derive(Clone, Serialize)]
pub struct Label(pub(crate) Alphabet);

impl TextCodec for Label {
    fn parse(text: &str) -> Result<Self, String> {
        if !text.starts_with(":") {
            return Err("label should start with ':'".to_string());
        }
        Ok(Label(text.trim_start_matches(":").parse_tc()?))
    }

    fn write_fmt(&self, f: &mut impl std::fmt::Write) -> std::fmt::Result {
        write!(f, ":{}", self.0.print())
    }
}

#[derive(Clone, Serialize)]
pub enum ImR {
    Nop,
    Halt,
    LoadImm {
        dest: Register,
        value: Number,
    },
    Load {
        dest: Register,
        addr: Number,
    },
    LoadLabel {
        dest: Register,
        label: Label,
    },
    Store {
        src: Register,
        value: Number,
    },
    StoreLabel {
        src: Register,
        label: Label,
    },
    Mov {
        dest: Register,
        src: Register,
    },
    Add {
        dest: Register,
        src: Register,
    },
    Sub {
        dest: Register,
        src: Register,
    },
    ReadPc {
        dest: Register,
    },
    JmpReg {
        target: Register,
    },
    JmpImm {
        value: Number,
    },
    JmpLabel {
        label: Label,
    },
    JmpRelReg {
        r: Register,
    },
    JmpRelImm {
        imm: Number,
    },
    JLtLabel {
        rl: Register,
        rr: Register,
        addr: Label,
    },
    JltRel {
        rl: Register,
        rr: Register,
        imm: Number,
    },
    JltRelBack {
        rl: Register,
        rr: Register,
        imm: Number,
    },
}

impl TextCodec for ImR {
    fn parse(text: &str) -> Result<Self, String> {
        let mut words = text.split_whitespace();
        let Some(first) = words.next() else {
            return Err("empty instruction".to_string());
        };
        match first {
            "NOP" => Ok(ImR::Nop),
            "HLT" => Ok(ImR::Halt),
            "LDI" => {
                let dest: Register = words.next().ok_or("missing dest register")?.parse_tc()?;
                let value: Number = words.next().ok_or("missing immediate value")?.parse_tc()?;
                Ok(ImR::LoadImm { dest, value })
            }
            "LD" => {
                let dest: Register = words.next().ok_or("missing dest register")?.parse_tc()?;
                let addr: Number = words.next().ok_or("missing address value")?.parse_tc()?;
                Ok(ImR::Load { dest, addr })
            }
            "LDL" => {
                let dest: Register = words.next().ok_or("missing dest register")?.parse_tc()?;
                let label = words.next().ok_or("missing label")?.parse_tc()?;
                Ok(ImR::LoadLabel { dest, label })
            }
            "ST" => {
                let src: Register = words.next().ok_or("missing src register")?.parse_tc()?;
                let value: Number = words.next().ok_or("missing value")?.parse_tc()?;
                Ok(ImR::Store { src, value })
            }
            "STL" => {
                let src: Register = words.next().ok_or("missing src register")?.parse_tc()?;
                let label = words.next().ok_or("missing label")?.parse_tc()?;
                Ok(ImR::StoreLabel { src, label })
            }
            "MOV" => {
                let dest: Register = words.next().ok_or("missing dest register")?.parse_tc()?;
                let src: Register = words.next().ok_or("missing src register")?.parse_tc()?;
                Ok(ImR::Mov { dest, src })
            }
            "ADD" => {
                let dest: Register = words.next().ok_or("missing dest register")?.parse_tc()?;
                let src: Register = words.next().ok_or("missing src register")?.parse_tc()?;
                Ok(ImR::Add { dest, src })
            }
            "SUB" => {
                let dest: Register = words.next().ok_or("missing dest register")?.parse_tc()?;
                let src: Register = words.next().ok_or("missing src register")?.parse_tc()?;
                Ok(ImR::Sub { dest, src })
            }
            "RPC" => {
                let dest: Register = words.next().ok_or("missing dest register")?.parse_tc()?;
                Ok(ImR::ReadPc { dest })
            }
            "JMPR" => {
                let target: Register = words.next().ok_or("missing target register")?.parse_tc()?;
                Ok(ImR::JmpReg { target })
            }
            "JMP" => {
                let value: Number = words.next().ok_or("missing immediate value")?.parse_tc()?;
                Ok(ImR::JmpImm { value })
            }
            "JMPL" => {
                let label = words.next().ok_or("missing label")?.parse_tc()?;
                Ok(ImR::JmpLabel { label })
            }
            "JMRR" => {
                let r: Register = words.next().ok_or("missing register")?.parse_tc()?;
                Ok(ImR::JmpRelReg { r })
            }
            "JMRI" => {
                let imm: Number = words.next().ok_or("missing immediate value")?.parse_tc()?;
                Ok(ImR::JmpRelImm { imm })
            }
            "JLTL" => {
                let rl: Register = words.next().ok_or("missing left register")?.parse_tc()?;
                let rr: Register = words.next().ok_or("missing right register")?.parse_tc()?;
                let addr: Label = words.next().ok_or("missing label")?.parse_tc()?;
                Ok(ImR::JLtLabel { rl, rr, addr })
            }
            "JLTI" => {
                let rl: Register = words.next().ok_or("missing left register")?.parse_tc()?;
                let rr: Register = words.next().ok_or("missing right register")?.parse_tc()?;
                let imm: Number = words.next().ok_or("missing immediate value")?.parse_tc()?;
                Ok(ImR::JltRel { rl, rr, imm })
            }
            "JLTR" => {
                let rl: Register = words.next().ok_or("missing left register")?.parse_tc()?;
                let rr: Register = words.next().ok_or("missing right register")?.parse_tc()?;
                let imm: Number = words.next().ok_or("missing immediate value")?.parse_tc()?;
                Ok(ImR::JltRelBack { rl, rr, imm })
            }
            _ => Err(format!("invalid instruction: {}", first)),
        }
    }

    fn write_fmt(&self, f: &mut impl std::fmt::Write) -> std::fmt::Result {
        todo!()
    }
}

#[derive(Clone, Serialize)]
pub struct Code(Vec<(Label, Vec<ImR>)>);

impl TextCodec for Code {
    fn parse(text: &str) -> Result<Self, String> {
        let mut code = vec![];
        let mut lines = text
            .lines()
            .filter(|line| !line.trim().is_empty() && !line.trim().starts_with("//"))
            .map(|line| line.trim());

        let mut v = vec![];

        // label is start with ":"
        let mut current_label: Label = {
            let first_line = lines.next().ok_or("empty code")?.trim().to_string();
            if !first_line.starts_with(":") {
                return Err("code must start with a label".to_string());
            }
            Label(first_line.trim_start_matches(":").parse_tc()?)
        };

        for line in lines {
            let line = line.trim();
            if line.starts_with(":") {
                // new label
                code.push((current_label, v));
                v = vec![];
                current_label = Label(line.trim_start_matches(":").parse_tc()?);
            } else {
                let instr = ImR::parse(line)?;
                v.push(instr);
            }
        }

        code.push((current_label, v));
        Ok(Code(code))
    }

    fn write_fmt(&self, f: &mut impl std::fmt::Write) -> std::fmt::Result {
        for (label, instrs) in &self.0 {
            writeln!(f, ":{}", label.0.print())?;
            for instr in instrs {
                writeln!(f, "    {}", instr.print())?;
            }
        }
        Ok(())
    }
}

impl Code {
    fn get_addr_from_label(&self, label: Label) -> Result<Number, String> {
        let mut addr = 0;
        for (lbl, instrs) in &self.0 {
            if lbl.0 == label.0 {
                return Ok(Number::from(addr));
            }
            addr += instrs.len();
        }
        Err("label not found in code".to_string())
    }
}

#[derive(Clone, Serialize)]
pub struct Dat(Vec<(Label, Number)>);

impl TextCodec for Dat {
    fn parse(text: &str) -> Result<Self, String> {
        let mut data = vec![];

        let lines = text
            .lines()
            .filter(|line| !line.trim().is_empty() && !line.trim().starts_with("//"))
            .map(|line| line.trim());

        for line in lines {
            let mut parts = line.split_whitespace();
            let label_str = parts.next().ok_or("data line must start with a label")?;
            if !label_str.starts_with(":") {
                return Err("data label must start with ':'".to_string());
            }
            let label = Label(label_str.trim_start_matches(":").parse_tc()?);
            let value_str = parts
                .next()
                .ok_or("data line must have a value after the label")?;
            let value = value_str.parse_tc()?;
            data.push((label, value));
        }
        Ok(Dat(data))
    }

    fn write_fmt(&self, f: &mut impl std::fmt::Write) -> std::fmt::Result {
        for (label, value) in &self.0 {
            writeln!(f, ":{} {}", label.0.print(), value.print())?;
        }
        Ok(())
    }
}

#[derive(Clone, Serialize)]
pub struct Environment {
    pub dat: Dat,
    pub code: Code,
    pub registers: [Number; 4],
    pub pc: Number,
}

impl Environment {
    fn get_register_mut(&mut self, reg: Register) -> &mut Number {
        match reg {
            Register::R0 => &mut self.registers[0],
            Register::R1 => &mut self.registers[1],
            Register::R2 => &mut self.registers[2],
            Register::R3 => &mut self.registers[3],
        }
    }
    fn get_data_mut_number(&mut self, addr: Number) -> Result<&mut Number, String> {
        let index = addr.as_usize();
        if index >= self.dat.0.len() {
            return Err("data address out of bounds".to_string());
        }
        Ok(&mut self.dat.0[index].1)
    }
    fn get_data_mut_label(&mut self, label: Label) -> Result<&mut Number, String> {
        for (lbl, value) in &mut self.dat.0 {
            if lbl.0 == label.0 {
                return Ok(value);
            }
        }
        Err("data label not found".to_string())
    }
}

// we need to implement Machine for asm
// i.e. we need to define semantics for this assembly language (independently from the VM)

impl Machine for Environment {
    type Code = Code;

    type AInput = Dat;

    type SnapShot = Environment;

    type RInput = ();

    type Output = Dat;

    fn make(code: Self::Code, ainput: Self::AInput) -> Result<Self, String> {
        Ok(Environment {
            dat: ainput,
            code,
            registers: std::array::from_fn(|_| Number::from(0)),
            pc: Number::from(0),
        })
    }

    fn step(&mut self, _rinput: Self::RInput) -> Result<Option<Self::Output>, String> {
        let inst: &ImR = self
            .code
            .0
            .iter()
            .flat_map(|(_, instrs)| instrs)
            .nth(self.pc.as_usize())
            .ok_or("PC out of bounds")?;
        match *inst {
            ImR::Nop => {
                self.pc += 1;
                Ok(None)
            }
            ImR::Halt => Ok(Some(self.dat.clone())),
            ImR::LoadImm { dest, ref value } => {
                *self.get_register_mut(dest) = value.clone();
                self.pc += 1;
                Ok(None)
            }
            ImR::Load { dest, ref addr } => {
                let addr = addr.clone();
                let value = self.get_data_mut_number(addr)?.clone();
                *self.get_register_mut(dest) = value;
                self.pc += 1;
                Ok(None)
            }
            ImR::LoadLabel { dest, ref label } => {
                let label = label.clone();
                let value = self.get_data_mut_label(label)?.clone();
                *self.get_register_mut(dest) = value;
                self.pc += 1;
                Ok(None)
            }
            ImR::Store { src, ref value } => {
                let value = value.clone();
                let data_addr = self.get_register_mut(src).clone();
                let data_cell = self.get_data_mut_number(data_addr)?;
                *data_cell = value;
                self.pc += 1;
                Ok(None)
            }
            ImR::StoreLabel { src, ref label } => {
                let label = label.clone();
                let value = self.get_register_mut(src).clone();
                let data_cell = self.get_data_mut_label(label)?;
                *data_cell = value;
                self.pc += 1;
                Ok(None)
            }
            ImR::Mov { dest, src } => {
                let value = self.get_register_mut(src).clone();
                *self.get_register_mut(dest) = value;
                self.pc += 1;
                Ok(None)
            }
            ImR::Add { dest, src } => {
                let value =
                    self.get_register_mut(dest).clone() + self.get_register_mut(src).clone();
                *self.get_register_mut(dest) = value;
                self.pc += 1;
                Ok(None)
            }
            ImR::Sub { dest, src } => {
                let value =
                    self.get_register_mut(dest).clone() - self.get_register_mut(src).clone();
                *self.get_register_mut(dest) = value;
                self.pc += 1;
                Ok(None)
            }
            ImR::ReadPc { dest } => {
                *self.get_register_mut(dest) = self.pc.clone();
                self.pc += 1;
                Ok(None)
            }
            ImR::JmpReg { target } => {
                let target_addr = self.get_register_mut(target).clone();
                self.pc = target_addr;
                Ok(None)
            }
            ImR::JmpImm { ref value } => {
                let value = value.clone();
                self.pc = value;
                Ok(None)
            }
            ImR::JmpLabel { ref label } => {
                let label = label.clone();
                let addr = self.code.get_addr_from_label(label)?;
                self.pc = addr;
                Ok(None)
            }
            ImR::JmpRelReg { r } => {
                let offset = self.get_register_mut(r).clone();
                self.pc = self.pc.clone() + offset;
                Ok(None)
            }
            ImR::JmpRelImm { ref imm } => {
                self.pc = self.pc.clone() + imm.clone();
                Ok(None)
            }
            ImR::JLtLabel { rl, rr, ref addr } => {
                let label = addr.clone();
                let val_l = self.get_register_mut(rl).clone();
                let val_r = self.get_register_mut(rr).clone();
                if val_l < val_r {
                    let addr = self.code.get_addr_from_label(label)?;
                    self.pc = addr;
                } else {
                    self.pc += 1;
                }
                Ok(None)
            }
            ImR::JltRel { rl, rr, ref imm } => {
                let imm = imm.clone();
                let val_l = self.get_register_mut(rl).clone();
                let val_r = self.get_register_mut(rr).clone();
                if val_l < val_r {
                    self.pc = self.pc.clone() + imm;
                } else {
                    self.pc += 1;
                }
                Ok(None)
            }
            ImR::JltRelBack { rl, rr, ref imm } => {
                let imm = imm.clone();
                let val_l = self.get_register_mut(rl).clone();
                let val_r = self.get_register_mut(rr).clone();
                if val_l < val_r {
                    self.pc = self.pc.clone() - imm;
                } else {
                    self.pc += 1;
                }
                Ok(None)
            }
        }
    }

    fn current(&self) -> Self::SnapShot {
        self.clone()
    }
}
