use utils::number::Number;
use utils::TextCodec;

use crate::symbolic_asm::{AsmCode, AsmInput, AsmOutput, RawProgramAst};

impl TextCodec for AsmCode {
    fn parse(text: &str) -> Result<Self, String> {
        Ok(Self(RawProgramAst::parse_source(text)?))
    }

    fn write_fmt(&self, f: &mut impl std::fmt::Write) -> std::fmt::Result {
        self.0.write_fmt(f)
    }
}

impl TextCodec for AsmInput {
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

        let regs: [Number; 8] = std::array::from_fn(|i| {
            Number::parse(reg_parts[i]).expect("validated in parse; unreachable")
        });

        let mut data_extension = Vec::new();
        for line in lines {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }
            data_extension.push(Number::parse(line)?);
        }

        Ok(Self {
            regs,
            data_extension,
        })
    }

    fn write_fmt(&self, f: &mut impl std::fmt::Write) -> std::fmt::Result {
        for (i, reg) in self.regs.iter().enumerate() {
            if i > 0 {
                write!(f, ", ")?;
            }
            reg.write_fmt(f)?;
        }
        for value in &self.data_extension {
            writeln!(f)?;
            value.write_fmt(f)?;
        }
        Ok(())
    }
}

impl TextCodec for AsmOutput {
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

        let regs: [Number; 8] = std::array::from_fn(|i| {
            Number::parse(reg_parts[i]).expect("validated in parse; unreachable")
        });

        let mut data_memory = Vec::new();
        for line in lines {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }
            data_memory.push(Number::parse(line)?);
        }

        Ok(Self { regs, data_memory })
    }

    fn write_fmt(&self, f: &mut impl std::fmt::Write) -> std::fmt::Result {
        for (i, reg) in self.regs.iter().enumerate() {
            if i > 0 {
                write!(f, ", ")?;
            }
            reg.write_fmt(f)?;
        }
        for value in &self.data_memory {
            writeln!(f)?;
            value.write_fmt(f)?;
        }
        Ok(())
    }
}
