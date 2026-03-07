use crate::{Compiler, Machine, StepResult, TextCodec};

#[doc(hidden)]
pub mod wasm_model {
    use super::*;

    wit_bindgen::generate!({
        path: "wit",
        world: "model",
        pub_export_macro: true,
    });

    pub fn make_machine_impl<T: Machine>(code: &str, ainput: &str) -> Result<T, String> {
        let code = T::parse_code(code)?;
        let ainput = T::parse_ainput(ainput)?;
        T::make(code, ainput)
    }

    pub fn step_machine_impl<T>(machine: &mut Option<T>, rinput: &str) -> Result<String, String>
    where
        T: Machine,
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
                serde_json::to_string(&serde_json::json!({
                    "kind": "continue",
                    "routput": routput.print(),
                }))
                .map_err(|e| e.to_string())
            }
            StepResult::Halt { output: foutput } => {
                *machine = None;
                serde_json::to_string(&serde_json::json!({
                    "kind": "halt",
                    "foutput": foutput.print(),
                }))
                .map_err(|e| e.to_string())
            }
        }
    }

    pub fn snapshot_machine_impl<T>(machine: &Option<T>) -> Result<String, String>
    where
        T: Machine,
    {
        let machine = machine
            .as_ref()
            .ok_or_else(|| "Machine not initialized".to_string())?;
        let snapshot = T::snapshot(machine);
        serde_json::to_string(&snapshot).map_err(|e| e.to_string())
    }

    pub fn restore_machine_impl<T>(snapshot: &str) -> Result<T, String>
    where
        T: Machine,
    {
        let snapshot: T::SnapShot = serde_json::from_str(snapshot).map_err(|e| e.to_string())?;
        Ok(T::restore(snapshot))
    }

    pub fn render_machine_impl<T>(snapshot: &str) -> Result<String, String>
    where
        T: Machine,
    {
        let snapshot: T::SnapShot = serde_json::from_str(snapshot).map_err(|e| e.to_string())?;
        let rendered = T::render(snapshot);
        serde_json::to_string(&rendered).map_err(|e| e.to_string())
    }
}

#[doc(hidden)]
pub mod wasm_compiler {
    use super::*;

    wit_bindgen::generate!({
        path: "wit",
        world: "compiler",
        pub_export_macro: true,
    });

    pub fn compile_code_impl<T: Compiler>(code: &str) -> Result<String, String> {
        let source_code = <T::Source as Machine>::parse_code(code)?;
        let target_code = T::compile(source_code)?;
        Ok(target_code.print())
    }

    pub fn encode_ainput_impl<T: Compiler>(ainput: &str) -> Result<String, String> {
        let source_ainput = <T::Source as Machine>::parse_ainput(ainput)?;
        let target_ainput = T::encode_ainput(source_ainput)?;
        Ok(target_ainput.print())
    }

    pub fn encode_rinput_impl<T: Compiler>(rinput: &str) -> Result<String, String> {
        let source_rinput = <T::Source as Machine>::parse_rinput(rinput)?;
        let target_rinput = T::encode_rinput(source_rinput)?;
        Ok(target_rinput.print())
    }

    pub fn decode_routput_impl<T: Compiler>(output: &str) -> Result<String, String> {
        let output_target =
            <<<T as Compiler>::Target as Machine>::ROutput as TextCodec>::parse(output)?;
        let output_source = T::decode_routput(output_target)?;
        Ok(output_source.print())
    }

    pub fn decode_foutput_impl<T: Compiler>(output: &str) -> Result<String, String> {
        let output_target =
            <<<T as Compiler>::Target as Machine>::FOutput as TextCodec>::parse(output)?;
        let output_source = T::decode_foutput(output_target)?;
        Ok(output_source.print())
    }
}

#[macro_export]
macro_rules! web_model {
    ($machine:path) => {
        use $crate::wasm_util::wasm_model::Guest;

        thread_local! {
            static MACHINE: std::cell::RefCell<Option<$machine>> = std::cell::RefCell::new(None);
        }

        struct Component;

        impl Guest for Component {
            fn make(code: String, ainput: String) -> Result<(), String> {
                let machine = $crate::wasm_util::wasm_model::make_machine_impl::<$machine>(&code, &ainput)?;
                MACHINE.with(|state| {
                    *state.borrow_mut() = Some(machine);
                });
                Ok(())
            }

            fn step(rinput: String) -> Result<String, String> {
                MACHINE.with(|machine| {
                    let mut machine = machine.borrow_mut();
                    $crate::wasm_util::wasm_model::step_machine_impl::<$machine>(&mut machine, &rinput)
                })
            }

            fn snapshot() -> Result<String, String> {
                MACHINE.with(|machine| {
                    let machine = machine.borrow();
                    $crate::wasm_util::wasm_model::snapshot_machine_impl::<$machine>(&machine)
                })
            }

            fn restore(snapshot: String) -> Result<(), String> {
                let machine = $crate::wasm_util::wasm_model::restore_machine_impl::<$machine>(&snapshot)?;
                MACHINE.with(|state| {
                    *state.borrow_mut() = Some(machine);
                });
                Ok(())
            }

            fn render(snapshot: String) -> Result<String, String> {
                $crate::wasm_util::wasm_model::render_machine_impl::<$machine>(&snapshot)
            }
        }

        $crate::wasm_util::wasm_model::export!(
            Component with_types_in $crate::wasm_util::wasm_model
        );

        fn main() {}
    };
}

#[macro_export]
macro_rules! web_compiler {
    ($compiler:path) => {
        use $crate::wasm_util::wasm_compiler::Guest;

        struct Component;

        impl Guest for Component {
            fn compile_code(input: String) -> Result<String, String> {
                $crate::wasm_util::wasm_compiler::compile_code_impl::<$compiler>(&input)
            }

            fn encode_ainput(ainput: String) -> Result<String, String> {
                $crate::wasm_util::wasm_compiler::encode_ainput_impl::<$compiler>(&ainput)
            }

            fn encode_rinput(rinput: String) -> Result<String, String> {
                $crate::wasm_util::wasm_compiler::encode_rinput_impl::<$compiler>(&rinput)
            }

            fn decode_routput(output: String) -> Result<String, String> {
                $crate::wasm_util::wasm_compiler::decode_routput_impl::<$compiler>(&output)
            }

            fn decode_foutput(output: String) -> Result<String, String> {
                $crate::wasm_util::wasm_compiler::decode_foutput_impl::<$compiler>(&output)
            }
        }

        $crate::wasm_util::wasm_compiler::export!(
            Component with_types_in $crate::wasm_util::wasm_compiler
        );

        fn main() {}
    };
}

#[macro_export]
macro_rules! json_text {
    ($text:expr) => {
        serde_json::json!({ "kind": "text", "text": $text })
    };
    ($text:expr, title: $title:expr) => {
        serde_json::json!({ "kind": "text", "text": $text, "title": $title })
    };
    ($text:expr, class: $class:expr) => {
        serde_json::json!({ "kind": "text", "text": $text, "className": $class })
    };
    ($text:expr, title: $title:expr, class: $class:expr) => {
        serde_json::json!({
            "kind": "text",
            "text": $text,
            "title": $title,
            "className": $class
        })
    };
    ($text:expr, class: $class:expr, title: $title:expr) => {
        serde_json::json!({
            "kind": "text",
            "text": $text,
            "title": $title,
            "className": $class
        })
    };
}

#[macro_export]
macro_rules! model_entry {
    ($machine:path) => {
        #[cfg(target_arch = "wasm32")]
        $crate::web_model!($machine);

        #[cfg(not(target_arch = "wasm32"))]
        fn main() {}
    };
}

#[macro_export]
macro_rules! compiler_entry {
    ($compiler:path) => {
        #[cfg(target_arch = "wasm32")]
        $crate::web_compiler!($compiler);

        #[cfg(not(target_arch = "wasm32"))]
        fn main() {}
    };
}
