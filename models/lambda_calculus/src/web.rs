use crate::machine::{LambdaTerm, MarkedTerm};
use serde::Serialize;
use utils::MealyMachine;
use wasm_bindgen::prelude::*;

#[derive(Debug, Clone, Serialize)]
pub enum LambdaTermWasm {
    Var(String),
    Abs {
        var: String,
        body: Box<LambdaTermWasm>,
    },
    App {
        func: Box<LambdaTermWasm>,
        arg: Box<LambdaTermWasm>,
    },
    Red {
        var: String,
        body: Box<LambdaTermWasm>,
        arg: Box<LambdaTermWasm>,
    },
}

impl MealyMachine for LambdaTermWasm {
    type Input = usize;
    type Output = ();
    type This = LambdaTermWasm;

    fn parse_self(input: &str) -> Result<Self, String> {
        todo!()
    }

    fn parse_input(input: &str) -> Result<Self::Input, String> {
        todo!()
    }

    fn step(&mut self, input: Self::Input) -> Result<Option<Self::Output>, String> {
        todo!()
    }

    fn current(&self) -> Self::This {
        todo!()
    }
}
