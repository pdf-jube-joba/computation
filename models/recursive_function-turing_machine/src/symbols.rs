use std::fmt::Display;

use turing_machine::machine::Sign;
use utils::TextCodec;

#[derive(Debug, Clone)]
pub enum S {
    B, // '-' blank
    L, // 'l' flag
    X, // 'x' partition
}

impl From<S> for Sign {
    fn from(s: S) -> Self {
        match s {
            S::B => Sign::blank(), // "-" blank
            S::L => Sign::parse("l").unwrap(),
            S::X => Sign::parse("x").unwrap(),
        }
    }
}

impl Display for S {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s: Sign = self.clone().into();
        TextCodec::write_fmt(&s, f)
    }
}
