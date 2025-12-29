use std::cell::RefCell;
use utils::{Machine, TextCodec};
use wasm_bindgen::prelude::*;
pub trait WebView {
    fn step(&mut self, rinput: &str) -> Result<Option<String>, String>;
    fn current(&self) -> Result<JsValue, JsValue>;
}

impl<T> WebView for T
where
    T: utils::Machine,
{
    fn step(&mut self, rinput: &str) -> Result<Option<String>, String> {
        let parsed = <Self as utils::Machine>::parse_rinput(rinput)?;
        let output = <Self as utils::Machine>::step(self, parsed)?;
        match output {
            Some(o) => {
                Ok(Some(o.print()))
            }
            None => Ok(None),
        }
    }

    fn current(&self) -> Result<JsValue, JsValue> {
        serde_wasm_bindgen::to_value(&<Self as utils::Machine>::current(self))
            .map_err(|e| JsValue::from_str(&e.to_string()))
    }
}

thread_local! {
    static MACHINE: RefCell<Option<Box<dyn WebView>>> = RefCell::new(None);
}

#[wasm_bindgen]
pub fn step_machine(rinput: &str) -> Result<Option<String>, JsValue> {
    MACHINE.with(|machine| {
        let mut machine = machine.borrow_mut();
        let m = machine
            .as_mut()
            .ok_or_else(|| JsValue::from_str("Machine not initialized"))?;
        let result = m.step(rinput).map_err(|e| JsValue::from_str(&e))?;
        Ok(result)
    })
}

#[wasm_bindgen]
pub fn current_machine() -> Result<JsValue, JsValue> {
    MACHINE.with(|machine| {
        let machine = machine.borrow();
        let m = machine
            .as_ref()
            .ok_or_else(|| JsValue::from_str("Machine not initialized"))?;
        m.current()
    })
}

#[allow(dead_code)]
fn create_machine<T: Machine + 'static>(code: &str, ainput: &str) -> Result<(), JsValue> {
    MACHINE.with(|machine| {
        let mut machine = machine.borrow_mut();
        *machine = None;
    });
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

#[wasm_bindgen]
#[allow(unused)]
pub fn create(input: &str, ainput: &str) -> Result<(), JsValue> {
    #[cfg(feature = "example")]
    return create_machine::<example::Counter>(input, ainput);

    #[cfg(feature = "turing_machine")]
    return create_machine::<turing_machine::machine::TuringMachineSet>(input, ainput);

    #[cfg(feature = "lambda_calculus")]
    return create_machine::<lambda_calculus::machine::LambdaTerm>(input, ainput);

    #[cfg(feature = "goto_lang")]
    return create_machine::<goto_lang::machine::Program>(input, ainput);

    #[cfg(feature = "recursive_function")]
    return create_machine::<recursive_function::machine::Program>(input, ainput);

    #[cfg(feature = "tiny_isa")]
    return create_machine::<tiny_isa::Environment>(input, ainput);

    #[cfg(feature = "symbolic_asm")]
    return create_machine::<symbolic_asm::Environment>(input, ainput);

    Err(JsValue::from_str(
        "No machine type selected. Please enable a feature flag.",
    ))
}

#[cfg(feature = "example")]
mod example {
    use serde::Serialize;
    use utils::{Machine, TextCodec};

    #[derive(Clone, Serialize)]
    pub struct Counter {
        pub count: usize,
    }

    impl TextCodec for Counter {
        fn parse(text: &str) -> Result<Self, String> {
            let counter: usize = if text.trim().is_empty() {
                0
            } else {
                text.trim().parse::<usize>().map_err(|e| e.to_string())?
            };
            Ok(Counter { count: counter })
        }
        fn write_fmt(&self, f: &mut impl std::fmt::Write) -> std::fmt::Result {
            write!(f, "{}", self.count)
        }
    }

    #[derive(Serialize)]
    pub enum Command {
        Increment,
        Decrement,
    }

    impl TextCodec for Command {
        fn parse(text: &str) -> Result<Self, String> {
            match text.trim() {
                "inc" => Ok(Command::Increment),
                "dec" => Ok(Command::Decrement),
                _ => Err("Invalid command".to_string()),
            }
        }

        fn write_fmt(&self, f: &mut impl std::fmt::Write) -> std::fmt::Result {
            match self {
                Command::Increment => write!(f, "inc"),
                Command::Decrement => write!(f, "dec"),
            }
        }
    }

    impl Machine for Counter {
        type Code = Counter;
        type AInput = ();
        type RInput = Command;
        type Output = String;
        type SnapShot = Counter;

        fn make(code: Self::Code, _ainput: Self::AInput) -> Result<Self, String> {
            Ok(code)
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

        fn current(&self) -> Self::SnapShot {
            self.clone()
        }
    }
}
