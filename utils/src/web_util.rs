use serde_json::Value;
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
            serde_json::to_string(&serde_json::json!({
                "kind": "continue",
                "routput": routput.print(),
            }))
            .map_err(|e| e.to_string())
        }
        StepResult::Halt {
            snapshot,
            output: foutput,
        } => serde_json::to_string(&serde_json::json!({
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
    serde_json::to_string(&json).map_err(|e| e.to_string())
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

#[macro_export]
macro_rules! web_model {
    ($machine:path) => {
        use $crate::component_bindings::model::Guest;

        thread_local! {
            static MACHINE: std::cell::RefCell<Option<$machine>> = std::cell::RefCell::new(None);
        }

        struct Component;

        impl Guest for Component {
            fn step_machine(rinput: String) -> Result<String, String> {
                MACHINE.with(|machine| {
                    let mut machine = machine.borrow_mut();
                    $crate::web_util::step_machine_impl::<$machine>(&mut machine, &rinput)
                })
            }

            fn current_machine() -> Result<String, String> {
                MACHINE.with(|machine| {
                    let machine = machine.borrow();
                    $crate::web_util::current_machine_impl::<$machine>(&machine)
                })
            }

            fn create(input: String, ainput: String) -> Result<String, String> {
                let machine = $crate::web_util::create_machine_impl::<$machine>(&input, &ainput)?;
                MACHINE.with(|state| {
                    *state.borrow_mut() = Some(machine);
                });
                Ok("ok".to_string())
            }
        }

        $crate::__export_component_model!(Component);

        fn main() {}
    };
}

#[macro_export]
macro_rules! web_compiler {
    ($compiler:path) => {
        use $crate::component_bindings::compiler::Guest;

        struct Component;

        impl Guest for Component {
            fn compile_code(input: String) -> Result<String, String> {
                $crate::web_util::compile_code_impl::<$compiler>(&input)
            }

            fn compile_ainput(ainput: String) -> Result<String, String> {
                $crate::web_util::compile_ainput_impl::<$compiler>(&ainput)
            }

            fn compile_rinput(rinput: String) -> Result<String, String> {
                $crate::web_util::compile_rinput_impl::<$compiler>(&rinput)
            }

            fn decode_routput(output: String) -> Result<String, String> {
                $crate::web_util::decode_routput_impl::<$compiler>(&output)
            }

            fn decode_foutput(output: String) -> Result<String, String> {
                $crate::web_util::decode_foutput_impl::<$compiler>(&output)
            }
        }

        $crate::__export_component_compiler!(Component);

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
