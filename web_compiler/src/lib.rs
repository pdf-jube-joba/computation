use utils::{Compiler, Machine, TextCodec};
use wasm_bindgen::prelude::*;

#[allow(dead_code)]
fn compile_code_for<T: Compiler>(code: &str) -> Result<String, JsValue> {
    let source_code =
        <T::Source as Machine>::parse_code(code).map_err(|e| JsValue::from_str(&e))?;
    let target_code = T::compile(source_code).map_err(|e| JsValue::from_str(&e))?;
    Ok(target_code.print())
}

#[allow(dead_code)]
fn compile_ainput_for<T: Compiler>(ainput: &str) -> Result<String, JsValue> {
    let source_ainput =
        <T as Compiler>::Source::parse_ainput(ainput).map_err(|e| JsValue::from_str(&e))?;
    let target_ainput = T::encode_ainput(source_ainput).map_err(|e| JsValue::from_str(&e))?;
    Ok(target_ainput.print())
}

#[allow(dead_code)]
fn compile_rinput_for<T: Compiler>(rinput: &str) -> Result<String, JsValue> {
    let source_rinput =
        <T as Compiler>::Source::parse_rinput(rinput).map_err(|e| JsValue::from_str(&e))?;
    let target_rinput = T::encode_rinput(source_rinput).map_err(|e| JsValue::from_str(&e))?;
    Ok(target_rinput.print())
}

#[allow(dead_code)]
fn decode_output_for<T: Compiler>(output: &str) -> Result<String, JsValue> {
    let output_target = <<<T as Compiler>::Target as Machine>::Output as TextCodec>::parse(output)
        .map_err(|e| JsValue::from_str(&e))?;
    let output_source = T::decode_output(output_target).map_err(|e| JsValue::from_str(&e))?;
    Ok(output_source.print())
}

#[allow(unused_macros)]
macro_rules! compiler_api {
    ($feature: literal, $path: path) => {
        #[cfg(feature = $feature)]
        #[wasm_bindgen]
        pub fn compile_code(input: &str) -> Result<String, JsValue> {
            compile_code_for::<$path>(input)
        }
        #[cfg(feature = $feature)]
        #[wasm_bindgen]
        pub fn compile_ainput(ainput: &str) -> Result<String, JsValue> {
            compile_ainput_for::<$path>(ainput)
        }
        #[cfg(feature = $feature)]
        #[wasm_bindgen]
        pub fn compile_rinput(rinput: &str) -> Result<String, JsValue> {
            compile_rinput_for::<$path>(rinput)
        }
        #[cfg(feature = $feature)]
        #[wasm_bindgen]
        pub fn decode_output(output: &str) -> Result<String, JsValue> {
            decode_output_for::<$path>(output)
        }
    };
}

compiler_api! {"recursive_function-lambda_calculus", recursive_function_to_lambda_calculus::Rec2LamCompiler}
