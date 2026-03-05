use crate::serde_json::Value;
use crate::{Compiler, Machine, StepResult, TextCodec};

pub fn step_machine_impl<T>(machine: &mut Option<T>, rinput: &str) -> Result<String, String>
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
            crate::serde_json::to_string(&crate::serde_json::json!({
                "kind": "continue",
                "routput": routput.print(),
            }))
            .map_err(|e| e.to_string())
        }
        StepResult::Halt {
            snapshot,
            output: foutput,
        } => crate::serde_json::to_string(&crate::serde_json::json!({
            "kind": "halt",
            "snapshot": Into::<Value>::into(snapshot),
            "foutput": foutput.print(),
        }))
        .map_err(|e| e.to_string()),
    }
}

pub fn current_machine_impl<T>(machine: &Option<T>) -> Result<String, String>
where
    T: Machine,
    T::SnapShot: Into<Value>,
{
    let machine = machine
        .as_ref()
        .ok_or_else(|| "Machine not initialized".to_string())?;
    let snapshot = T::current(machine);
    let json: Value = snapshot.into();
    crate::serde_json::to_string(&json).map_err(|e| e.to_string())
}

pub fn create_machine_impl<T: Machine>(code: &str, ainput: &str) -> Result<T, String>
where
    T::SnapShot: Into<Value>,
{
    let code = T::parse_code(code)?;
    let ainput = T::parse_ainput(ainput)?;
    T::make(code, ainput)
}

pub fn compile_code_impl<T: Compiler>(code: &str) -> Result<String, String> {
    let source_code = <T::Source as Machine>::parse_code(code)?;
    let target_code = T::compile(source_code)?;
    Ok(target_code.print())
}

pub fn compile_ainput_impl<T: Compiler>(ainput: &str) -> Result<String, String> {
    let source_ainput = <T as Compiler>::Source::parse_ainput(ainput)?;
    let target_ainput = T::encode_ainput(source_ainput)?;
    Ok(target_ainput.print())
}

pub fn compile_rinput_impl<T: Compiler>(rinput: &str) -> Result<String, String> {
    let source_rinput = <T as Compiler>::Source::parse_rinput(rinput)?;
    let target_rinput = T::encode_rinput(source_rinput)?;
    Ok(target_rinput.print())
}

pub fn decode_routput_impl<T: Compiler>(output: &str) -> Result<String, String> {
    let output_target = <<<T as Compiler>::Target as Machine>::ROutput as TextCodec>::parse(output)
        ?;
    let output_source = T::decode_routput(output_target)?;
    Ok(output_source.print())
}

pub fn decode_foutput_impl<T: Compiler>(output: &str) -> Result<String, String> {
    let output_target = <<<T as Compiler>::Target as Machine>::FOutput as TextCodec>::parse(output)
        ?;
    let output_source = T::decode_foutput(output_target)?;
    Ok(output_source.print())
}

#[macro_export]
macro_rules! web_model {
    ($machine:path) => {
        mod __web_model {
            use $crate::wasm_bindgen::prelude::JsValue;

            thread_local! {
                static MACHINE: std::cell::RefCell<Option<$machine>> = std::cell::RefCell::new(None);
            }

            #[$crate::wasm_bindgen::prelude::wasm_bindgen(wasm_bindgen = $crate::wasm_bindgen)]
            pub fn step_machine(rinput: &str) -> Result<String, JsValue> {
                MACHINE.with(|machine| {
                    let mut machine = machine.borrow_mut();
                    $crate::web_util::step_machine_impl::<$machine>(&mut machine, rinput)
                        .map_err(|e| JsValue::from_str(&e))
                })
            }

            #[$crate::wasm_bindgen::prelude::wasm_bindgen(wasm_bindgen = $crate::wasm_bindgen)]
            pub fn current_machine() -> Result<String, JsValue> {
                MACHINE.with(|machine| {
                    let machine = machine.borrow();
                    $crate::web_util::current_machine_impl::<$machine>(&machine)
                        .map_err(|e| JsValue::from_str(&e))
                })
            }

            #[$crate::wasm_bindgen::prelude::wasm_bindgen(wasm_bindgen = $crate::wasm_bindgen)]
            pub fn create(input: &str, ainput: &str) -> Result<String, JsValue> {
                let machine = $crate::web_util::create_machine_impl::<$machine>(input, ainput)?;
                MACHINE.with(|state| {
                    *state.borrow_mut() = Some(machine);
                });
                Ok("ok".to_string())
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
                    .map_err(|e| JsValue::from_str(&e))
            }

            #[$crate::wasm_bindgen::prelude::wasm_bindgen(wasm_bindgen = $crate::wasm_bindgen)]
            pub fn compile_ainput(ainput: &str) -> Result<String, JsValue> {
                $crate::web_util::compile_ainput_impl::<$compiler>(ainput)
                    .map_err(|e| JsValue::from_str(&e))
            }

            #[$crate::wasm_bindgen::prelude::wasm_bindgen(wasm_bindgen = $crate::wasm_bindgen)]
            pub fn compile_rinput(rinput: &str) -> Result<String, JsValue> {
                $crate::web_util::compile_rinput_impl::<$compiler>(rinput)
                    .map_err(|e| JsValue::from_str(&e))
            }

            #[$crate::wasm_bindgen::prelude::wasm_bindgen(wasm_bindgen = $crate::wasm_bindgen)]
            pub fn decode_routput(output: &str) -> Result<String, JsValue> {
                $crate::web_util::decode_routput_impl::<$compiler>(output)
                    .map_err(|e| JsValue::from_str(&e))
            }

            #[$crate::wasm_bindgen::prelude::wasm_bindgen(wasm_bindgen = $crate::wasm_bindgen)]
            pub fn decode_foutput(output: &str) -> Result<String, JsValue> {
                $crate::web_util::decode_foutput_impl::<$compiler>(output)
                    .map_err(|e| JsValue::from_str(&e))
            }
        }

        pub use __web_compiler::{
            compile_ainput, compile_code, compile_rinput, decode_foutput, decode_routput,
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
