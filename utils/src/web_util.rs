use crate::serde::Serialize;
use crate::serde_json::Value;
use crate::wasm_bindgen::prelude::JsValue;
use crate::{Compiler, Machine, TextCodec};

pub trait WebView {
    fn step(&mut self, rinput: &str) -> Result<Option<String>, String>;
    fn current(&self) -> Result<JsValue, JsValue>;
}

impl<T> WebView for T
where
    T: Machine,
    T::SnapShot: Into<Value>,
{
    fn step(&mut self, rinput: &str) -> Result<Option<String>, String> {
        let parsed = T::parse_rinput(rinput)?;
        let output = T::step(self, parsed)?;
        Ok(output.map(|o| o.print()))
    }

    fn current(&self) -> Result<JsValue, JsValue> {
        let snapshot = T::current(self);
        let json: Value = snapshot.into();
        json.serialize(&crate::serde_wasm_bindgen::Serializer::json_compatible())
            .map_err(|e| JsValue::from_str(&e.to_string()))
    }
}

pub fn step_machine_impl(
    machine: &mut Option<Box<dyn WebView>>,
    rinput: &str,
) -> Result<Option<String>, JsValue> {
    let machine = machine
        .as_mut()
        .ok_or_else(|| JsValue::from_str("Machine not initialized"))?;
    machine.step(rinput).map_err(|e| JsValue::from_str(&e))
}

pub fn current_machine_impl(machine: &Option<Box<dyn WebView>>) -> Result<JsValue, JsValue> {
    let machine = machine
        .as_ref()
        .ok_or_else(|| JsValue::from_str("Machine not initialized"))?;
    machine.current()
}

pub fn create_machine_impl<T: Machine + 'static>(
    code: &str,
    ainput: &str,
) -> Result<Box<dyn WebView>, JsValue>
where
    T::SnapShot: Into<Value>,
{
    let code = T::parse_code(code).map_err(|e| JsValue::from_str(&e))?;
    let ainput = T::parse_ainput(ainput).map_err(|e| JsValue::from_str(&e))?;
    let machine = T::make(code, ainput).map_err(|e| JsValue::from_str(&e))?;
    Ok(Box::new(machine))
}

pub fn compile_code_impl<T: Compiler>(code: &str) -> Result<String, JsValue> {
    let source_code = <T::Source as Machine>::parse_code(code).map_err(|e| JsValue::from_str(&e))?;
    let target_code = T::compile(source_code).map_err(|e| JsValue::from_str(&e))?;
    Ok(target_code.print())
}

pub fn compile_ainput_impl<T: Compiler>(ainput: &str) -> Result<String, JsValue> {
    let source_ainput = <T as Compiler>::Source::parse_ainput(ainput).map_err(|e| JsValue::from_str(&e))?;
    let target_ainput = T::encode_ainput(source_ainput).map_err(|e| JsValue::from_str(&e))?;
    Ok(target_ainput.print())
}

pub fn compile_rinput_impl<T: Compiler>(rinput: &str) -> Result<String, JsValue> {
    let source_rinput = <T as Compiler>::Source::parse_rinput(rinput).map_err(|e| JsValue::from_str(&e))?;
    let target_rinput = T::encode_rinput(source_rinput).map_err(|e| JsValue::from_str(&e))?;
    Ok(target_rinput.print())
}

pub fn decode_output_impl<T: Compiler>(output: &str) -> Result<String, JsValue> {
    let output_target = <<<T as Compiler>::Target as Machine>::Output as TextCodec>::parse(output)
        .map_err(|e| JsValue::from_str(&e))?;
    let output_source = T::decode_output(output_target).map_err(|e| JsValue::from_str(&e))?;
    Ok(output_source.print())
}

#[macro_export]
macro_rules! web_model {
    ($machine:path) => {
        mod __web_model {
            use $crate::wasm_bindgen::prelude::JsValue;

            thread_local! {
                static MACHINE: std::cell::RefCell<Option<Box<dyn $crate::web_util::WebView>>> = std::cell::RefCell::new(None);
            }

            #[$crate::wasm_bindgen::prelude::wasm_bindgen(wasm_bindgen = $crate::wasm_bindgen)]
            pub fn step_machine(rinput: &str) -> Result<Option<String>, JsValue> {
                MACHINE.with(|machine| {
                    let mut machine = machine.borrow_mut();
                    $crate::web_util::step_machine_impl(&mut machine, rinput)
                })
            }

            #[$crate::wasm_bindgen::prelude::wasm_bindgen(wasm_bindgen = $crate::wasm_bindgen)]
            pub fn current_machine() -> Result<JsValue, JsValue> {
                MACHINE.with(|machine| {
                    let machine = machine.borrow();
                    $crate::web_util::current_machine_impl(&machine)
                })
            }

            #[$crate::wasm_bindgen::prelude::wasm_bindgen(wasm_bindgen = $crate::wasm_bindgen)]
            pub fn create(input: &str, ainput: &str) -> Result<(), JsValue> {
                let machine = $crate::web_util::create_machine_impl::<$machine>(input, ainput)?;
                MACHINE.with(|state| {
                    *state.borrow_mut() = Some(machine);
                });
                Ok(())
            }
        }

        pub use __web_model::{create, current_machine, step_machine};

        fn main() {}
    };
}

#[macro_export]
macro_rules! web_compiler {
    ($compiler:path) => {
        mod __web_compiler {
            use $crate::wasm_bindgen::prelude::JsValue;

            #[$crate::wasm_bindgen::prelude::wasm_bindgen(wasm_bindgen = $crate::wasm_bindgen)]
            pub fn compile_code(input: &str) -> Result<String, JsValue> {
                $crate::web_util::compile_code_impl::<$compiler>(input)
            }

            #[$crate::wasm_bindgen::prelude::wasm_bindgen(wasm_bindgen = $crate::wasm_bindgen)]
            pub fn compile_ainput(ainput: &str) -> Result<String, JsValue> {
                $crate::web_util::compile_ainput_impl::<$compiler>(ainput)
            }

            #[$crate::wasm_bindgen::prelude::wasm_bindgen(wasm_bindgen = $crate::wasm_bindgen)]
            pub fn compile_rinput(rinput: &str) -> Result<String, JsValue> {
                $crate::web_util::compile_rinput_impl::<$compiler>(rinput)
            }

            #[$crate::wasm_bindgen::prelude::wasm_bindgen(wasm_bindgen = $crate::wasm_bindgen)]
            pub fn decode_output(output: &str) -> Result<String, JsValue> {
                $crate::web_util::decode_output_impl::<$compiler>(output)
            }
        }

        pub use __web_compiler::{compile_ainput, compile_code, compile_rinput, decode_output};

        fn main() {}
    };
}

#[macro_export]
macro_rules! json_text {
    ($text:expr) => {
        $crate::serde_json::json!({ "kind": "text", "text": $text })
    };
    ($text:expr, title: $title:expr) => {
        $crate::serde_json::json!({ "kind": "text", "text": $text, "title": $title })
    };
    ($text:expr, class: $class:expr) => {
        $crate::serde_json::json!({ "kind": "text", "text": $text, "className": $class })
    };
    ($text:expr, title: $title:expr, class: $class:expr) => {
        $crate::serde_json::json!({
            "kind": "text",
            "text": $text,
            "title": $title,
            "className": $class
        })
    };
    ($text:expr, class: $class:expr, title: $title:expr) => {
        $crate::serde_json::json!({
            "kind": "text",
            "text": $text,
            "title": $title,
            "className": $class
        })
    };
}
