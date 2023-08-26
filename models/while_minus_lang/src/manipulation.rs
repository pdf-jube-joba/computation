use crate::machine::{FlatWhileStatement, Var};
use utils::*;

fn one_line_parse(line: &str) -> Result<FlatWhileStatement, ()> {
    let mut words = line.split_whitespace();
    if let Some(str) = words.next() {
        match str {
            "inc" => {
                let num: usize = words.next().ok_or(())?.parse().map_err(|_| ())?;
                Ok(FlatWhileStatement::inc(Var(num)))
            },
            _ => Err(()),
        }
    } else {
        Err(())
    }
}