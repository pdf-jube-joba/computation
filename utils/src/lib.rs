pub mod bool;
pub mod number;
pub mod set;
pub mod alphabet;
pub mod variable;

pub trait ToJsResult<T> {
    fn to_js(self) -> Result<T, String>;
}

impl<T> ToJsResult<T> for anyhow::Result<T> {
    fn to_js(self) -> Result<T, String> {
        self.map_err(|e| format!("{e:?}"))
    }
}
