use utils::TextCodec;

use crate::tiny_isa::{MachineCode, RegisterMemoryImage};
use utils::number::Number;

impl TextCodec for MachineCode {
    fn parse(text: &str) -> Result<Self, String> {
        let mut code = Vec::new();
        for line in text.lines() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }
            code.push(Number::parse(line)?);
        }
        Ok(Self(code))
    }

    fn write_fmt(&self, f: &mut impl std::fmt::Write) -> std::fmt::Result {
        for (i, n) in self.0.iter().enumerate() {
            if i > 0 {
                writeln!(f)?;
            }
            n.write_fmt(f)?;
        }
        Ok(())
    }
}

impl TextCodec for RegisterMemoryImage {
    fn parse(text: &str) -> Result<Self, String> {
        let mut lines = text.lines();
        let reg_line = lines
            .find(|line| !line.trim().is_empty())
            .ok_or_else(|| "Expected first line with 8 registers".to_string())?;

        let reg_parts: Vec<_> = reg_line
            .split(',')
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .collect();
        if reg_parts.len() != 8 {
            return Err("Expected 8 registers on the first line (comma separated)".to_string());
        }

        let register: [Number; 8] = std::array::from_fn(|i| {
            Number::parse(reg_parts[i]).expect("validated in parse; unreachable")
        });

        let mut memory = Vec::new();
        for line in lines {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }
            memory.push(Number::parse(line)?);
        }

        Ok(Self { register, memory })
    }

    fn write_fmt(&self, f: &mut impl std::fmt::Write) -> std::fmt::Result {
        for (i, reg) in self.register.iter().enumerate() {
            if i > 0 {
                write!(f, ", ")?;
            }
            reg.write_fmt(f)?;
        }
        for mem in &self.memory {
            writeln!(f)?;
            mem.write_fmt(f)?;
        }
        Ok(())
    }
}
