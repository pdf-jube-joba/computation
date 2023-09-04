use crate::machine::{FlatWhileLanguage, FlatWhileStatement, Var, WhileLanguage};

fn one_line_parse(line: &str) -> Option<FlatWhileStatement> {
    let mut words = line.split_whitespace();
    match words.next()? {
        "inc" => {
            let var: Var = words.next()?.try_into().ok()?;
            Some(FlatWhileStatement::inc(var))
        }
        "dec" => {
            let var: Var = words.next()?.try_into().ok()?;
            Some(FlatWhileStatement::dec(var))
        }
        "init" => {
            let var: Var = words.next()?.try_into().ok()?;
            Some(FlatWhileStatement::init(var))
        }
        "copy" => {
            let var1: Var = words.next()?.try_into().ok()?;
            let var2: Var = words.next()?.try_into().ok()?;
            Some(FlatWhileStatement::copy(var1, var2))
        }
        "while" => {
            let var: Var = words.next()?.try_into().ok()?;
            Some(FlatWhileStatement::while_not_zero(var))
        }
        "end" => Some(FlatWhileStatement::while_end()),
        _ => None,
    }
}

pub fn parse_flat(code: &str) -> Option<FlatWhileLanguage> {
    let vec: Option<Vec<FlatWhileStatement>> = code.lines().map(one_line_parse).collect();
    Some(vec?.into())
}

pub fn parse(code: &str) -> Option<WhileLanguage> {
    let vec: Option<Vec<FlatWhileStatement>> = code.lines().map(one_line_parse).collect();
    FlatWhileLanguage::from(vec?).try_into().ok()
}
