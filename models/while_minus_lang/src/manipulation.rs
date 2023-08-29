use crate::machine::{FlatWhileLanguage, FlatWhileStatement, Var, WhileLanguage};

fn one_line_parse(line: &str) -> Result<FlatWhileStatement, ()> {
    let mut words = line.split_whitespace();
    if let Some(str) = words.next() {
        match str {
            "inc" => {
                let var: Var = words.next().ok_or(())?.try_into()?;
                Ok(FlatWhileStatement::inc(var))
            }
            "dec" => {
                let var: Var = words.next().ok_or(())?.try_into()?;
                Ok(FlatWhileStatement::dec(var))
            }
            "init" => {
                let var: Var = words.next().ok_or(())?.try_into()?;
                Ok(FlatWhileStatement::init(var))
            }
            "copy" => {
                let var1: Var = words.next().ok_or(())?.try_into()?;
                let var2: Var = words.next().ok_or(())?.try_into()?;
                Ok(FlatWhileStatement::copy(var1, var2))
            }
            "while" => {
                let var: Var = words.next().ok_or(())?.try_into()?;
                Ok(FlatWhileStatement::while_not_zero(var))
            }
            "end" => Ok(FlatWhileStatement::while_end()),
            _ => Err(()),
        }
    } else {
        Err(())
    }
}

pub fn parse_flat(code: &str) -> Result<FlatWhileLanguage, ()> {
    let vec: Result<Vec<FlatWhileStatement>, ()> = code.lines().map(one_line_parse).collect();
    Ok(vec?.into())
}

pub fn parse(code: &str) -> Result<WhileLanguage, ()> {
    let vec: Result<Vec<FlatWhileStatement>, ()> = code.lines().map(one_line_parse).collect();
    Ok(FlatWhileLanguage::from(vec?).try_into()?)
}
