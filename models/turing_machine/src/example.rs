use crate::machine::*;
use crate::manipulation::{Interpretation, TuringMachineBuilder};

#[derive(Debug, Clone)]
pub struct BinInt(usize);

fn one() -> Sign {
    Sign::try_from("1").unwrap()
}

fn zero() -> Sign {
    Sign::try_from("0").unwrap()
}

pub fn str_to_bin(str: &str) -> Option<usize> {
    str.trim().parse().ok()
}

pub fn str_to_two_bin(str: &str) -> Option<(usize, usize)> {
    let mut parts = str.split(',');
    let first = str_to_bin(parts.next()?.trim().trim_start_matches('('))?;
    let second = str_to_bin(parts.next()?.trim().trim_end_matches(')'))?;
    Some((first, second))
}

fn vec_sign_to_usize(vec: Vec<Sign>) -> Result<usize, String> {
    let vec: Vec<Option<bool>> = vec
        .iter()
        .map(|sign| match sign {
            _ if *sign == Sign::blank() => Ok(None),
            _ if *sign == one() => Ok(Some(true)),
            _ if *sign == zero() => Ok(Some(false)),
            _ => Err("error sign".to_string()),
        })
        .collect::<Result<_, _>>()?;

    let first = vec
        .iter()
        .enumerate()
        .find(|(_, o)| o.is_some())
        .ok_or("not found sign".to_string())?
        .0;

    let last = vec
        .iter()
        .rev()
        .enumerate()
        .find(|(_, o)| o.is_some())
        .ok_or("not found sign".to_string())?
        .0;

    let vec: Vec<bool> = (vec[first..=last])
        .iter()
        .rev()
        .map(|s| s.ok_or("contains blank in center"))
        .collect::<Result<_, _>>()?;

    let sum = vec
        .into_iter()
        .enumerate()
        .map(|(i, b)| if b { 0 } else { 2 ^ i })
        .sum();

    Ok(sum)
}

// fn tape_to_bin(tape: TapeAsVec) -> Option<usize> {

// }

// ".. 1001 .." => "9"
fn read(tape: TapeAsVec) -> Result<String, String> {
    let u = vec_sign_to_usize(tape.right)?;
    Ok(format!("{u}"))
}

fn usize_to_bin_vec(u: usize) -> Vec<Sign> {
    if u == 0 {
        return vec![zero()];
    }
    let mut bits = Vec::new();
    let mut n = u;
    while n > 0 {
        bits.push(n % 2 == 1);
        n /= 2;
    }
    bits.reverse();
    bits.iter()
        .map(|&b| if b { one() } else { zero() })
        .collect()
}

pub fn two_bin_to_str((u1, u2): (usize, usize)) -> String {
    format!("({u1},{u2})")
}

// "(2, 3)" => "... 10 . 11 ..."
fn write(str: String) -> Result<TapeAsVec, String> {
    fn to_tape((u1, u2): (usize, usize)) -> TapeAsVec {
        let mut right = usize_to_bin_vec(u1);
        right.push(Sign::blank());
        right.extend_from_slice(&usize_to_bin_vec(u2));

        TapeAsVec {
            left: Vec::new(),
            head: Sign::blank(),
            right,
        }
    }
    Ok(to_tape(str_to_two_bin(&str).ok_or("".to_string())?))
}

impl BinInt {
    pub fn interpretation() -> Interpretation<String, String> {
        Interpretation::new(write, read)
    }
}

pub fn bin_adder() -> TuringMachineBuilder<String, String> {
    let mut builder = TuringMachineBuilder::new("bin_adder", BinInt::interpretation()).unwrap();
    builder
        .init_state(State::try_from("start").unwrap())
        .code_push_str("")
        .unwrap()
        .code_push_str("")
        .unwrap()
        .accepted_state(vec![State::try_from("end").unwrap()]);
    builder
}

mod tests {

    #[test]
    fn test_parse() {
        use super::{str_to_two_bin, two_bin_to_str};
        assert_eq!(Some((1, 2)), str_to_two_bin("(1, 2)"));
        assert_eq!(Some((10, 21)), str_to_two_bin("  ( 10  , 21 )"));
        assert_eq!(Some((1, 2)), str_to_two_bin("1, 2"));

        assert_eq!(
            two_bin_to_str(str_to_two_bin("(1, 2").unwrap()),
            "(1,2)".to_string()
        );
    }
    #[test]
    fn test_usize_vec_bool() {
        use super::{one, usize_to_bin_vec, zero};

        assert_eq!(vec![zero()], usize_to_bin_vec(0));
        assert_eq!(vec![one()], usize_to_bin_vec(1));
        assert_eq!(vec![one(), zero()], usize_to_bin_vec(2));
        assert_eq!(vec![one(), one()], usize_to_bin_vec(3));
    }
    #[test]
    fn other() {
        use super::{bin_adder, two_bin_to_str};
        let mut builder = bin_adder();
        let u = (10, 2);
        let str = two_bin_to_str(u);
        builder.input(str);
        builder.build().unwrap();
    }
}
