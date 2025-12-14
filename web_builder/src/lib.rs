use std::cell::RefCell;
use utils::{Compiler, Machine, TextCodec};
use wasm_bindgen::prelude::*;
pub trait WebView {
    fn step(&mut self, rinput: &str) -> Result<Option<JsValue>, String>;
    fn current(&self) -> Result<JsValue, JsValue>;
}

impl<T> WebView for T
where
    T: utils::Machine,
{
    fn step(&mut self, rinput: &str) -> Result<Option<JsValue>, String> {
        let parsed = <Self as utils::Machine>::parse_rinput(rinput)?;
        let output = <Self as utils::Machine>::step(self, parsed)?;
        match output {
            Some(o) => {
                let js = serde_wasm_bindgen::to_value(&o).map_err(|e| e.to_string())?;
                Ok(Some(js))
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

    Err(JsValue::from_str(
        "No machine type selected. Please enable a feature flag.",
    ))
}

#[allow(dead_code)]
fn compile_code_for<T: Compiler>(code: &str) -> Result<String, JsValue> {
    let source_code =
        <T::Source as Machine>::parse_code(code).map_err(|e| JsValue::from_str(&e))?;
    let target_code = T::compile(source_code).map_err(|e| JsValue::from_str(&e))?;
    <<<T as Compiler>::Target as Machine>::Code as TextCodec>::print(&target_code)
        .map_err(|e| JsValue::from_str(&e))
}

#[wasm_bindgen]
#[allow(unused)]
pub fn compile_code(input: &str) -> Result<String, JsValue> {
    Err(JsValue::from_str(
        "No compiler type selected. Please enable a feature flag.",
    ))
}

#[allow(dead_code)]
fn compile_ainput_for<T: Compiler>(ainput: &str) -> Result<String, JsValue> {
    let source_ainput =
        <T as Compiler>::Source::parse_ainput(ainput).map_err(|e| JsValue::from_str(&e))?;
    let target_ainput = T::encode_ainput(source_ainput).map_err(|e| JsValue::from_str(&e))?;
    <<<T as Compiler>::Target as Machine>::AInput as TextCodec>::print(&target_ainput)
        .map_err(|e| JsValue::from_str(&e))
}

#[wasm_bindgen]
#[allow(unused)]
pub fn compile_ainput(input: &str) -> Result<String, JsValue> {
    Err(JsValue::from_str(
        "No compiler type selected. Please enable a feature flag.",
    ))
}

#[allow(dead_code)]
fn compile_rinput_for<T: Compiler>(rinput: &str) -> Result<String, JsValue> {
    let source_rinput =
        <T as Compiler>::Source::parse_rinput(rinput).map_err(|e| JsValue::from_str(&e))?;
    let target_rinput = T::encode_rinput(source_rinput).map_err(|e| JsValue::from_str(&e))?;
    <<<T as Compiler>::Target as Machine>::RInput as TextCodec>::print(&target_rinput)
        .map_err(|e| JsValue::from_str(&e))
}

#[wasm_bindgen]
#[allow(unused)]
pub fn compile_rinput(input: &str) -> Result<String, JsValue> {
    Err(JsValue::from_str(
        "No compiler type selected. Please enable a feature flag.",
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

        fn print(data: &Self) -> Result<String, String> {
            Ok(data.count.to_string())
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

        fn print(data: &Self) -> Result<String, String> {
            match data {
                Command::Increment => Ok("inc".to_string()),
                Command::Decrement => Ok("dec".to_string()),
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
