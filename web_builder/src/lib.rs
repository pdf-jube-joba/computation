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

// return { "code": code, "ainput": ainput }
// where code = print(compile(code)), ainput = encode_ainput(ainput)
// contains target machine
#[allow(dead_code)]
fn create_compiler<T: Compiler + 'static>(
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

#[allow(dead_code)]
fn encode_rinput_for<T: Compiler>(rinput: &str) -> Result<String, JsValue> {
    let source_rinput = <T::Source as Machine>::parse_rinput(rinput)
        .map_err(|e| JsValue::from_str(&e))?;
    let target_rinput = T::encode_rinput(source_rinput).map_err(|e| JsValue::from_str(&e))?;
    <<<T as Compiler>::Target as Machine>::RInput as TextCodec>::print(&target_rinput)
        .map_err(|e| JsValue::from_str(&e))
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

#[cfg(all(feature = "mod_counter", not(feature = "compiler-example-to-mod_counter")))]
#[wasm_bindgen]
pub fn create(input: &str, ainput: &str) -> Result<(), JsValue> {
    create_machine::<mod_counter::ModCounter>(input, ainput)
}

#[cfg(all(
    feature = "example",
    not(any(feature = "compiler-example-to-example", feature = "compiler-example-to-mod_counter"))
))]
#[wasm_bindgen]
pub fn create(input: &str, ainput: &str) -> Result<(), JsValue> {
    create_machine::<example::Counter>(input, ainput)
}

#[cfg(feature = "compiler-example-to-example")]
#[wasm_bindgen]
pub fn create(input: &str, ainput: &str) -> Result<JsValue, JsValue> {
    create_compiler::<example_compiler::ExampleIdentityCompiler>(input, ainput)
}

#[cfg(feature = "compiler-example-to-example")]
#[wasm_bindgen]
pub fn display_encode_rinput(rinput: &str) -> Result<String, JsValue> {
    encode_rinput_for::<example_compiler::ExampleIdentityCompiler>(rinput)
}

#[cfg(feature = "compiler-example-to-mod_counter")]
#[wasm_bindgen]
pub fn create(input: &str, ainput: &str) -> Result<JsValue, JsValue> {
    create_compiler::<compiler_example_to_mod_counter::ExampleToModCounter>(input, ainput)
}

#[cfg(feature = "compiler-example-to-mod_counter")]
#[wasm_bindgen]
pub fn display_encode_rinput(rinput: &str) -> Result<String, JsValue> {
    encode_rinput_for::<compiler_example_to_mod_counter::ExampleToModCounter>(rinput)
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

#[cfg(any(feature = "mod_counter", feature = "compiler-example-to-mod_counter"))]
mod mod_counter {
    use serde::Serialize;
    use utils::{Machine, TextCodec};

    #[derive(Clone, Serialize)]
    pub struct Code {
        pub modulus: usize,
        pub init: usize,
    }

    pub type AInput = ();

    #[derive(Clone, Serialize)]
    pub struct SnapShot {
        pub count: usize,
        pub remainder: usize,
    }

    #[derive(Clone, Serialize)]
    pub struct Output {
        pub count: usize,
        pub wrapped: bool,
    }

    #[derive(Clone, Serialize)]
    pub enum Command {
        Increment(usize),
        Decrement(usize),
    }

    impl TextCodec for Code {
        fn parse(text: &str) -> Result<Self, String> {
            if text.trim().is_empty() {
                return Err("expected: <modulus> [init]".to_string());
            }
            let mut parts = text.split_whitespace();
            let modulus = parts
                .next()
                .ok_or_else(|| "missing modulus".to_string())?
                .parse::<usize>()
                .map_err(|e| e.to_string())?;
            if modulus < 2 {
                return Err("modulus must be >= 2".to_string());
            }
            let init = parts
                .next()
                .map(|s| s.parse::<usize>().map_err(|e| e.to_string()))
                .transpose()?
                .unwrap_or(0);
            Ok(Code { modulus, init })
        }

        fn print(data: &Self) -> Result<String, String> {
            Ok(format!("{} {}", data.modulus, data.init))
        }
    }

    impl TextCodec for Command {
        fn parse(text: &str) -> Result<Self, String> {
            let trimmed = text.trim();
            if trimmed.is_empty() {
                return Err("empty command".to_string());
            }
            let mut parts = trimmed.split_whitespace();
            let head = parts.next().unwrap_or_default();
            let amount = parts
                .next()
                .map(|s| s.parse::<usize>().map_err(|e| e.to_string()))
                .transpose()?
                .unwrap_or(1);
            match head {
                "inc" | "+" => Ok(Command::Increment(amount)),
                "dec" | "-" => Ok(Command::Decrement(amount)),
                _ => Err("command must be inc|dec".to_string()),
            }
        }

        fn print(data: &Self) -> Result<String, String> {
            match data {
                Command::Increment(n) => Ok(format!("inc {}", n)),
                Command::Decrement(n) => Ok(format!("dec {}", n)),
            }
        }
    }

    pub struct ModCounter {
        modulus: usize,
        count: usize,
    }

    impl ModCounter {
        fn wrapped(old: usize, new: usize, modulus: usize) -> bool {
            old / modulus != new / modulus
        }
    }

    impl Machine for ModCounter {
        type Code = Code;
        type AInput = AInput;
        type SnapShot = SnapShot;
        type RInput = Command;
        type Output = Output;

        fn make(code: Self::Code, _ainput: Self::AInput) -> Result<Self, String> {
            Ok(ModCounter {
                modulus: code.modulus,
                count: code.init % code.modulus,
            })
        }

        fn step(&mut self, rinput: Self::RInput) -> Result<Option<Self::Output>, String> {
            let old = self.count;
            match rinput {
                Command::Increment(n) => {
                    self.count = self.count.saturating_add(n);
                }
                Command::Decrement(n) => {
                    self.count = self.count.saturating_sub(n);
                }
            }
            let wrapped = Self::wrapped(old, self.count, self.modulus);
            let output = Output {
                count: self.count,
                wrapped,
            };
            if wrapped {
                Ok(Some(output))
            } else {
                Ok(None)
            }
        }

        fn current(&self) -> Self::SnapShot {
            SnapShot {
                count: self.count,
                remainder: self.count % self.modulus,
            }
        }
    }
}

#[cfg(feature = "compiler-example-to-example")]
mod example_compiler {
    use utils::{Compiler, Machine};

    pub struct ExampleIdentityCompiler;

    impl Compiler for ExampleIdentityCompiler {
        type Source = super::example::Counter;
        type Target = super::example::Counter;

        fn compile(
            source: <<Self as Compiler>::Source as Machine>::Code,
        ) -> Result<<<Self as Compiler>::Target as Machine>::Code, String> {
            Ok(source)
        }

        fn encode_ainput(
            ainput: <<Self as Compiler>::Source as Machine>::AInput,
        ) -> Result<<<Self as Compiler>::Target as Machine>::AInput, String> {
            Ok(ainput)
        }

        fn encode_rinput(
            rinput: <<Self as Compiler>::Source as Machine>::RInput,
        ) -> Result<<<Self as Compiler>::Target as Machine>::RInput, String> {
            Ok(rinput)
        }

        fn decode_output(
            output: <<Self as Compiler>::Target as Machine>::Output,
        ) -> Result<<<Self as Compiler>::Source as Machine>::Output, String> {
            Ok(output)
        }
    }
}

#[cfg(feature = "compiler-example-to-mod_counter")]
mod compiler_example_to_mod_counter {
    use utils::{Compiler, Machine};

    pub struct ExampleToModCounter;

    impl Compiler for ExampleToModCounter {
        type Source = super::example::Counter;
        type Target = super::mod_counter::ModCounter;

        fn compile(
            source: <<Self as Compiler>::Source as Machine>::Code,
        ) -> Result<<<Self as Compiler>::Target as Machine>::Code, String> {
            let modulus = if source.count < 2 { 2 } else { source.count + 2 };
            Ok(super::mod_counter::Code {
                modulus,
                init: source.count % modulus,
            })
        }

        fn encode_ainput(
            _ainput: <<Self as Compiler>::Source as Machine>::AInput,
        ) -> Result<<<Self as Compiler>::Target as Machine>::AInput, String> {
            Ok(())
        }

        fn encode_rinput(
            rinput: <<Self as Compiler>::Source as Machine>::RInput,
        ) -> Result<<<Self as Compiler>::Target as Machine>::RInput, String> {
            let mapped = match rinput {
                super::example::Command::Increment => super::mod_counter::Command::Increment(1),
                super::example::Command::Decrement => super::mod_counter::Command::Decrement(1),
            };
            Ok(mapped)
        }

        fn decode_output(
            output: <<Self as Compiler>::Target as Machine>::Output,
        ) -> Result<<<Self as Compiler>::Source as Machine>::Output, String> {
            Ok(format!("wrapped at {}", output.count))
        }
    }
}
