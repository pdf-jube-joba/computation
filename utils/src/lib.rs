use serde::Serialize;

pub mod alphabet;
pub mod bool;
pub mod number;
pub mod parse;
pub mod variable;

// ここら辺がないと動かないが、
// utils 側でも import をしておかないと、 `#[wasm_bindgen]` マクロが動かない。
pub use serde_wasm_bindgen;

pub trait TextCodec: Sized {
    fn parse(text: &str) -> Result<Self, String>;
    fn write_fmt(&self, f: &mut impl std::fmt::Write) -> std::fmt::Result;
    fn print(&self) -> String {
        let mut s = String::new();
        self.write_fmt(&mut s).unwrap();
        s
    }
}

pub trait Machine: Sized {
    type Code: Serialize + TextCodec; // static code
    type AInput: Serialize + TextCodec; // ahead of time input
    type SnapShot: Serialize; // representation of the current state
    type RInput: Serialize + TextCodec; // runtime input
    type Output: Serialize + TextCodec; // output after a step

    fn parse_code(code: &str) -> Result<Self::Code, String> {
        Self::Code::parse(code)
    }
    fn parse_ainput(ainput: &str) -> Result<Self::AInput, String> {
        Self::AInput::parse(ainput)
    }
    fn parse_rinput(rinput: &str) -> Result<Self::RInput, String> {
        Self::RInput::parse(rinput)
    }
    fn make(code: Self::Code, ainput: Self::AInput) -> Result<Self, String>;
    fn step(&mut self, rinput: Self::RInput) -> Result<Option<Self::Output>, String>;
    fn current(&self) -> Self::SnapShot;
}

pub trait Compiler: Sized {
    type Source: Machine; // source code
    type Target: Machine; // target code

    fn compile(
        source: <<Self as Compiler>::Source as Machine>::Code,
    ) -> Result<<<Self as Compiler>::Target as Machine>::Code, String>;
    fn encode_ainput(
        ainput: <<Self as Compiler>::Source as Machine>::AInput,
    ) -> Result<<<Self as Compiler>::Target as Machine>::AInput, String>;
    fn encode_rinput(
        rinput: <<Self as Compiler>::Source as Machine>::RInput,
    ) -> Result<<<Self as Compiler>::Target as Machine>::RInput, String>;
    fn decode_output(
        output: <<Self as Compiler>::Target as Machine>::Output,
    ) -> Result<<<Self as Compiler>::Source as Machine>::Output, String>;
}

#[macro_export]
macro_rules! web_model {
    ($machine:ty) => {
        fn main() {}

        use wasm_bindgen::prelude::{wasm_bindgen, JsValue};

        pub trait WebView {
            fn step(&mut self, rinput: &str) -> Result<Option<String>, String>;
            fn current(&self) -> Result<JsValue, JsValue>;
        }

        impl<T> WebView for T
        where
            T: $crate::Machine,
        {
            fn step(&mut self, rinput: &str) -> Result<Option<String>, String> {
                let parsed = <Self as $crate::Machine>::parse_rinput(rinput)?;
                let output = <Self as $crate::Machine>::step(self, parsed)?;
                match output {
                    Some(o) => {
                        let s = <<Self as $crate::Machine>::Output as $crate::TextCodec>::print(&o);
                        Ok(Some(s))
                    }
                    None => Ok(None),
                }
            }

            fn current(&self) -> Result<JsValue, JsValue> {
                $crate::serde_wasm_bindgen::to_value(&<Self as $crate::Machine>::current(self))
                    .map_err(|e| JsValue::from_str(&e.to_string()))
            }
        }

        thread_local! {
            static MACHINE: std::cell::RefCell<Option<Box<dyn WebView>>> = std::cell::RefCell::new(None);
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
        fn create_machine<T: $crate::Machine + 'static>(
            code: &str,
            ainput: &str,
        ) -> Result<(), JsValue> {
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
        pub fn create(input: &str, ainput: &str) -> Result<(), JsValue> {
            create_machine::<$machine>(input, ainput)
        }
    };
}
