use crate::serde::Serialize;
use crate::serde_json::Value;
use crate::wasm_bindgen::prelude::JsValue;
use crate::{Compiler, Machine, StepResult, TextCodec};

pub fn step_machine_impl<T>(machine: &mut Option<T>, rinput: &str) -> Result<Value, String>
where
    T: Machine,
    T::SnapShot: Into<Value>,
{
    let current = machine
        .take()
        .ok_or_else(|| "Machine not initialized".to_string())?;
    let parsed = T::parse_rinput(rinput)?;
    match current.step(parsed)? {
        StepResult::Continue {
            next,
            output: routput,
        } => {
            *machine = Some(next);
            Ok(crate::serde_json::json!({
                "kind": "continue",
                "routput": routput.print(),
            }))
        }
        StepResult::Halt {
            snapshot,
            output: foutput,
        } => Ok(crate::serde_json::json!({
            "kind": "halt",
            "snapshot": Into::<Value>::into(snapshot),
            "foutput": foutput.print(),
        })),
    }
}

pub fn current_machine_impl<T>(machine: &Option<T>) -> Result<JsValue, JsValue>
where
    T: Machine,
    T::SnapShot: Into<Value>,
{
    let machine = machine
        .as_ref()
        .ok_or_else(|| JsValue::from_str("Machine not initialized"))?;
    let snapshot = T::current(machine);
    let json: Value = snapshot.into();
    json.serialize(&crate::serde_wasm_bindgen::Serializer::json_compatible())
        .map_err(|e| JsValue::from_str(&e.to_string()))
}

pub fn create_machine_impl<T: Machine>(code: &str, ainput: &str) -> Result<T, JsValue>
where
    T::SnapShot: Into<Value>,
{
    let code = T::parse_code(code).map_err(|e| JsValue::from_str(&e))?;
    let ainput = T::parse_ainput(ainput).map_err(|e| JsValue::from_str(&e))?;
    T::make(code, ainput).map_err(|e| JsValue::from_str(&e))
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

pub fn decode_routput_impl<T: Compiler>(output: &str) -> Result<String, JsValue> {
    let output_target = <<<T as Compiler>::Target as Machine>::ROutput as TextCodec>::parse(output)
        .map_err(|e| JsValue::from_str(&e))?;
    let output_source = T::decode_routput(output_target).map_err(|e| JsValue::from_str(&e))?;
    Ok(output_source.print())
}

pub fn decode_foutput_impl<T: Compiler>(output: &str) -> Result<String, JsValue> {
    let output_target = <<<T as Compiler>::Target as Machine>::FOutput as TextCodec>::parse(output)
        .map_err(|e| JsValue::from_str(&e))?;
    let output_source = T::decode_foutput(output_target).map_err(|e| JsValue::from_str(&e))?;
    Ok(output_source.print())
}

#[macro_export]
macro_rules! web_model {
    ($machine:path) => {
        mod __web_model {
            use $crate::serde::Serialize;
            use $crate::wasm_bindgen::prelude::JsValue;

            thread_local! {
                static MACHINE: std::cell::RefCell<Option<$machine>> = std::cell::RefCell::new(None);
            }

            #[$crate::wasm_bindgen::prelude::wasm_bindgen(wasm_bindgen = $crate::wasm_bindgen)]
            pub fn step_machine(rinput: &str) -> Result<JsValue, JsValue> {
                MACHINE.with(|machine| {
                    let mut machine = machine.borrow_mut();
                    let value = $crate::web_util::step_machine_impl::<$machine>(&mut machine, rinput)
                        .map_err(|e| JsValue::from_str(&e))?;
                    value
                        .serialize(&$crate::serde_wasm_bindgen::Serializer::json_compatible())
                        .map_err(|e| JsValue::from_str(&e.to_string()))
                })
            }

            #[$crate::wasm_bindgen::prelude::wasm_bindgen(wasm_bindgen = $crate::wasm_bindgen)]
            pub fn current_machine() -> Result<JsValue, JsValue> {
                MACHINE.with(|machine| {
                    let machine = machine.borrow();
                    $crate::web_util::current_machine_impl::<$machine>(&machine)
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
            pub fn decode_routput(output: &str) -> Result<String, JsValue> {
                $crate::web_util::decode_routput_impl::<$compiler>(output)
            }

            #[$crate::wasm_bindgen::prelude::wasm_bindgen(wasm_bindgen = $crate::wasm_bindgen)]
            pub fn decode_foutput(output: &str) -> Result<String, JsValue> {
                $crate::web_util::decode_foutput_impl::<$compiler>(output)
            }
        }

        pub use __web_compiler::{
            compile_ainput,
            compile_code,
            compile_rinput,
            decode_foutput,
            decode_routput,
        };

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
