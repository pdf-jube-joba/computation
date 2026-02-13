use crate::machine::*;
use utils::TextCodec;
use utils::alphabet::Alphabet;
use utils::parse::ParseTextCodec;

impl TextCodec for Direction {
    fn parse(text: &str) -> Result<Self, String> {
        let value = text.trim();
        match value {
            "R" => Ok(Direction::Right),
            "L" => Ok(Direction::Left),
            "C" => Ok(Direction::Constant),
            _ => Err(format!("Invalid direction: {}", value)),
        }
    }
    fn write_fmt(&self, f: &mut impl std::fmt::Write) -> std::fmt::Result {
        match self {
            Direction::Right => write!(f, "R"),
            Direction::Constant => write!(f, "C"),
            Direction::Left => write!(f, "L"),
        }
    }
}

impl TextCodec for Sign {
    fn parse(text: &str) -> Result<Self, String> {
        match <Alphabet as TextCodec>::parse(text) {
            Ok(alphabet) => Ok(Sign(Some(alphabet))),
            Err(err) => {
                if text.trim() == "-" {
                    Ok(Sign::blank())
                } else {
                    Err(format!("Invalid sign: {}, {}", text, err))
                }
            }
        }
    }

    fn write_fmt(&self, f: &mut impl std::fmt::Write) -> std::fmt::Result {
        match &self.0 {
            Some(alphabet) => alphabet.write_fmt(f),
            None => write!(f, "-"),
        }
    }
}

impl TextCodec for Tape {
    fn parse(text: &str) -> Result<Self, String> {
        let parts: Vec<&str> = text.split('|').collect();
        if parts.len() != 3 {
            return Err("Invalid tape format | format ... 0,1,2|3|4,5,6".to_string());
        }
        let mut v = vec![];
        for s in parts[0].split(',') {
            let s = <Sign as TextCodec>::parse(s.trim())?;
            v.push(s);
        }
        let pos = v.len();
        let head: Sign = <Sign as TextCodec>::parse(parts[1].trim())?;
        v.push(head.clone());
        for s in parts[2].split(',') {
            let sign: Sign = <Sign as TextCodec>::parse(s.trim())?;
            v.push(sign);
        }
        Tape::from_vec(v, pos)
    }

    fn write_fmt(&self, f: &mut impl std::fmt::Write) -> std::fmt::Result {
        let (tapes, pos) = self.into_vec();
        for i in 0..pos {
            if i > 0 {
                write!(f, ",")?;
            }
            write!(f, "{}", tapes[i].print())?;
        }

        write!(f, "|")?;
        write!(f, "{}", tapes[pos].print())?;
        write!(f, "|")?;

        for i in pos + 1..tapes.len() {
            if i > pos + 1 {
                write!(f, ",")?;
            }
            write!(f, "{}", tapes[i].print())?;
        }
        Ok(())
    }
}

#[test]
fn test_tape_text_codec() {
    let tape_str = "-|d|e,f,g";
    let tape: Tape = tape_str.parse_tc().unwrap();
    let mut output = String::new();
    tape.write_fmt(&mut output).unwrap();
    assert_eq!(tape_str, output);
}

impl TextCodec for State {
    fn parse(text: &str) -> Result<Self, String> {
        let al: Alphabet = text.parse_tc()?;
        Ok(State(al))
    }

    fn write_fmt(&self, f: &mut impl std::fmt::Write) -> std::fmt::Result {
        self.0.write_fmt(f)
    }
}

pub fn parse_one_code_entry(code: &str) -> Result<CodeEntry, String> {
    let v: Vec<_> = code.split(',').collect();
    if v.len() < 5 {
        return Err(format!("Invalid code entry: {}", code));
    }
    // .trim() で parse 用に成形する
    Ok((
        (v[0].trim().parse_tc()?, v[1].trim().parse_tc()?),
        (
            v[2].trim().parse_tc()?,
            v[3].trim().parse_tc()?,
            v[4].trim().parse_tc()?,
        ),
    ))
}

impl TextCodec for TuringMachineDefinition {
    fn parse(text: &str) -> Result<Self, String> {
        let mut lines = text.lines();

        let Some(init_state_line) = lines.next() else {
            return Err("Missing initial state line".to_string());
        };

        let init_state: State = init_state_line.trim().parse_tc()?;

        let Some(accepted_state_line) = lines.next() else {
            return Err("Missing accepted states line".to_string());
        };

        let accepted_state: Vec<State> = accepted_state_line
            .split(',')
            .map(|s| s.trim().parse_tc())
            .collect::<Result<_, _>>()?;

        let code: Vec<_> = lines
            .enumerate()
            .filter(|(_, line)| !line.trim().is_empty() && !line.starts_with('#'))
            .map(|(index, line)| {
                parse_one_code_entry(line).map_err(|err| {
                    format!("Error parsing code entry at line {}: {}", index + 1, err)
                })
            })
            .collect::<Result<_, _>>()?;

        TuringMachineDefinition::new(init_state, accepted_state, code).map_err(|e| e.to_string())
    }

    fn write_fmt(&self, f: &mut impl std::fmt::Write) -> std::fmt::Result {
        self.init_state().write_fmt(f)?;
        writeln!(f)?;
        for (i, state) in self.accepted_state().iter().enumerate() {
            if i > 0 {
                write!(f, ",")?;
            }
            state.write_fmt(f)?;
        }
        writeln!(f)?;
        for entry in self.code() {
            entry.0.0.write_fmt(f)?;
            write!(f, ",")?;
            entry.0.1.write_fmt(f)?;
            write!(f, ",")?;
            entry.1.0.write_fmt(f)?;
            write!(f, ",")?;
            entry.1.1.write_fmt(f)?;
            write!(f, ",")?;
            entry.1.2.write_fmt(f)?;
            writeln!(f)?;
        }
        Ok(())
    }
}
