use serde::Serialize;
use wasm_bindgen::JsValue;

pub mod alphabet;
pub mod bool;
pub mod number;
pub mod set;
pub mod variable;

// This trait is implemented by concrete models. It stays generic and not object-safe.
pub trait IntoWeb: Sized {
    type Input: Serialize;
    type Output: Serialize;
    type This: Serialize;

    fn parse_self(input: &str) -> Result<Self, String>;
    fn parse_input(input: &str) -> Result<Self::Input, String>;
    fn step(&mut self, input: Self::Input) -> Result<Option<Self::Output>, String>;
    fn current(&self) -> Self::This;
}

// Object-safe wrapper used at runtime.
pub trait WebView {
    fn step(&mut self, input: &str) -> Result<Option<JsValue>, String>;
    fn current(&self) -> JsValue;
}

pub trait ToJsResult<T> {
    fn to_js(self) -> Result<T, String>;
}

impl<T> ToJsResult<T> for anyhow::Result<T> {
    fn to_js(self) -> Result<T, String> {
        self.map_err(|e| format!("{e:?}"))
    }
}

impl<T> WebView for T
where
    T: IntoWeb,
{
    fn step(&mut self, input: &str) -> Result<Option<JsValue>, String> {
        let parsed = <Self as IntoWeb>::parse_input(input)?;
        let output = <Self as IntoWeb>::step(self, parsed)?;
        match output {
            Some(o) => {
                let js = serde_wasm_bindgen::to_value(&o).map_err(|e| e.to_string())?;
                Ok(Some(js))
            }
            None => Ok(None),
        }
    }

    fn current(&self) -> JsValue {
        serde_wasm_bindgen::to_value(&<Self as IntoWeb>::current(self))
            .unwrap_or_else(|e| JsValue::from_str(&e.to_string()))
    }
}
