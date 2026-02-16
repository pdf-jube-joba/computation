use turing_machine::machine::Sign;
use utils::parse::ParseTextCodec;

pub const BLANK_STR: &str = "-";
pub const PARTITION_STR: &str = "x";
pub const ONE_STR: &str = "l";
pub const HASH_STR: &str = "h";

pub fn blank_sign() -> Sign {
    BLANK_STR.parse_tc().unwrap()
}

pub fn partition_sign() -> Sign {
    PARTITION_STR.parse_tc().unwrap()
}

pub fn one_sign() -> Sign {
    ONE_STR.parse_tc().unwrap()
}

pub fn hash_sign() -> Sign {
    HASH_STR.parse_tc().unwrap()
}
