#[macro_export]
macro_rules! web_model {
    ($machine:path) => {
        mod __web_model {
            use $crate::{Machine, TextCodec};
            use serde::Serialize;
            use serde_json::Value;
            use $crate::wasm_bindgen::prelude::JsValue;

            pub trait WebView {
                fn step(&mut self, rinput: &str) -> Result<Option<String>, String>;
                fn current(&self) -> Result<JsValue, JsValue>;
            }

            impl<T> WebView for T
            where
                T: $crate::Machine,
                <T as $crate::Machine>::SnapShot: Into<Value>,
            {
                fn step(&mut self, rinput: &str) -> Result<Option<String>, String> {
                    let parsed = <Self as $crate::Machine>::parse_rinput(rinput)?;
                    let output = <Self as $crate::Machine>::step(self, parsed)?;
                    match output {
                        Some(o) => {
                            let s =
                                <<Self as $crate::Machine>::Output as $crate::TextCodec>::print(
                                    &o,
                                );
                            Ok(Some(s))
                        }
                        None => Ok(None),
                    }
                }

                fn current(&self) -> Result<JsValue, JsValue> {
                    let snapshot = <Self as $crate::Machine>::current(self);
                    let json: Value = snapshot.into();
                    json.serialize(&$crate::serde_wasm_bindgen::Serializer::json_compatible())
                        .map_err(|e| JsValue::from_str(&e.to_string()))
                }
            }

            thread_local! {
                static MACHINE: std::cell::RefCell<Option<Box<dyn WebView>>> = std::cell::RefCell::new(None);
            }

            #[$crate::wasm_bindgen::prelude::wasm_bindgen(wasm_bindgen = $crate::wasm_bindgen)]
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

            #[$crate::wasm_bindgen::prelude::wasm_bindgen(wasm_bindgen = $crate::wasm_bindgen)]
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
            ) -> Result<(), JsValue> where <T as $crate::Machine>::SnapShot: Into<Value> {
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

            #[$crate::wasm_bindgen::prelude::wasm_bindgen(wasm_bindgen = $crate::wasm_bindgen)]
            pub fn create(input: &str, ainput: &str) -> Result<(), JsValue> {
                create_machine::<$machine>(input, ainput)
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
            use $crate::{Machine, TextCodec};
            use $crate::wasm_bindgen::prelude::JsValue;

            #[allow(dead_code)]
            fn compile_code_for<T: $crate::Compiler>(code: &str) -> Result<String, JsValue> {
                let source_code = <T::Source as $crate::Machine>::parse_code(code)
                    .map_err(|e| JsValue::from_str(&e))?;
                let target_code = T::compile(source_code).map_err(|e| JsValue::from_str(&e))?;
                Ok(target_code.print())
            }

            #[$crate::wasm_bindgen::prelude::wasm_bindgen(wasm_bindgen = $crate::wasm_bindgen)]
            pub fn compile_code(input: &str) -> Result<String, JsValue> {
                compile_code_for::<$compiler>(input)
            }

            #[allow(dead_code)]
            fn compile_ainput_for<T: $crate::Compiler>(ainput: &str) -> Result<String, JsValue> {
                let source_ainput = <T as $crate::Compiler>::Source::parse_ainput(ainput)
                    .map_err(|e| JsValue::from_str(&e))?;
                let target_ainput =
                    T::encode_ainput(source_ainput).map_err(|e| JsValue::from_str(&e))?;
                Ok(target_ainput.print())
            }

            #[$crate::wasm_bindgen::prelude::wasm_bindgen(wasm_bindgen = $crate::wasm_bindgen)]
            pub fn compile_ainput(ainput: &str) -> Result<String, JsValue> {
                compile_ainput_for::<$compiler>(ainput)
            }

            #[allow(dead_code)]
            fn compile_rinput_for<T: $crate::Compiler>(rinput: &str) -> Result<String, JsValue> {
                let source_rinput = <T as $crate::Compiler>::Source::parse_rinput(rinput)
                    .map_err(|e| JsValue::from_str(&e))?;
                let target_rinput =
                    T::encode_rinput(source_rinput).map_err(|e| JsValue::from_str(&e))?;
                Ok(target_rinput.print())
            }

            #[$crate::wasm_bindgen::prelude::wasm_bindgen(wasm_bindgen = $crate::wasm_bindgen)]
            pub fn compile_rinput(rinput: &str) -> Result<String, JsValue> {
                compile_rinput_for::<$compiler>(rinput)
            }

            #[allow(dead_code)]
            fn decode_output_for<T: $crate::Compiler>(output: &str) -> Result<String, JsValue> {
                let output_target =
                    <<<T as $crate::Compiler>::Target as $crate::Machine>::Output as TextCodec>::parse(
                        output,
                    )
                    .map_err(|e| JsValue::from_str(&e))?;
                let output_source =
                    T::decode_output(output_target).map_err(|e| JsValue::from_str(&e))?;
                Ok(output_source.print())
            }

            #[$crate::wasm_bindgen::prelude::wasm_bindgen(wasm_bindgen = $crate::wasm_bindgen)]
            pub fn decode_output(output: &str) -> Result<String, JsValue> {
                decode_output_for::<$compiler>(output)
            }
        }

        pub use __web_compiler::{compile_ainput, compile_code, compile_rinput, decode_output};

        fn main() {}
    };
}
