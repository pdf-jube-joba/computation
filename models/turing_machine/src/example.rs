use crate::machine::*;
use crate::manipulation::{Interpretation, TuringMachineBuilder};

#[derive(Debug, Clone, PartialEq)]
pub struct BinInt(usize);

impl BinInt {
    fn zero() -> Sign {
        Sign::try_from("0").unwrap()
    }
    fn one() -> Sign {
        Sign::try_from("1").unwrap()
    }
    fn part() -> Sign {
        Sign::try_from("-").unwrap()
    }
}

impl From<usize> for BinInt {
    fn from(value: usize) -> Self {
        BinInt(value)
    }
}

impl TryFrom<&str> for BinInt {
    type Error = String;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value.trim().parse::<usize>() {
            Ok(i) => Ok(BinInt(i)),
            Err(_) => Err("failed on parse bin".into())
        }
    }
}

impl From<&BinInt> for String {
    fn from(value: &BinInt) -> Self {
        value.0.to_string()
    }
}

pub fn str_to_two_bin(str: &str) -> Option<(BinInt, BinInt)> {
    let mut parts = str.split(',');
    let first = BinInt::try_from(parts.next()?.trim().trim_start_matches('(')).ok()?;
    let second = BinInt::try_from(parts.next()?.trim().trim_end_matches(')')).ok()?;
    Some((first, second))
}

pub fn two_bin_to_str((u1, u2): &(BinInt, BinInt)) -> String {
    format!("({},{})", String::from(u1), String::from(u2))
}

// 0 か 1 の並んだテープを受け取り、BinIntに変換する。
// 他の文字が含まれていた場合は失敗する。
fn vecsign_to_bin(vec: Vec<Sign>) -> Result<BinInt, String> {
    let u = vec.iter()
        .enumerate()
        .map(|(i, sign)| match sign {
            _ if *sign == BinInt::zero() => Ok(0),
            _ if *sign == BinInt::one() => Ok(2^i),
            _ => Err("error sign".to_string()),
        })
        .collect::<Result<Vec<_>, _>>()?
        .iter().sum::<usize>();
    Ok(u.into())
}

fn bin_to_vecsign(BinInt(u): BinInt) -> Vec<Sign> {
    if u == 0 {
        return vec![BinInt::zero()];
    }
    let mut bits = Vec::new();
    let mut n = u;
    while n > 0 {
        bits.push(n % 2 == 1);
        n /= 2;
    }
    bits.reverse();
    bits.iter()
        .map(|&b| if b { BinInt::one() } else { BinInt::zero() })
        .collect()
}

// fn tape_to_bin(tape: TapeAsVec) -> Option<usize> {

// }

// ".. 1001 .." => "9"
fn read(tape: TapeAsVec) -> Result<String, String> {
    let u = vecsign_to_bin(tape.right)?;
    Ok(format!("{}", String::from(&u)))
}

// "(2, 3)" => "... 10 . 11 ..."
fn write(str: String) -> Result<TapeAsVec, String> {
    let (u1, u2) = str_to_two_bin(&str).ok_or("err on str -> bin")?;
    let mut right = bin_to_vecsign(u1);
    right.push(BinInt::part());
    right.extend_from_slice(&bin_to_vecsign(u2));
    right.push(BinInt::part());
    let tape = TapeAsVec {
        left: Vec::new(),
        head: BinInt::part(),
        right,
    };
    Ok(tape)
}

impl BinInt {
    pub fn interpretation_str() -> Interpretation<String, String> {
        Interpretation::new(write, read)
    }
    pub fn interpretation() -> Interpretation<(BinInt, BinInt), BinInt> {
        todo!()
    }
}

// fn bin_adder() -> TuringMachineBuilder<usize, usize> {


// }

pub fn bin_adder_str() -> TuringMachineBuilder<String, String> {
    let mut builder = TuringMachineBuilder::new("bin_adder", BinInt::interpretation_str()).unwrap();
    builder
        .init_state(State::try_from("start").unwrap())
        .code_from_str(include_str!("bin_adder.txt")).unwrap()
        .accepted_state(vec![State::try_from("end").unwrap()]);
    builder
}

fn bin_adder() -> TuringMachineBuilder<(BinInt, BinInt), BinInt> {
    let mut builder = TuringMachineBuilder::new("bin_adder", BinInt::interpretation()).unwrap();
    builder
        .init_state(State::try_from("start").unwrap())
        .code_from_str(include_str!("bin_adder.txt")).unwrap()
        .accepted_state(vec![State::try_from("end").unwrap()]);
    builder
}

mod tests {

    #[test]
    fn test_parse() {
        use super::{str_to_two_bin, two_bin_to_str};
        assert_eq!(Some((1.into(), 2.into())), str_to_two_bin("(1, 2)"));
        assert_eq!(Some((10.into(), 21.into())), str_to_two_bin("  ( 10  , 21 )"));
        assert_eq!(Some((1.into(), 2.into())), str_to_two_bin("1, 2"));

        assert_eq!(
            two_bin_to_str(&str_to_two_bin("(1, 2").unwrap()),
            "(1,2)".to_string()
        );
    }
    #[test]
    fn test_usize_vec_bool() {
        use super::{BinInt, bin_to_vecsign};
        assert_eq!(vec![BinInt::zero()], bin_to_vecsign(0.into()));
        assert_eq!(vec![BinInt::one()], bin_to_vecsign(1.into()));
        assert_eq!(vec![BinInt::one(), BinInt::zero()], bin_to_vecsign(2.into()));
        assert_eq!(vec![BinInt::one(), BinInt::one()], bin_to_vecsign(3.into()));
    }
    #[test]
    fn other() {
        use super::{bin_adder, two_bin_to_str};
        let mut builder = bin_adder();
        let u = (10.into(), 2.into());
        builder.input(u);
        let mut machine = builder.build().unwrap();
        machine.step(100);
    }
}
