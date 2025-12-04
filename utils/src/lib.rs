use serde_json::Value;

pub mod alphabet;
pub mod bool;
pub mod number;
pub mod set;
pub mod variable;

// this trait represents a web view model
// any model that wants to be used in the web view must implement this trait
pub trait WebView {
    // this function is not object safe
    // => we expect implement individually for each model
    // fn create(&self) -> Result<Self, String>;
    
    // step the model with the given input and return output if terminated
    fn step(&mut self, input: &str) -> Result<Option<Value>, String>;
    // get the current state of the model as a Value
    fn current(&self) -> Value;
}

pub trait ToJsResult<T> {
    fn to_js(self) -> Result<T, String>;
}

impl<T> ToJsResult<T> for anyhow::Result<T> {
    fn to_js(self) -> Result<T, String> {
        self.map_err(|e| format!("{e:?}"))
    }
}
