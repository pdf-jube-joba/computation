use crate::machine::*;
use anyhow::{bail, Ok, Result};
use pest::{iterators::Pair, Parser};

#[derive(pest_derive::Parser)]
#[grammar = "my_HDL.pest"]
struct MyParser;
