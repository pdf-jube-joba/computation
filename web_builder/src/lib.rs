use std::cell::RefCell;
use utils::{Compiler, CompilerWrapper, Machine, TextCodec};
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

pub fn create_machine<T: Machine + 'static>(code: &str, ainput: &str) -> Result<(), JsValue> {
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

// return { "code": code, "ainput": ainput }
// where code = print(compile(code)), ainput = encode_ainput(ainput)
// contains target machine
pub fn create_compiler<T: Compiler + 'static>(
    code: &str,
    ainput: &str,
) -> Result<JsValue, JsValue> {
    MACHINE.with(|machine| {
        let mut machine = machine.borrow_mut();
        *machine = None;
    });
    let code_source =
        <T as Compiler>::Source::parse_code(code).map_err(|e| JsValue::from_str(&e))?;
    let code_target = T::compile(code_source).map_err(|e| JsValue::from_str(&e))?;
    let print_code_target =
        <<<T as Compiler>::Target as Machine>::Code as TextCodec>::print(&code_target)
            .map_err(|e| JsValue::from_str(&e))?;

    let ainput_source =
        <T as Compiler>::Source::parse_ainput(ainput).map_err(|e| JsValue::from_str(&e))?;
    let ainput_target = T::encode_ainput(ainput_source).map_err(|e| JsValue::from_str(&e))?;
    let print_ainput_target =
        <<<T as Compiler>::Target as Machine>::AInput as TextCodec>::print(&ainput_target)
            .map_err(|e| JsValue::from_str(&e))?;

    let return_json = serde_json::json!({
        "code": print_code_target,
        "ainput": print_ainput_target,
    });

    let return_value: JsValue = serde_wasm_bindgen::to_value(&return_json)
        .map_err(|e| JsValue::from_str(&e.to_string()))?;

    let target_machine =
        <T::Target as Machine>::make(code_target, ainput_target).map_err(|e| JsValue::from_str(&e))?;
    let machine = CompilerWrapper::<T>::from_target(target_machine);

    let boxed: Box<dyn WebView> = Box::new(machine);
    MACHINE.with(|machine| {
        let mut machine = machine.borrow_mut();
        *machine = Some(boxed);
        Ok::<(), String>(())
    })?;
    Ok(return_value)
}

#[cfg(feature = "turing_machine")]
#[wasm_bindgen]
pub fn create(input: &str, ainput: &str) -> Result<(), JsValue> {
    create_machine::<turing_machine::machine::TuringMachineSet>(input, ainput)
}

#[cfg(feature = "lambda_calculus")]
#[wasm_bindgen]
pub fn create(input: &str, ainput: &str) -> Result<(), JsValue> {
    create_machine::<lambda_calculus::machine::LambdaTerm>(input, ainput)
}

#[cfg(feature = "goto_lang")]
#[wasm_bindgen]
pub fn create(input: &str, ainput: &str) -> Result<(), JsValue> {
    create_machine::<goto_lang::machine::Program>(input, ainput)
}

#[cfg(feature = "recursive_function")]
#[wasm_bindgen]
pub fn create(input: &str, ainput: &str) -> Result<(), JsValue> {
    create_machine::<recursive_function::machine::Program>(input, ainput)
}

#[cfg(feature = "example")]
#[wasm_bindgen]
pub fn create(input: &str, ainput: &str) -> Result<(), JsValue> {
    create_machine::<example::Counter>(input, ainput)
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
