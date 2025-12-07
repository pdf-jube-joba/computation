use std::cell::RefCell;
use utils::WebView;
use wasm_bindgen::prelude::*;

thread_local! {
    static MACHINE: RefCell<Option<Box<dyn WebView>>> = RefCell::new(None);
}

#[wasm_bindgen]
pub fn step_machine(input: &str) -> Result<JsValue, JsValue> {
    MACHINE.with(|machine| {
        let mut machine = machine.borrow_mut();
        let m = machine
            .as_mut()
            .ok_or_else(|| JsValue::from_str("Machine not initialized"))?;
        let result = m.step(input).map_err(|e| JsValue::from_str(&e))?;
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

pub fn create_machine<T: utils::MealyMachine + 'static>(input: &str) -> Result<(), JsValue> {
    let m = T::parse_self(input).map_err(|e| JsValue::from_str(&e))?;
    let boxed: Box<dyn WebView> = Box::new(m);
    MACHINE.with(|machine| {
        let mut machine = machine.borrow_mut();
        *machine = Some(boxed);
        Ok(())
    })
}

#[cfg(feature = "example")]
#[wasm_bindgen]
pub fn create(input: &str) -> Result<(), JsValue> {
    create_machine::<example::Counter>(input)
}

#[cfg(feature = "example")]
mod example {
    use serde::Serialize;
    use utils::MealyMachine;

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

    impl MealyMachine for Counter {
        type Input = Command;
        type Output = String;
        type This = Current;

        fn parse_self(input: &str) -> Result<Self, String> {
            let initial_count = input.trim().parse::<usize>().map_err(|e| e.to_string())?;
            if initial_count >= 10 {
                return Err("Initial count must be less than 10".to_string());
            }
            Ok(Counter {
                count: initial_count,
            })
        }

        fn parse_input(input: &str) -> Result<Self::Input, String> {
            match input.trim() {
                "increment" => Ok(Command::Increment),
                "decrement" => Ok(Command::Decrement),
                _ => Err("Invalid input".to_string()),
            }
        }

        fn step(&mut self, input: Self::Input) -> Result<Option<Self::Output>, String> {
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
