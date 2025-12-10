use std::cell::RefCell;
use wasm_bindgen::prelude::*;

pub trait WebView {
    fn step(&mut self, input: &str) -> Result<Option<JsValue>, String>;
    fn current(&self) -> JsValue;
}

impl<T> WebView for T
where
    T: utils::Machine,
{
    fn step(&mut self, input: &str) -> Result<Option<JsValue>, String> {
        let parsed = <Self as utils::Machine>::parse_rinput(input)?;
        let output = <Self as utils::Machine>::step(self, parsed)?;
        match output {
            Some(o) => {
                let js = serde_wasm_bindgen::to_value(&o).map_err(|e| e.to_string())?;
                Ok(Some(js))
            }
            None => Ok(None),
        }
    }

    fn current(&self) -> JsValue {
        serde_wasm_bindgen::to_value(&<Self as utils::Machine>::current(self))
            .unwrap_or_else(|e| JsValue::from_str(&e.to_string()))
    }
}

thread_local! {
    static MACHINE: RefCell<Option<Box<dyn WebView>>> = RefCell::new(None);
}

#[wasm_bindgen]
pub fn step_machine(rinput: &str) -> Result<JsValue, JsValue> {
    MACHINE.with(|machine| {
        let mut machine = machine.borrow_mut();
        let m = machine
            .as_mut()
            .ok_or_else(|| JsValue::from_str("Machine not initialized"))?;
        let result = m.step(rinput).map_err(|e| JsValue::from_str(&e))?;
        Ok(result.unwrap_or(JsValue::UNDEFINED))
    })
}

#[wasm_bindgen]
pub fn current_machine() -> Result<JsValue, JsValue> {
    MACHINE.with(|machine| {
        let machine = machine.borrow();
        let m = machine
            .as_ref()
            .ok_or_else(|| JsValue::from_str("Machine not initialized"))?;
        Ok(m.current())
    })
}

pub fn create_machine<T: utils::Machine + 'static>(
    code: &str,
    ainput: &str,
) -> Result<(), JsValue> {
    let code = T::parse_code(code).map_err(|e| JsValue::from_str(&e))?;
    let ainput = T::parse_ainput(ainput).map_err(|e| JsValue::from_str(&e))?;
    let machine = T::make(code, ainput).map_err(|e| JsValue::from_str(&e))?;
    let boxed: Box<dyn WebView> = Box::new(machine);
    MACHINE.with(|machine| {
        let mut machine = machine.borrow_mut();
        *machine = Some(boxed);
        Ok(())
    })
}

#[cfg(feature = "turing_machine")]
#[wasm_bindgen]
pub fn create(input: &str, ainput: &str) -> Result<(), JsValue> {
    create_machine::<turing_machine::web::TuringMachineWeb>(input)
}

#[cfg(feature = "lambda_calculus")]
#[wasm_bindgen]
pub fn create(input: &str, ainput: &str) -> Result<(), JsValue> {
    create_machine::<lambda_calculus::machine::LambdaTerm>(input)
}

#[cfg(feature = "goto_lang")]
#[wasm_bindgen]
pub fn create(input: &str, ainput: &str) -> Result<(), JsValue> {
    create_machine::<goto_lang::machine::Program>(input)
}

#[cfg(feature = "recursive_function")]
#[wasm_bindgen]
pub fn create(input: &str, ainput: &str) -> Result<(), JsValue> {
    create_machine::<recursive_function::machine::Program>(input)
}

#[cfg(feature = "example")]
#[wasm_bindgen]
pub fn create(input: &str, ainput: &str) -> Result<(), JsValue> {
    create_machine::<example::Counter>(input)
}

#[cfg(feature = "example")]
mod example {
    use serde::Serialize;
    use utils::Machine;

    pub struct Counter {
        pub count: usize,
    }

    #[derive(Serialize)]
    pub struct Current {
        count: usize,
    }

    #[derive(Serialize)]
    pub enum Command {
        Increment,
        Decrement,
    }

    impl Machine for Counter {
        type Code = usize;
        type AInput = ();
        type RInput = Command;
        type Output = String;
        type This = Current;

        fn parse_code(code: &str) -> Result<Self::Code, String> {
            let initial_count = code.trim().parse::<usize>().map_err(|e| e.to_string())?;
            if initial_count >= 10 {
                return Err("Initial count must be less than 10".to_string());
            }
            Ok(initial_count)
        }

        fn parse_ainput(ainput: &str) -> Result<Self::AInput, String> {
            if !ainput.trim().is_empty() {
                return Err("AInput must be empty".to_string());
            }
            Ok(())
        }

        fn parse_rinput(rinput: &str) -> Result<Self::RInput, String> {
            match rinput.trim() {
                "Increment" => Ok(Command::Increment),
                "Decrement" => Ok(Command::Decrement),
                _ => Err("Invalid command".to_string()),
            }
        }

        fn make(code: Self::Code, _ainput: Self::AInput) -> Result<Self, String> {
            Ok(Counter { count: code })
        }

        fn step(&mut self, input: Self::RInput) -> Result<Option<Self::Output>, String> {
            match input {
                Command::Increment => {
                    self.count += 1;
                    if self.count >= 10 {
                        Ok(Some("End".to_string()))
                    } else {
                        Ok(None)
                    }
                }
                Command::Decrement => {
                    if self.count == 0 {
                        Err("Count cannot be negative".to_string())
                    } else {
                        self.count -= 1;
                        Ok(None)
                    }
                }
            }
        }

        fn current(&self) -> Self::This {
            Current { count: self.count }
        }
    }
}
